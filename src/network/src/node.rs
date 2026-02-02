//! Network node implementation
//!
//! Main P2P network node handling connections and message routing.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use libp2p::{
    gossipsub::{self, IdentTopic},
    mdns,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, noise, yamux, Multiaddr, PeerId,
};
use futures::prelude::*;
use crate::messages::{NetworkMessage, HandshakeMsg};
use crate::peer::{PeerManager, PeerInfo};
use aequitas_core::{Block, Transaction, Blockchain};

/// Default P2P port
pub const DEFAULT_PORT: u16 = 23420;

/// Topic for block announcements
pub const BLOCKS_TOPIC: &str = "aequitas/blocks/1";

/// Topic for transaction announcements  
pub const TX_TOPIC: &str = "aequitas/tx/1";

/// Combined network behaviour
#[derive(NetworkBehaviour)]
pub struct AequitasBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

/// Network node configuration
#[derive(Clone, Debug)]
pub struct NodeConfig {
    /// Listen address
    pub listen_addr: Multiaddr,
    
    /// Bootstrap peers
    pub bootstrap_peers: Vec<Multiaddr>,
    
    /// Is this a testnet node
    pub testnet: bool,
    
    /// Enable mDNS for local discovery
    pub enable_mdns: bool,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            listen_addr: format!("/ip4/0.0.0.0/tcp/{}", DEFAULT_PORT).parse().unwrap(),
            bootstrap_peers: Vec::new(),
            testnet: true,
            enable_mdns: true,
        }
    }
}

/// Network event types
#[derive(Clone, Debug)]
pub enum NetworkEvent {
    /// New peer connected
    PeerConnected(PeerId),
    
    /// Peer disconnected
    PeerDisconnected(PeerId),
    
    /// New block received
    NewBlock(Block),
    
    /// New transaction received
    NewTransaction(Transaction),
    
    /// Sync request from peer
    SyncRequest { peer: PeerId, from_height: u64 },
}

/// Network node
pub struct Node {
    /// Node configuration
    config: NodeConfig,
    
    /// Local peer ID
    local_peer_id: PeerId,
    
    /// Peer manager
    peer_manager: Arc<RwLock<PeerManager>>,
    
    /// Event sender
    event_tx: mpsc::Sender<NetworkEvent>,
    
    /// Event receiver
    event_rx: Option<mpsc::Receiver<NetworkEvent>>,
    
    /// Message sender for broadcasting
    broadcast_tx: mpsc::Sender<NetworkMessage>,
}

impl Node {
    /// Create a new network node
    pub fn new(config: NodeConfig) -> Self {
        let (event_tx, event_rx) = mpsc::channel(1000);
        let (broadcast_tx, _broadcast_rx) = mpsc::channel(1000);
        
        // Generate peer ID from random keypair
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        
        log::info!("Local peer ID: {}", local_peer_id);
        
        Self {
            config,
            local_peer_id,
            peer_manager: Arc::new(RwLock::new(PeerManager::new())),
            event_tx,
            event_rx: Some(event_rx),
            broadcast_tx,
        }
    }

    /// Start the network node loop
    pub async fn start(mut self) -> anyhow::Result<()> {
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|key| {
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .build()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

                Ok(AequitasBehaviour {
                    gossipsub: gossipsub::Behaviour::new(
                        gossipsub::MessageAuthenticity::Signed(key.clone()),
                        gossipsub_config,
                    )?,
                    mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?,
                })
            })?
            .build();

        // Subscribe to topics
        let blocks_topic = IdentTopic::new(BLOCKS_TOPIC);
        let tx_topic = IdentTopic::new(TX_TOPIC);
        swarm.behaviour_mut().gossipsub.subscribe(&blocks_topic)?;
        swarm.behaviour_mut().gossipsub.subscribe(&tx_topic)?;

        // Listen on all interfaces
        swarm.listen_on(self.config.listen_addr.clone())?;

        // Bootstrap
        for addr in &self.config.bootstrap_peers {
            swarm.dial(addr.clone())?;
        }

        log::info!("P2P Node started on {}", self.config.listen_addr);

        loop {
            tokio::select! {
                event = swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(AequitasBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, addr) in list {
                            log::info!("mDNS discovered peer: {}", peer_id);
                            swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                            swarm.dial(addr)?;
                        }
                    },
                    SwarmEvent::Behaviour(AequitasBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        propagation_source: peer_id,
                        message_id: _id,
                        message,
                    })) => {
                        if message.topic == blocks_topic.hash() {
                            if let Ok(block) = bincode::deserialize::<Block>(&message.data) {
                                let _ = self.event_tx.send(NetworkEvent::NewBlock(block)).await;
                            }
                        } else if message.topic == tx_topic.hash() {
                            if let Ok(tx) = bincode::deserialize::<Transaction>(&message.data) {
                                let _ = self.event_tx.send(NetworkEvent::NewTransaction(tx)).await;
                            }
                        }
                    },
                    SwarmEvent::NewListenAddr { address, .. } => {
                        log::info!("Local node is listening on {}", address);
                    },
                    _ => {}
                }
            }
        }
    }
    
    /// Get event receiver
    pub fn take_event_receiver(&mut self) -> Option<mpsc::Receiver<NetworkEvent>> {
        self.event_rx.take()
    }
    
    /// Get local peer ID
    pub fn local_peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }
    
    /// Broadcast a new block
    pub async fn broadcast_block(&self, block: &Block) -> anyhow::Result<()> {
        log::info!("Broadcasting block {} to network", hex::encode(block.hash()));
        // Note: Real broadcast would happen via the swarm. In a full implementation,
        // we'd use a channel to communicate with the swarm task.
        Ok(())
    }
}

/// Seed nodes for mainnet (to be updated)
pub const MAINNET_SEEDS: &[&str] = &[
    // Will be populated with community-run seed nodes
];

/// Seed nodes for testnet
pub const TESTNET_SEEDS: &[&str] = &[
    // Local testnet by default
];

/// DNS seeds (future)
pub const DNS_SEEDS: &[&str] = &[
    // "seed.aequitas.network",
];

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_creation() {
        let config = NodeConfig::default();
        let node = Node::new(config);
        
        assert!(!node.local_peer_id.to_string().is_empty());
    }
    
    #[test]
    fn test_default_config() {
        let config = NodeConfig::default();
        assert!(config.testnet);
        assert!(config.enable_mdns);
    }
}

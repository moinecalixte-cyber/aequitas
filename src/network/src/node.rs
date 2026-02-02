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
        let msg = NetworkMessage::NewBlock(crate::messages::NewBlockMsg {
            block: block.clone(),
            total_work: Vec::new(),
        });
        
        self.broadcast_tx.send(msg).await?;
        Ok(())
    }
    
    /// Broadcast a new transaction
    pub async fn broadcast_transaction(&self, tx: &Transaction) -> anyhow::Result<()> {
        let msg = NetworkMessage::NewTransactions(crate::messages::NewTxMsg {
            hashes: vec![tx.hash()],
        });
        
        self.broadcast_tx.send(msg).await?;
        Ok(())
    }
    
    /// Get connected peer count
    pub async fn peer_count(&self) -> usize {
        self.peer_manager.read().await.peer_count()
    }
    
    /// Get peer info
    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        self.peer_manager.read().await
            .connected_peers()
            .into_iter()
            .cloned()
            .collect()
    }
    
    /// Add bootstrap peer
    pub fn add_bootstrap_peer(&mut self, addr: Multiaddr) {
        self.config.bootstrap_peers.push(addr);
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

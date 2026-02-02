//! Network node implementation
//!
//! Main P2P network node handling connections and message routing.

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use libp2p::{
    gossipsub,
    mdns,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, noise, yamux, Multiaddr, PeerId,
};
use futures::stream::StreamExt;
use crate::peer::PeerManager;
use aequitas_core::{Block, Transaction};
use log;
use hex;

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
    pub listen_addr: Multiaddr,
    pub bootstrap_peers: Vec<Multiaddr>,
    pub testnet: bool,
    pub enable_mdns: bool,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            listen_addr: "/ip4/0.0.0.0/tcp/23420".parse().unwrap(),
            bootstrap_peers: Vec::new(),
            testnet: true,
            enable_mdns: true,
        }
    }
}

/// Network event types
#[derive(Clone, Debug)]
pub enum NetworkEvent {
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
    NewBlock(Block),
    NewTransaction(Transaction),
}

/// Network node
pub struct Node {
    config: NodeConfig,
    local_peer_id: PeerId,
    _peer_manager: Arc<RwLock<PeerManager>>,
    event_tx: mpsc::Sender<NetworkEvent>,
    event_rx: Option<mpsc::Receiver<NetworkEvent>>,
}

impl Node {
    pub fn new(config: NodeConfig) -> Self {
        let (event_tx, event_rx) = mpsc::channel(1000);
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        
        Self {
            config,
            local_peer_id,
            _peer_manager: Arc::new(RwLock::new(PeerManager::new())),
            event_tx,
            event_rx: Some(event_rx),
        }
    }

    pub async fn start(self, mut external_rx: mpsc::Receiver<Block>) -> anyhow::Result<()> {
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

        let blocks_topic = gossipsub::IdentTopic::new(BLOCKS_TOPIC);
        swarm.behaviour_mut().gossipsub.subscribe(&blocks_topic)?;

        swarm.listen_on(self.config.listen_addr.clone())?;

        log::info!("P2P Node started on {}", self.config.listen_addr);

        loop {
            tokio::select! {
                block = external_rx.recv() => {
                    if let Some(block) = block {
                        if let Ok(data) = bincode::serialize(&block) {
                            let _ = swarm.behaviour_mut().gossipsub.publish(blocks_topic.clone(), data);
                        }
                    }
                }
                event = swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(AequitasBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, addr) in list {
                            swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                            let _ = swarm.dial(addr);
                        }
                    },
                    SwarmEvent::Behaviour(AequitasBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        message,
                        ..
                    })) => {
                        if message.topic == blocks_topic.hash() {
                            if let Ok(block) = bincode::deserialize::<Block>(&message.data) {
                                let _ = self.event_tx.send(NetworkEvent::NewBlock(block)).await;
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }
    
    pub fn take_event_receiver(&mut self) -> Option<mpsc::Receiver<NetworkEvent>> {
        self.event_rx.take()
    }
}

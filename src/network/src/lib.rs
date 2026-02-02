//! Aequitas Network - P2P Communication Layer
//!
//! Handles peer discovery, block propagation, and transaction broadcasting.

pub mod node;
pub mod messages;
pub mod peer;

pub use node::Node;
pub use messages::NetworkMessage;
pub use peer::PeerManager;

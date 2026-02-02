//! Network message definitions
//!
//! Protocol messages for peer communication.

use serde::{Deserialize, Serialize};
use aequitas_core::{Block, Transaction};

/// Protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// Network magic bytes for mainnet
pub const MAINNET_MAGIC: [u8; 4] = [0xAE, 0x51, 0xC0, 0x01];

/// Network magic bytes for testnet
pub const TESTNET_MAGIC: [u8; 4] = [0xAE, 0x51, 0xDE, 0x5A];

/// Network message types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Handshake message for connection establishment
    Handshake(HandshakeMsg),
    
    /// Request block headers starting from a hash
    GetHeaders(GetHeadersMsg),
    
    /// Response with block headers
    Headers(HeadersMsg),
    
    /// Request full blocks by hash
    GetBlocks(GetBlocksMsg),
    
    /// Response with full blocks
    Blocks(BlocksMsg),
    
    /// Announce a new block
    NewBlock(NewBlockMsg),
    
    /// Announce new transactions
    NewTransactions(NewTxMsg),
    
    /// Request specific transactions
    GetTransactions(GetTxMsg),
    
    /// Response with transactions
    Transactions(TxMsg),
    
    /// Request mempool contents
    GetMempool,
    
    /// Response with mempool transaction hashes
    Mempool(MempoolMsg),
    
    /// Ping for connection keep-alive
    Ping(u64),
    
    /// Pong response
    Pong(u64),
    
    /// Peer address sharing
    Addr(AddrMsg),
    
    /// Request peer addresses
    GetAddr,
}

/// Handshake message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HandshakeMsg {
    /// Protocol version
    pub version: u32,
    
    /// Network magic
    pub magic: [u8; 4],
    
    /// Sender's best block height
    pub height: u64,
    
    /// Sender's best block hash
    pub best_hash: [u8; 32],
    
    /// Unix timestamp
    pub timestamp: i64,
    
    /// User agent string
    pub user_agent: String,
    
    /// Services offered (bitmask)
    pub services: u64,
}

impl HandshakeMsg {
    /// Create a new handshake message
    pub fn new(height: u64, best_hash: [u8; 32], testnet: bool) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            magic: if testnet { TESTNET_MAGIC } else { MAINNET_MAGIC },
            height,
            best_hash,
            timestamp: chrono::Utc::now().timestamp(),
            user_agent: format!("Aequitas/{}", env!("CARGO_PKG_VERSION")),
            services: 1, // Full node
        }
    }
}

/// Request headers message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetHeadersMsg {
    /// Block locator hashes (from tip to genesis)
    pub locator: Vec<[u8; 32]>,
    
    /// Stop hash (or zero for no limit)
    pub stop_hash: [u8; 32],
    
    /// Maximum headers to return
    pub max_headers: u32,
}

/// Headers response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeadersMsg {
    /// Block headers
    pub headers: Vec<aequitas_core::BlockHeader>,
}

/// Request blocks message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetBlocksMsg {
    /// Block hashes to fetch
    pub hashes: Vec<[u8; 32]>,
}

/// Blocks response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlocksMsg {
    /// Full blocks
    pub blocks: Vec<Block>,
}

/// New block announcement
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewBlockMsg {
    /// The new block
    pub block: Block,
    
    /// Total chain work (for fork detection)
    pub total_work: Vec<u8>,
}

/// New transactions announcement
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewTxMsg {
    /// Transaction hashes being announced
    pub hashes: Vec<[u8; 32]>,
}

/// Request transactions message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetTxMsg {
    /// Transaction hashes to fetch
    pub hashes: Vec<[u8; 32]>,
}

/// Transactions response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxMsg {
    /// Transactions
    pub transactions: Vec<Transaction>,
}

/// Mempool contents
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MempoolMsg {
    /// Transaction hashes in mempool
    pub hashes: Vec<[u8; 32]>,
}

/// Peer addresses
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddrMsg {
    /// List of peer addresses
    pub addresses: Vec<PeerAddr>,
}

/// Peer address info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerAddr {
    /// IP address (IPv4 or IPv6)
    pub ip: String,
    
    /// Port
    pub port: u16,
    
    /// Services offered
    pub services: u64,
    
    /// Last seen timestamp
    pub last_seen: i64,
}

impl NetworkMessage {
    /// Serialize message to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }
    
    /// Deserialize message from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }
    
    /// Get message type name
    pub fn type_name(&self) -> &'static str {
        match self {
            NetworkMessage::Handshake(_) => "handshake",
            NetworkMessage::GetHeaders(_) => "getheaders",
            NetworkMessage::Headers(_) => "headers",
            NetworkMessage::GetBlocks(_) => "getblocks",
            NetworkMessage::Blocks(_) => "blocks",
            NetworkMessage::NewBlock(_) => "newblock",
            NetworkMessage::NewTransactions(_) => "newtx",
            NetworkMessage::GetTransactions(_) => "gettx",
            NetworkMessage::Transactions(_) => "tx",
            NetworkMessage::GetMempool => "getmempool",
            NetworkMessage::Mempool(_) => "mempool",
            NetworkMessage::Ping(_) => "ping",
            NetworkMessage::Pong(_) => "pong",
            NetworkMessage::Addr(_) => "addr",
            NetworkMessage::GetAddr => "getaddr",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_handshake_serialization() {
        let msg = HandshakeMsg::new(100, [0u8; 32], false);
        let network_msg = NetworkMessage::Handshake(msg);
        
        let bytes = network_msg.to_bytes().unwrap();
        let decoded = NetworkMessage::from_bytes(&bytes).unwrap();
        
        assert_eq!(network_msg.type_name(), decoded.type_name());
    }
}

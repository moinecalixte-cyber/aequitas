//! Block structures for Aequitas blockchain
//!
//! Defines the Block and BlockHeader structures used throughout the network.

use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use chrono::{DateTime, Utc};
use crate::transaction::Transaction;
use crate::merkle::compute_merkle_root;

/// Block header containing metadata and proof-of-work data
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockHeader {
    /// Version of the block format
    pub version: u32,
    
    /// Hash of the previous block
    pub prev_hash: [u8; 32],
    
    /// Merkle root of all transactions in the block
    pub merkle_root: [u8; 32],
    
    /// Timestamp of block creation
    pub timestamp: DateTime<Utc>,
    
    /// Current difficulty target
    pub difficulty: u64,
    
    /// Nonce for proof-of-work
    pub nonce: u64,
    
    /// Height of this block in the chain
    pub height: u64,
    
    /// Extra data for AequiHash algorithm (epoch, dag info)
    pub extra_data: [u8; 32],
}

impl BlockHeader {
    /// Create a new block header
    pub fn new(
        prev_hash: [u8; 32],
        merkle_root: [u8; 32],
        height: u64,
        difficulty: u64,
    ) -> Self {
        Self {
            version: 1,
            prev_hash,
            merkle_root,
            timestamp: Utc::now(),
            difficulty,
            nonce: 0,
            height,
            extra_data: [0u8; 32],
        }
    }
    
    /// Compute the hash of this header
    pub fn hash(&self) -> [u8; 32] {
        let serialized = bincode::serialize(self).expect("Failed to serialize header");
        let mut hasher = Keccak256::new();
        hasher.update(&serialized);
        hasher.finalize().into()
    }
    
    /// Check if the header hash meets the difficulty target
    pub fn meets_difficulty(&self) -> bool {
        let hash = self.hash();
        let hash_value = u64::from_be_bytes(hash[0..8].try_into().unwrap());
        let target = u64::MAX / self.difficulty;
        hash_value <= target
    }
}

/// A complete block with header and transactions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    /// The block header
    pub header: BlockHeader,
    
    /// List of transactions in this block
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Create a new block with transactions
    pub fn new(
        prev_hash: [u8; 32],
        height: u64,
        difficulty: u64,
        transactions: Vec<Transaction>,
    ) -> Self {
        let merkle_root = compute_merkle_root(&transactions);
        let header = BlockHeader::new(prev_hash, merkle_root, height, difficulty);
        
        Self {
            header,
            transactions,
        }
    }
    
    /// Create the genesis block
    pub fn genesis() -> Self {
        let coinbase = Transaction::coinbase(
            Address::genesis_address(),
            GENESIS_REWARD,
            0,
        );
        
        Self::new(
            [0u8; 32],
            0,
            INITIAL_DIFFICULTY,
            vec![coinbase],
        )
    }
    
    /// Get the hash of this block
    pub fn hash(&self) -> [u8; 32] {
        self.header.hash()
    }
    
    /// Get the hex string representation of the block hash
    pub fn hash_hex(&self) -> String {
        hex::encode(self.hash())
    }
    
    /// Validate the block structure
    pub fn validate(&self) -> Result<(), BlockError> {
        // Check merkle root
        let computed_merkle = compute_merkle_root(&self.transactions);
        if computed_merkle != self.header.merkle_root {
            return Err(BlockError::InvalidMerkleRoot);
        }
        
        // Check proof of work
        if !self.header.meets_difficulty() {
            return Err(BlockError::InsufficientProofOfWork);
        }
        
        // Validate transactions
        for tx in &self.transactions {
            tx.validate()?;
        }
        
        Ok(())
    }
}

use crate::address::Address;

/// Initial block reward in AEQ (smallest unit)
pub const GENESIS_REWARD: u64 = 50_000_000_000; // 50 AEQ with 9 decimals

/// Initial difficulty target
pub const INITIAL_DIFFICULTY: u64 = 1_000_000;

/// Block validation errors
#[derive(Debug, thiserror::Error)]
pub enum BlockError {
    #[error("Invalid merkle root")]
    InvalidMerkleRoot,
    
    #[error("Insufficient proof of work")]
    InsufficientProofOfWork,
    
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(#[from] crate::transaction::TxError),
    
    #[error("Invalid previous hash")]
    InvalidPrevHash,
    
    #[error("Invalid timestamp")]
    InvalidTimestamp,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis();
        assert_eq!(genesis.header.height, 0);
        assert_eq!(genesis.header.prev_hash, [0u8; 32]);
        assert!(!genesis.transactions.is_empty());
    }
    
    #[test]
    fn test_block_hash() {
        let genesis = Block::genesis();
        let hash1 = genesis.hash();
        let hash2 = genesis.hash();
        assert_eq!(hash1, hash2);
    }
}

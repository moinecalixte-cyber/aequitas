//! Aequitas Core - Block and Blockchain structures
//! 
//! This module defines the fundamental building blocks of the Aequitas blockchain.

pub mod block;
pub mod transaction;
pub mod blockchain;
pub mod merkle;
pub mod address;
pub mod difficulty;

pub use block::{Block, BlockHeader, BlockError, GENESIS_REWARD, INITIAL_DIFFICULTY};
pub use transaction::{Transaction, TxInput, TxOutput, TxType, TxError};
pub use blockchain::{Blockchain, ChainError, UtxoId, HALVING_INTERVAL, MAX_SUPPLY, TREASURY_PERCENTAGE};
pub use address::{Address, Keypair, AddressError};
pub use difficulty::{Difficulty, TARGET_BLOCK_TIME};
pub use merkle::{compute_merkle_root, MerkleProof};

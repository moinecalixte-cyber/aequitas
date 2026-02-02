//! Transaction mempool

use std::collections::HashMap;
use aequitas_core::{Transaction, Address};

/// Maximum mempool size in transactions
pub const MAX_MEMPOOL_SIZE: usize = 10000;

/// Maximum transaction age in seconds
pub const MAX_TX_AGE: i64 = 3600; // 1 hour

/// Transaction pool entry
#[derive(Clone, Debug)]
pub struct MempoolEntry {
    /// The transaction
    pub transaction: Transaction,
    
    /// Time added to mempool
    pub added_at: i64,
    
    /// Fee in smallest units
    pub fee: u64,
    
    /// Fee per byte
    pub fee_per_byte: f64,
    
    /// Size in bytes
    pub size: usize,
}

impl MempoolEntry {
    /// Create new mempool entry
    pub fn new(transaction: Transaction, fee: u64) -> Self {
        let serialized = bincode::serialize(&transaction).unwrap_or_default();
        let size = serialized.len();
        let fee_per_byte = if size > 0 { fee as f64 / size as f64 } else { 0.0 };
        
        Self {
            transaction,
            added_at: chrono::Utc::now().timestamp(),
            fee,
            fee_per_byte,
            size,
        }
    }
    
    /// Check if expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now - self.added_at > MAX_TX_AGE
    }
}

/// Transaction mempool
pub struct Mempool {
    /// Transactions indexed by hash
    transactions: HashMap<[u8; 32], MempoolEntry>,
    
    /// Transaction hashes by sender
    by_sender: HashMap<Address, Vec<[u8; 32]>>,
}

impl Mempool {
    /// Create new mempool
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
            by_sender: HashMap::new(),
        }
    }
    
    /// Add transaction to mempool
    pub fn add(&mut self, tx: Transaction, fee: u64) -> Result<(), MempoolError> {
        // Check size limit
        if self.transactions.len() >= MAX_MEMPOOL_SIZE {
            return Err(MempoolError::MempoolFull);
        }
        
        let hash = tx.hash();
        
        // Check for duplicate
        if self.transactions.contains_key(&hash) {
            return Err(MempoolError::AlreadyExists);
        }
        
        // Validate transaction
        tx.validate().map_err(|e| MempoolError::InvalidTransaction(e.to_string()))?;
        
        // Add to mempool
        let entry = MempoolEntry::new(tx.clone(), fee);
        self.transactions.insert(hash, entry);
        
        Ok(())
    }
    
    /// Remove transaction
    pub fn remove(&mut self, hash: &[u8; 32]) -> Option<MempoolEntry> {
        self.transactions.remove(hash)
    }
    
    /// Get transaction
    pub fn get(&self, hash: &[u8; 32]) -> Option<&MempoolEntry> {
        self.transactions.get(hash)
    }
    
    /// Check if transaction exists
    pub fn contains(&self, hash: &[u8; 32]) -> bool {
        self.transactions.contains_key(hash)
    }
    
    /// Get all transaction hashes
    pub fn hashes(&self) -> Vec<[u8; 32]> {
        self.transactions.keys().copied().collect()
    }
    
    /// Get transactions for block (sorted by fee)
    pub fn get_for_block(&self, max_size: usize) -> Vec<Transaction> {
        let mut entries: Vec<_> = self.transactions.values().collect();
        
        // Sort by fee per byte (descending)
        entries.sort_by(|a, b| b.fee_per_byte.partial_cmp(&a.fee_per_byte).unwrap());
        
        let mut result = Vec::new();
        let mut total_size = 0;
        
        for entry in entries {
            if total_size + entry.size <= max_size {
                result.push(entry.transaction.clone());
                total_size += entry.size;
            }
        }
        
        result
    }
    
    /// Remove confirmed transactions
    pub fn remove_confirmed(&mut self, tx_hashes: &[[u8; 32]]) {
        for hash in tx_hashes {
            self.transactions.remove(hash);
        }
    }
    
    /// Remove expired transactions
    pub fn remove_expired(&mut self) {
        let expired: Vec<[u8; 32]> = self.transactions
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(hash, _)| *hash)
            .collect();
        
        for hash in expired {
            self.transactions.remove(&hash);
        }
    }
    
    /// Get mempool size
    pub fn size(&self) -> usize {
        self.transactions.len()
    }
    
    /// Get total fees
    pub fn total_fees(&self) -> u64 {
        self.transactions.values().map(|e| e.fee).sum()
    }
    
    /// Clear mempool
    pub fn clear(&mut self) {
        self.transactions.clear();
        self.by_sender.clear();
    }
}

impl Default for Mempool {
    fn default() -> Self {
        Self::new()
    }
}

/// Mempool errors
#[derive(Debug, thiserror::Error)]
pub enum MempoolError {
    #[error("Mempool is full")]
    MempoolFull,
    
    #[error("Transaction already exists")]
    AlreadyExists,
    
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    
    #[error("Insufficient fee")]
    InsufficientFee,
}

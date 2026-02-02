//! DAG (Directed Acyclic Graph) generation for AequiHash
//!
//! The DAG is a large dataset (~4GB) stored in GPU memory that makes
//! mining memory-hard and ASIC-resistant.

use sha3::{Digest, Keccak256};
use blake3;
use byteorder::{ByteOrder, LittleEndian};
use crate::aequihash::{EPOCH_LENGTH, DAG_SIZE, CACHE_SIZE, MIX_WORDS, compute_cache};
use std::sync::Arc;

/// DAG item size in 32-bit words
pub const DAG_ITEM_WORDS: usize = 16;

/// DAG item size in bytes
pub const DAG_ITEM_BYTES: usize = DAG_ITEM_WORDS * 4;

/// Number of parent lookups for DAG generation
pub const DAG_PARENTS: usize = 256;

/// DAG structure for an epoch
pub struct DAG {
    /// Epoch number
    epoch: u64,
    
    /// The full DAG dataset (for GPU mining)
    data: Option<Arc<Vec<u32>>>,
    
    /// Light cache (for verification)
    cache: Arc<Vec<u32>>,
    
    /// DAG size in words
    size_words: usize,
}

impl DAG {
    /// Create a light DAG (cache only, for verification)
    pub fn new_light(epoch: u64) -> Self {
        let cache = Arc::new(compute_cache(epoch, CACHE_SIZE));
        
        Self {
            epoch,
            data: None,
            cache,
            size_words: DAG_SIZE / 4,
        }
    }
    
    /// Create a full DAG (for mining)
    /// Warning: This allocates ~4GB of memory!
    pub fn new_full(epoch: u64) -> Self {
        log::info!("Generating DAG for epoch {}... This may take a few minutes.", epoch);
        
        let cache = Arc::new(compute_cache(epoch, CACHE_SIZE));
        let num_items = DAG_SIZE / DAG_ITEM_BYTES;
        let mut data = vec![0u32; DAG_SIZE / 4];
        
        // Generate DAG items
        for item_idx in 0..num_items {
            let item = Self::calc_dag_item(item_idx, &cache);
            let offset = item_idx * DAG_ITEM_WORDS;
            data[offset..offset + DAG_ITEM_WORDS].copy_from_slice(&item);
            
            if item_idx % (num_items / 100).max(1) == 0 {
                log::info!("DAG generation: {}%", item_idx * 100 / num_items);
            }
        }
        
        log::info!("DAG generation complete for epoch {}", epoch);
        
        Self {
            epoch,
            data: Some(Arc::new(data)),
            cache,
            size_words: DAG_SIZE / 4,
        }
    }
    
    /// Calculate a single DAG item
    fn calc_dag_item(item_idx: usize, cache: &[u32]) -> [u32; DAG_ITEM_WORDS] {
        let cache_words = cache.len();
        let mut mix = [0u32; DAG_ITEM_WORDS];
        
        // Initial mix from cache
        let init_idx = item_idx % (cache_words / DAG_ITEM_WORDS);
        let init_offset = init_idx * DAG_ITEM_WORDS;
        
        for i in 0..DAG_ITEM_WORDS {
            mix[i] = cache[(init_offset + i) % cache_words];
        }
        mix[0] ^= item_idx as u32;
        
        // FNV hash the initial mix
        let mut fnv_hash = Keccak256::new();
        for m in &mix {
            fnv_hash.update(&m.to_le_bytes());
        }
        let fnv_result = fnv_hash.finalize();
        for i in 0..4.min(DAG_ITEM_WORDS) {
            mix[i] = LittleEndian::read_u32(&fnv_result[i * 4..(i + 1) * 4]);
        }
        
        // Aggregate data from pseudorandom cache locations
        for parent in 0..DAG_PARENTS {
            let parent_idx = Self::fnv(
                item_idx as u32 ^ parent as u32,
                mix[parent % DAG_ITEM_WORDS]
            ) as usize % (cache_words / DAG_ITEM_WORDS);
            
            let parent_offset = parent_idx * DAG_ITEM_WORDS;
            
            for i in 0..DAG_ITEM_WORDS {
                let cache_val = cache[(parent_offset + i) % cache_words];
                mix[i] = Self::fnv(mix[i], cache_val);
            }
        }
        
        // Final hash
        let mut final_hash = blake3::Hasher::new();
        for m in &mix {
            final_hash.update(&m.to_le_bytes());
        }
        let result = final_hash.finalize();
        
        for i in 0..DAG_ITEM_WORDS.min(8) {
            mix[i] = LittleEndian::read_u32(&result.as_bytes()[i * 4..(i + 1) * 4]);
        }
        for i in 8..DAG_ITEM_WORDS {
            mix[i] = mix[i] ^ mix[i % 8];
        }
        
        mix
    }
    
    /// FNV-1a hash function for 32-bit integers
    fn fnv(a: u32, b: u32) -> u32 {
        (a.wrapping_mul(0x01000193)) ^ b
    }
    
    /// Get epoch
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    
    /// Get cache reference
    pub fn cache(&self) -> &[u32] {
        &self.cache
    }
    
    /// Get DAG data (if full DAG is loaded)
    pub fn data(&self) -> Option<&[u32]> {
        self.data.as_ref().map(|d| d.as_slice())
    }
    
    /// Check if this is a full DAG
    pub fn is_full(&self) -> bool {
        self.data.is_some()
    }
    
    /// Get approximate memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        let cache_size = self.cache.len() * 4;
        let dag_size = self.data.as_ref().map(|d| d.len() * 4).unwrap_or(0);
        cache_size + dag_size
    }
    
    /// Calculate a single item on-the-fly (for light verification)
    pub fn calc_item(&self, idx: usize) -> [u32; DAG_ITEM_WORDS] {
        Self::calc_dag_item(idx, &self.cache)
    }
}

/// DAG manager for caching DAGs across epochs
pub struct DAGManager {
    /// Current DAG
    current: Option<DAG>,
    
    /// Whether to generate full DAGs (for mining) or light (for verification)
    full_dag: bool,
}

impl DAGManager {
    /// Create a new DAG manager
    pub fn new(full_dag: bool) -> Self {
        Self {
            current: None,
            full_dag,
        }
    }
    
    /// Get or create DAG for epoch
    pub fn get_dag(&mut self, epoch: u64) -> &DAG {
        // Check if we need a new DAG
        let need_new = self.current.as_ref().map(|d| d.epoch() != epoch).unwrap_or(true);
        
        if need_new {
            let dag = if self.full_dag {
                DAG::new_full(epoch)
            } else {
                DAG::new_light(epoch)
            };
            self.current = Some(dag);
        }
        
        self.current.as_ref().unwrap()
    }
    
    /// Get DAG for a block height
    pub fn get_dag_for_height(&mut self, height: u64) -> &DAG {
        let epoch = height / EPOCH_LENGTH;
        self.get_dag(epoch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_light_dag_creation() {
        let dag = DAG::new_light(0);
        assert_eq!(dag.epoch(), 0);
        assert!(!dag.is_full());
        assert!(dag.cache().len() > 0);
    }
    
    #[test]
    fn test_dag_item_deterministic() {
        let dag = DAG::new_light(0);
        let item1 = dag.calc_item(0);
        let item2 = dag.calc_item(0);
        assert_eq!(item1, item2);
        
        let item3 = dag.calc_item(1);
        assert_ne!(item1, item3);
    }
    
    #[test]
    fn test_dag_manager() {
        let mut manager = DAGManager::new(false);
        
        let dag0 = manager.get_dag(0);
        assert_eq!(dag0.epoch(), 0);
        
        // Same epoch should return cached DAG
        let dag0_again = manager.get_dag(0);
        assert_eq!(dag0_again.epoch(), 0);
    }
}

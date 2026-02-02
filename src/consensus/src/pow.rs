//! Proof of Work verification and mining utilities
//!
//! Provides tools for validating and finding valid proofs of work.

use crate::aequihash::AequiHash;
use crate::dag::{DAG, DAGManager};
use num_bigint::BigUint;
use num_traits::One;

/// Difficulty to target conversion
pub fn difficulty_to_target(difficulty: u64) -> [u8; 32] {
    if difficulty == 0 {
        return [0xff; 32];
    }
    
    // Target = 2^256 / difficulty
    let max_target = BigUint::from(2u32).pow(256) - BigUint::one();
    let target = max_target / BigUint::from(difficulty);
    
    let target_bytes = target.to_bytes_be();
    let mut result = [0u8; 32];
    
    let start = 32 - target_bytes.len().min(32);
    result[start..].copy_from_slice(&target_bytes[..target_bytes.len().min(32)]);
    
    result
}

/// Compare two hashes (returns true if a <= b)
pub fn hash_less_or_equal(a: &[u8; 32], b: &[u8; 32]) -> bool {
    for i in 0..32 {
        if a[i] < b[i] {
            return true;
        }
        if a[i] > b[i] {
            return false;
        }
    }
    true
}

/// Proof of Work result
#[derive(Clone, Debug)]
pub struct ProofOfWork {
    /// The winning nonce
    pub nonce: u64,
    
    /// The resulting hash
    pub hash: [u8; 32],
    
    /// Mix hash (for verification)
    pub mix_hash: [u8; 32],
}

impl ProofOfWork {
    /// Verify this proof of work
    pub fn verify(
        &self,
        header_hash: &[u8; 32],
        difficulty: u64,
        epoch: u64,
        cache: &[u32],
    ) -> bool {
        let aequihash = AequiHash::new(epoch);
        let target = difficulty_to_target(difficulty);
        aequihash.verify(header_hash, self.nonce, &target, cache)
    }
}

/// Mining statistics
#[derive(Clone, Debug, Default)]
pub struct MiningStats {
    /// Total hashes computed
    pub hashes: u64,
    
    /// Start time (unix timestamp)
    pub start_time: i64,
    
    /// Blocks found
    pub blocks_found: u64,
    
    /// Current hashrate (H/s)
    pub hashrate: f64,
}

impl MiningStats {
    /// Update hashrate calculation
    pub fn update_hashrate(&mut self, current_time: i64) {
        let elapsed = (current_time - self.start_time) as f64;
        if elapsed > 0.0 {
            self.hashrate = self.hashes as f64 / elapsed;
        }
    }
}

/// CPU miner (for testing/reference)
pub struct CpuMiner {
    /// DAG manager
    dag_manager: DAGManager,
}

impl CpuMiner {
    /// Create a new CPU miner
    pub fn new() -> Self {
        Self {
            dag_manager: DAGManager::new(false), // Light DAG for CPU
        }
    }
    
    /// Mine a block (CPU reference implementation)
    pub fn mine(
        &mut self,
        header_hash: &[u8; 32],
        difficulty: u64,
        height: u64,
        start_nonce: u64,
        max_nonce: u64,
    ) -> Option<ProofOfWork> {
        let epoch = height / super::aequihash::EPOCH_LENGTH;
        let dag = self.dag_manager.get_dag(epoch);
        let aequihash = AequiHash::new(epoch);
        let target = difficulty_to_target(difficulty);
        let cache = dag.cache();
        
        for nonce in start_nonce..max_nonce {
            let hash = aequihash.hash_light(header_hash, nonce, cache);
            
            if hash_less_or_equal(&hash, &target) {
                return Some(ProofOfWork {
                    nonce,
                    hash,
                    mix_hash: hash, // Simplified for CPU
                });
            }
        }
        
        None
    }
    
    /// Benchmark the CPU miner
    pub fn benchmark(&mut self, seconds: u64) -> f64 {
        let header = [0u8; 32];
        let start = std::time::Instant::now();
        let mut hashes = 0u64;
        
        let epoch = 0;
        let dag = self.dag_manager.get_dag(epoch);
        let aequihash = AequiHash::new(epoch);
        let cache = dag.cache();
        
        while start.elapsed().as_secs() < seconds {
            for nonce in 0..1000 {
                let _ = aequihash.hash_light(&header, hashes + nonce, cache);
            }
            hashes += 1000;
        }
        
        let elapsed = start.elapsed().as_secs_f64();
        hashes as f64 / elapsed
    }
}

impl Default for CpuMiner {
    fn default() -> Self {
        Self::new()
    }
}

/// Work unit for mining pools
#[derive(Clone, Debug)]
pub struct WorkUnit {
    /// Block header hash
    pub header_hash: [u8; 32],
    
    /// Target difficulty
    pub difficulty: u64,
    
    /// Block height
    pub height: u64,
    
    /// Starting nonce
    pub start_nonce: u64,
    
    /// Ending nonce
    pub end_nonce: u64,
    
    /// Job ID
    pub job_id: String,
}

impl WorkUnit {
    /// Create a new work unit
    pub fn new(
        header_hash: [u8; 32],
        difficulty: u64,
        height: u64,
        job_id: String,
    ) -> Self {
        Self {
            header_hash,
            difficulty,
            height,
            start_nonce: 0,
            end_nonce: u64::MAX,
            job_id,
        }
    }
    
    /// Split into sub-units for parallel mining
    pub fn split(&self, num_parts: u64) -> Vec<WorkUnit> {
        let range = self.end_nonce - self.start_nonce;
        let part_size = range / num_parts;
        
        (0..num_parts)
            .map(|i| {
                let start = self.start_nonce + i * part_size;
                let end = if i == num_parts - 1 {
                    self.end_nonce
                } else {
                    start + part_size
                };
                
                WorkUnit {
                    header_hash: self.header_hash,
                    difficulty: self.difficulty,
                    height: self.height,
                    start_nonce: start,
                    end_nonce: end,
                    job_id: format!("{}-{}", self.job_id, i),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_difficulty_to_target() {
        let low_diff = difficulty_to_target(1);
        let high_diff = difficulty_to_target(1000000);
        
        // Higher difficulty = lower target
        assert!(high_diff < low_diff);
    }
    
    #[test]
    fn test_hash_comparison() {
        let a = [0u8; 32];
        let b = [1u8; 32];
        
        assert!(hash_less_or_equal(&a, &b));
        assert!(!hash_less_or_equal(&b, &a));
        assert!(hash_less_or_equal(&a, &a));
    }
    
    #[test]
    fn test_work_unit_split() {
        let work = WorkUnit::new([0u8; 32], 1000, 0, "test".to_string());
        let parts = work.split(4);
        
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0].start_nonce, 0);
    }
}

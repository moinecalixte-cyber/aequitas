//! Difficulty adjustment algorithm for Aequitas
//!
//! Implements a smooth difficulty adjustment algorithm (DAA) that adjusts
//! every block to maintain a target block time of 30 seconds.

use num_bigint::BigUint;
use num_traits::{One, Zero};

/// Target block time in seconds
pub const TARGET_BLOCK_TIME: u64 = 30;

/// Number of blocks to average for difficulty calculation
pub const DIFFICULTY_AVERAGING_WINDOW: u64 = 60;

/// Maximum difficulty adjustment per block (10%)
pub const MAX_ADJUSTMENT_FACTOR: f64 = 1.10;

/// Minimum difficulty adjustment per block (90%)
pub const MIN_ADJUSTMENT_FACTOR: f64 = 0.90;

/// Minimum difficulty value
pub const MIN_DIFFICULTY: u64 = 1000;

/// Difficulty target representation
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Difficulty {
    /// Difficulty value (higher = harder)
    value: u64,
}

impl Difficulty {
    /// Create a new difficulty from a value
    pub fn new(value: u64) -> Self {
        Self {
            value: value.max(MIN_DIFFICULTY),
        }
    }
    
    /// Get the difficulty value
    pub fn value(&self) -> u64 {
        self.value
    }
    
    /// Calculate target from difficulty
    /// Target = MAX_TARGET / difficulty
    pub fn target(&self) -> BigUint {
        let max_target = BigUint::from(u64::MAX);
        max_target / BigUint::from(self.value)
    }
    
    /// Check if a hash meets this difficulty
    pub fn hash_meets_target(&self, hash: &[u8; 32]) -> bool {
        let hash_value = BigUint::from_bytes_be(hash);
        hash_value <= self.target()
    }
    
    /// Calculate next difficulty based on block times
    ///
    /// # Arguments
    /// * `block_times` - Vector of (height, timestamp) for recent blocks
    ///
    /// # Returns
    /// New difficulty value
    pub fn calculate_next(
        current: u64,
        block_times: &[(u64, i64)],
    ) -> u64 {
        if block_times.len() < 2 {
            return current;
        }
        
        // Calculate average block time
        let window_size = block_times.len().min(DIFFICULTY_AVERAGING_WINDOW as usize);
        let recent = &block_times[block_times.len() - window_size..];
        
        let time_span = (recent.last().unwrap().1 - recent.first().unwrap().1) as f64;
        let block_count = (recent.len() - 1) as f64;
        
        if block_count == 0.0 {
            return current;
        }
        
        let average_time = time_span / block_count;
        let target_time = TARGET_BLOCK_TIME as f64;
        
        // Calculate adjustment factor
        let mut adjustment = target_time / average_time;
        
        // Clamp adjustment to prevent extreme changes
        adjustment = adjustment.clamp(MIN_ADJUSTMENT_FACTOR, MAX_ADJUSTMENT_FACTOR);
        
        // Calculate new difficulty
        let new_difficulty = (current as f64 * adjustment) as u64;
        
        // Ensure minimum difficulty
        new_difficulty.max(MIN_DIFFICULTY)
    }
}

impl Default for Difficulty {
    fn default() -> Self {
        Self::new(MIN_DIFFICULTY)
    }
}

/// Block time statistics
#[derive(Clone, Debug)]
pub struct BlockTimeStats {
    /// Average block time (seconds)
    pub average: f64,
    
    /// Median block time (seconds)
    pub median: f64,
    
    /// Standard deviation
    pub std_dev: f64,
    
    /// Min block time
    pub min: f64,
    
    /// Max block time
    pub max: f64,
}

impl BlockTimeStats {
    /// Calculate statistics from block times
    pub fn from_times(times: &[i64]) -> Option<Self> {
        if times.len() < 2 {
            return None;
        }
        
        // Calculate intervals
        let mut intervals: Vec<f64> = times
            .windows(2)
            .map(|w| (w[1] - w[0]) as f64)
            .collect();
        
        if intervals.is_empty() {
            return None;
        }
        
        // Calculate average
        let sum: f64 = intervals.iter().sum();
        let average = sum / intervals.len() as f64;
        
        // Calculate median
        intervals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = if intervals.len() % 2 == 0 {
            (intervals[intervals.len() / 2 - 1] + intervals[intervals.len() / 2]) / 2.0
        } else {
            intervals[intervals.len() / 2]
        };
        
        // Calculate std dev
        let variance: f64 = intervals
            .iter()
            .map(|x| (x - average).powi(2))
            .sum::<f64>()
            / intervals.len() as f64;
        let std_dev = variance.sqrt();
        
        Some(Self {
            average,
            median,
            std_dev,
            min: *intervals.first().unwrap(),
            max: *intervals.last().unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_difficulty_target() {
        let d = Difficulty::new(1000);
        let target = d.target();
        assert!(target > BigUint::zero());
    }
    
    #[test]
    fn test_difficulty_adjustment_faster() {
        // If blocks are coming too fast, difficulty should increase
        let current = 10000;
        let times: Vec<(u64, i64)> = (0..10)
            .map(|i| (i, (i * 15) as i64)) // 15 second blocks
            .collect();
        
        let new_diff = Difficulty::calculate_next(current, &times);
        assert!(new_diff > current);
    }
    
    #[test]
    fn test_difficulty_adjustment_slower() {
        // If blocks are coming too slow, difficulty should decrease
        let current = 10000;
        let times: Vec<(u64, i64)> = (0..10)
            .map(|i| (i, (i * 60) as i64)) // 60 second blocks
            .collect();
        
        let new_diff = Difficulty::calculate_next(current, &times);
        assert!(new_diff < current);
    }
    
    #[test]
    fn test_min_difficulty() {
        let d = Difficulty::new(0);
        assert_eq!(d.value(), MIN_DIFFICULTY);
    }
}

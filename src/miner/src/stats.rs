//! Mining statistics

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::Instant;

/// Mining statistics
pub struct MiningStats {
    /// Total hashes computed
    total_hashes: AtomicU64,
    
    /// Blocks found
    blocks_found: AtomicU64,
    
    /// Shares submitted (for pool mining)
    shares_submitted: AtomicU64,
    
    /// Shares accepted
    shares_accepted: AtomicU64,
    
    /// Current hashrate
    hashrate: RwLock<f64>,
    
    /// Average hashrate (1 min)
    avg_hashrate_1m: RwLock<f64>,
    
    /// Average hashrate (15 min)
    avg_hashrate_15m: RwLock<f64>,
    
    /// Start time
    start_time: Instant,
    
    /// Hashrate history (for averaging)
    hashrate_history: RwLock<Vec<(Instant, f64)>>,
}

impl MiningStats {
    /// Create new stats
    pub fn new() -> Self {
        Self {
            total_hashes: AtomicU64::new(0),
            blocks_found: AtomicU64::new(0),
            shares_submitted: AtomicU64::new(0),
            shares_accepted: AtomicU64::new(0),
            hashrate: RwLock::new(0.0),
            avg_hashrate_1m: RwLock::new(0.0),
            avg_hashrate_15m: RwLock::new(0.0),
            start_time: Instant::now(),
            hashrate_history: RwLock::new(Vec::new()),
        }
    }
    
    /// Add hashes to counter
    pub fn add_hashes(&self, count: u64) {
        self.total_hashes.fetch_add(count, Ordering::Relaxed);
    }
    
    /// Record block found
    pub fn record_block(&self) {
        self.blocks_found.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record share submitted
    pub fn record_share_submitted(&self) {
        self.shares_submitted.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record share accepted
    pub fn record_share_accepted(&self) {
        self.shares_accepted.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Update current hashrate
    pub fn update_hashrate(&self, rate: f64) {
        *self.hashrate.write().unwrap() = rate;
        
        let now = Instant::now();
        let mut history = self.hashrate_history.write().unwrap();
        history.push((now, rate));
        
        // Keep only last 15 minutes
        let cutoff = now - std::time::Duration::from_secs(15 * 60);
        history.retain(|(t, _)| *t > cutoff);
        
        // Calculate averages
        let one_min_ago = now - std::time::Duration::from_secs(60);
        let fifteen_min_ago = now - std::time::Duration::from_secs(15 * 60);
        
        let avg_1m: f64 = {
            let samples: Vec<f64> = history.iter()
                .filter(|(t, _)| *t > one_min_ago)
                .map(|(_, r)| *r)
                .collect();
            if samples.is_empty() { 0.0 } else { samples.iter().sum::<f64>() / samples.len() as f64 }
        };
        
        let avg_15m: f64 = {
            let samples: Vec<f64> = history.iter()
                .filter(|(t, _)| *t > fifteen_min_ago)
                .map(|(_, r)| *r)
                .collect();
            if samples.is_empty() { 0.0 } else { samples.iter().sum::<f64>() / samples.len() as f64 }
        };
        
        *self.avg_hashrate_1m.write().unwrap() = avg_1m;
        *self.avg_hashrate_15m.write().unwrap() = avg_15m;
    }
    
    /// Get current hashrate
    pub fn hashrate(&self) -> f64 {
        *self.hashrate.read().unwrap()
    }
    
    /// Get 1 minute average hashrate
    pub fn avg_hashrate_1m(&self) -> f64 {
        *self.avg_hashrate_1m.read().unwrap()
    }
    
    /// Get 15 minute average hashrate
    pub fn avg_hashrate_15m(&self) -> f64 {
        *self.avg_hashrate_15m.read().unwrap()
    }
    
    /// Get total hashes
    pub fn total_hashes(&self) -> u64 {
        self.total_hashes.load(Ordering::Relaxed)
    }
    
    /// Get blocks found
    pub fn blocks_found(&self) -> u64 {
        self.blocks_found.load(Ordering::Relaxed)
    }
    
    /// Get shares submitted
    pub fn shares_submitted(&self) -> u64 {
        self.shares_submitted.load(Ordering::Relaxed)
    }
    
    /// Get shares accepted
    pub fn shares_accepted(&self) -> u64 {
        self.shares_accepted.load(Ordering::Relaxed)
    }
    
    /// Get share acceptance rate
    pub fn acceptance_rate(&self) -> f64 {
        let submitted = self.shares_submitted.load(Ordering::Relaxed);
        let accepted = self.shares_accepted.load(Ordering::Relaxed);
        
        if submitted == 0 {
            100.0
        } else {
            (accepted as f64 / submitted as f64) * 100.0
        }
    }
    
    /// Get uptime in seconds
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
    
    /// Get formatted uptime string
    pub fn uptime_string(&self) -> String {
        let secs = self.uptime_secs();
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let secs = secs % 60;
        
        format!("{}h {}m {}s", hours, mins, secs)
    }
    
    /// Format hashrate for display
    pub fn format_hashrate(hashrate: f64) -> String {
        if hashrate >= 1_000_000_000.0 {
            format!("{:.2} GH/s", hashrate / 1_000_000_000.0)
        } else if hashrate >= 1_000_000.0 {
            format!("{:.2} MH/s", hashrate / 1_000_000.0)
        } else if hashrate >= 1_000.0 {
            format!("{:.2} KH/s", hashrate / 1_000.0)
        } else {
            format!("{:.2} H/s", hashrate)
        }
    }
    
    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "Hashrate: {} (1m: {}, 15m: {}) | Blocks: {} | Uptime: {}",
            Self::format_hashrate(self.hashrate()),
            Self::format_hashrate(self.avg_hashrate_1m()),
            Self::format_hashrate(self.avg_hashrate_15m()),
            self.blocks_found(),
            self.uptime_string(),
        )
    }
}

impl Default for MiningStats {
    fn default() -> Self {
        Self::new()
    }
}

/// GPU statistics
#[derive(Clone, Debug, Default)]
pub struct GpuStats {
    /// GPU index
    pub index: u32,
    
    /// GPU name
    pub name: String,
    
    /// Current hashrate
    pub hashrate: f64,
    
    /// Temperature (Celsius)
    pub temperature: Option<f32>,
    
    /// Fan speed (%)
    pub fan_speed: Option<u32>,
    
    /// Power usage (Watts)
    pub power: Option<f32>,
    
    /// Memory used (bytes)
    pub memory_used: Option<u64>,
    
    /// Memory total (bytes)
    pub memory_total: Option<u64>,
}

impl GpuStats {
    /// Format memory as string
    pub fn memory_string(&self) -> String {
        match (self.memory_used, self.memory_total) {
            (Some(used), Some(total)) => {
                format!("{:.1}/{:.1} GB", 
                    used as f64 / 1_000_000_000.0,
                    total as f64 / 1_000_000_000.0)
            }
            _ => "N/A".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stats_creation() {
        let stats = MiningStats::new();
        assert_eq!(stats.total_hashes(), 0);
        assert_eq!(stats.blocks_found(), 0);
    }
    
    #[test]
    fn test_hashrate_formatting() {
        assert_eq!(MiningStats::format_hashrate(500.0), "500.00 H/s");
        assert_eq!(MiningStats::format_hashrate(5000.0), "5.00 KH/s");
        assert_eq!(MiningStats::format_hashrate(5_000_000.0), "5.00 MH/s");
        assert_eq!(MiningStats::format_hashrate(5_000_000_000.0), "5.00 GH/s");
    }
    
    #[test]
    fn test_acceptance_rate() {
        let stats = MiningStats::new();
        assert_eq!(stats.acceptance_rate(), 100.0);
        
        stats.record_share_submitted();
        stats.record_share_submitted();
        stats.record_share_accepted();
        
        assert_eq!(stats.acceptance_rate(), 50.0);
    }
}

//! Miner configuration

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Default number of CPU threads (half of available)
pub fn default_cpu_threads() -> usize {
    (num_cpus::get() / 2).max(1)
}

/// Miner configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MinerConfig {
    /// Wallet address to receive mining rewards
    pub address: String,
    
    /// Node RPC endpoint
    #[serde(default = "default_node_url")]
    pub node_url: String,
    
    /// Number of CPU threads (0 = disable CPU mining)
    #[serde(default = "default_cpu_threads")]
    pub cpu_threads: usize,
    
    /// Enable GPU mining
    #[serde(default = "default_gpu_enabled")]
    pub gpu_enabled: bool,
    
    /// GPU device indices to use
    #[serde(default)]
    pub gpu_devices: Vec<u32>,
    
    /// GPU intensity (1-100)
    #[serde(default = "default_gpu_intensity")]
    pub gpu_intensity: u32,
    
    /// Worker name for pool mining
    #[serde(default = "default_worker_name")]
    pub worker_name: String,
    
    /// Enable stratum (pool) mining
    #[serde(default)]
    pub stratum_enabled: bool,
    
    /// Stratum pool URL
    #[serde(default)]
    pub stratum_url: Option<String>,
    
    /// Stratum password
    #[serde(default)]
    pub stratum_password: Option<String>,
    
    /// Log level
    #[serde(default = "default_log_level")]
    pub log_level: String,
    
    /// Statistics update interval (seconds)
    #[serde(default = "default_stats_interval")]
    pub stats_interval: u64,
}

fn default_node_url() -> String {
    "http://127.0.0.1:23421".to_string()
}

fn default_gpu_enabled() -> bool {
    true
}

fn default_gpu_intensity() -> u32 {
    80
}

fn default_worker_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "miner".to_string())
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_stats_interval() -> u64 {
    10
}

impl Default for MinerConfig {
    fn default() -> Self {
        Self {
            address: String::new(),
            node_url: default_node_url(),
            cpu_threads: default_cpu_threads(),
            gpu_enabled: default_gpu_enabled(),
            gpu_devices: Vec::new(),
            gpu_intensity: default_gpu_intensity(),
            worker_name: default_worker_name(),
            stratum_enabled: false,
            stratum_url: None,
            stratum_password: None,
            log_level: default_log_level(),
            stats_interval: default_stats_interval(),
        }
    }
}

impl MinerConfig {
    /// Load configuration from TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Save configuration to TOML file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Validate configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.address.is_empty() {
            anyhow::bail!("Wallet address is required");
        }
        
        if !self.address.starts_with("aeq1") {
            anyhow::bail!("Invalid wallet address format");
        }
        
        if self.cpu_threads == 0 && !self.gpu_enabled {
            anyhow::bail!("At least CPU or GPU mining must be enabled");
        }
        
        if self.gpu_intensity < 1 || self.gpu_intensity > 100 {
            anyhow::bail!("GPU intensity must be between 1 and 100");
        }
        
        if self.stratum_enabled && self.stratum_url.is_none() {
            anyhow::bail!("Stratum URL is required when stratum is enabled");
        }
        
        Ok(())
    }
    
    /// Create a sample configuration file
    pub fn create_sample<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
        let sample = r#"# Aequitas Miner Configuration
# ================================

# Your Aequitas wallet address (REQUIRED)
# Generate one with: aequitas-wallet new
address = "aeq1YourAddressHere"

# Node RPC endpoint (for solo mining)
node_url = "http://127.0.0.1:23421"

# Number of CPU threads (0 to disable CPU mining)
cpu_threads = 4

# Enable GPU mining
gpu_enabled = true

# GPU devices to use (empty = all available)
# Example: [0, 1] for first two GPUs
gpu_devices = []

# GPU mining intensity (1-100)
# Higher = more power, more heat
# Recommended: 70-80 for RTX 3060
gpu_intensity = 75

# Worker name (for pool statistics)
worker_name = "my-rig"

# Pool mining (stratum)
stratum_enabled = false
# stratum_url = "stratum+tcp://pool.example.com:3333"
# stratum_password = "x"

# Logging level: trace, debug, info, warn, error
log_level = "info"

# Statistics update interval (seconds)
stats_interval = 10
"#;
        
        std::fs::write(path, sample)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = MinerConfig::default();
        assert!(config.gpu_enabled);
        assert!(config.cpu_threads > 0);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = MinerConfig::default();
        assert!(config.validate().is_err()); // Missing address
        
        config.address = "aeq1TestAddress".to_string();
        assert!(config.validate().is_ok());
    }
}

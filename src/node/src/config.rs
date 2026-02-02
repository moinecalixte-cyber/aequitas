//! Node configuration

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Default data directory
pub fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aequitas")
}

/// Node configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Data directory for blockchain storage
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
    
    /// P2P listen address
    #[serde(default = "default_p2p_addr")]
    pub p2p_addr: String,
    
    /// RPC listen address
    #[serde(default = "default_rpc_addr")]
    pub rpc_addr: String,
    
    /// Enable RPC server
    #[serde(default = "default_rpc_enabled")]
    pub rpc_enabled: bool,
    
    /// Network (mainnet or testnet)
    #[serde(default = "default_network")]
    pub network: String,
    
    /// Bootstrap peers
    #[serde(default)]
    pub bootstrap_peers: Vec<String>,
    
    /// Enable mining
    #[serde(default)]
    pub mining_enabled: bool,
    
    /// Mining address (required if mining enabled)
    #[serde(default)]
    pub mining_address: Option<String>,
    
    /// Log level
    #[serde(default = "default_log_level")]
    pub log_level: String,
    
    /// Maximum peers
    #[serde(default = "default_max_peers")]
    pub max_peers: usize,
    
    /// Enable pruning (reduce storage)
    #[serde(default)]
    pub pruning: bool,
}

fn default_p2p_addr() -> String {
    "0.0.0.0:23420".to_string()
}

fn default_rpc_addr() -> String {
    "127.0.0.1:23421".to_string()
}

fn default_rpc_enabled() -> bool {
    true
}

fn default_network() -> String {
    "testnet".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_peers() -> usize {
    50
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            p2p_addr: default_p2p_addr(),
            rpc_addr: default_rpc_addr(),
            rpc_enabled: default_rpc_enabled(),
            network: default_network(),
            bootstrap_peers: Vec::new(),
            mining_enabled: false,
            mining_address: None,
            log_level: default_log_level(),
            max_peers: default_max_peers(),
            pruning: false,
        }
    }
}

impl NodeConfig {
    /// Load from TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Save to TOML file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Create sample config file
    pub fn create_sample<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
        let sample = r#"# Aequitas Node Configuration
# ============================

# Data directory for blockchain storage
# data_dir = "~/.aequitas"

# P2P network address
p2p_addr = "0.0.0.0:23420"

# RPC API address
rpc_addr = "127.0.0.1:23421"

# Enable RPC server
rpc_enabled = true

# Network: "mainnet" or "testnet"
network = "testnet"

# Bootstrap peers (leave empty for testnet discovery)
bootstrap_peers = []

# Enable built-in mining
mining_enabled = false

# Mining reward address (required if mining_enabled = true)
# mining_address = "aeq1YourAddress"

# Logging level: trace, debug, info, warn, error
log_level = "info"

# Maximum peer connections
max_peers = 50

# Enable blockchain pruning (saves disk space)
pruning = false
"#;
        
        std::fs::write(path, sample)?;
        Ok(())
    }
    
    /// Validate configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.mining_enabled && self.mining_address.is_none() {
            anyhow::bail!("mining_address required when mining_enabled is true");
        }
        
        if let Some(ref addr) = self.mining_address {
            if !addr.starts_with("aeq1") {
                anyhow::bail!("Invalid mining address format");
            }
        }
        
        Ok(())
    }
}

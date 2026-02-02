//! Blockchain storage (placeholder)

use std::path::Path;

/// Database storage for blockchain
pub struct Storage {
    // TODO: Implement RocksDB storage
    _path: std::path::PathBuf,
}

impl Storage {
    /// Open or create storage
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&path)?;
        
        Ok(Self { _path: path })
    }
}

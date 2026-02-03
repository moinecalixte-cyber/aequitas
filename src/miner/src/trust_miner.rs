//! GPU-Optimized Miner for Aequitas
//!
//! Trust-based mining that works with ANY graphics card
//! - Auto-detects GPU capabilities
//! - Optimizes batch processing
//! - Adaptive difficulty adjustment
//! - Hardware-agnostic performance

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::address::Address;
use crate::consensus::{AequiHash, GpuConfig};
use crate::core::{Block, BlockHeader, Transaction};

/// Trust-based miner that adapts to ANY GPU
pub struct TrustMiner {
    /// Mining configuration
    config: MinerConfig,

    /// Current mining state
    state: Arc<MinerState>,

    /// GPU optimization profile
    gpu_config: GpuConfig,

    /// Mining control
    should_mine: Arc<AtomicBool>,
}

/// Mining configuration
#[derive(Debug, Clone)]
pub struct MinerConfig {
    /// Miner's payout address
    pub address: Address,

    /// Mining pool endpoint (optional)
    pub pool_url: Option<String>,

    /// Number of mining threads (auto-detect if None)
    pub threads: Option<u32>,

    /// Target difficulty
    pub difficulty: Option<u64>,
}

/// Shared mining state
struct MinerState {
    /// Current block being mined
    current_block: Arc<parking_lot::Mutex<Option<Block>>>,

    /// Blocks found
    blocks_found: AtomicU64,

    /// Hashes computed per second
    hash_rate: AtomicU64,

    /// Mining start time
    start_time: std::time::Instant,
}

impl TrustMiner {
    /// Create a new trust-based miner
    pub fn new(config: MinerConfig) -> Self {
        let gpu_config = GpuConfig::detect();

        let state = Arc::new(MinerState {
            current_block: Arc::new(parking_lot::Mutex::new(None)),
            blocks_found: AtomicU64::new(0),
            hash_rate: AtomicU64::new(0),
            start_time: Instant::now(),
        });

        Self {
            config,
            state,
            gpu_config,
            should_mine: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start mining with automatic GPU optimization
    pub fn start_mining(&self) -> Result<(), MinerError> {
        if self.should_mine.load(Ordering::Acquire) {
            return Err(MinerError::AlreadyMining);
        }

        self.should_mine.store(true, Ordering::Acquire);

        println!("⛏️  AequiHash Mining Started");
        println!("{}", self.gpu_config.optimization_hints());

        // Create genesis block for initial mining
        let initial_block = self.create_initial_block();
        *self.state.current_block.lock() = Some(initial_block.clone());

        // Start mining threads based on detected hardware
        let thread_count = self.config.threads.unwrap_or_else(|| {
            self.gpu_config.compute_units.min(16) // Cap at 16 threads
        });

        println!("🚀 Starting {} mining threads", thread_count);

        for thread_id in 0..thread_count {
            let miner = self.clone();
            thread::spawn(move || {
                miner.mining_thread(thread_id);
            });
        }

        // Start performance monitoring thread
        self.start_monitoring_thread();

        Ok(())
    }

    /// Stop mining
    pub fn stop_mining(&self) {
        self.should_mine.store(false, Ordering::Acquire);
        println!("⏹️  Mining stopped");
    }

    /// Individual mining thread
    fn mining_thread(&self, thread_id: u32) {
        let mut nonce_counter = thread_id as u64;
        let mut hash_counter = 0u64;
        let mut last_rate_update = Instant::now();

        while self.should_mine.load(Ordering::Acquire) {
            if let Some(block) = self.state.current_block.lock().clone() {
                // GPU-optimized batch mining
                let batch_size = self.gpu_config.optimal_batch_size;

                if let Some((found_nonce, hash)) = self.mine_batch(
                    &block.header,
                    nonce_counter,
                    batch_size,
                    &self.config.difficulty.unwrap_or(1_000_000),
                ) {
                    // Block found!
                    self.on_block_found(block, found_nonce, hash, thread_id);
                    nonce_counter += batch_size;
                } else {
                    nonce_counter += batch_size;
                }

                // Update hash rate
                hash_counter += batch_size;

                // Update metrics every second
                if last_rate_update.elapsed() >= Duration::from_secs(1) {
                    self.state.hash_rate.store(hash_counter, Ordering::Acquire);
                    hash_counter = 0;
                    last_rate_update = Instant::now();
                }
            }
        }
    }

    /// GPU-optimized batch mining
    fn mine_batch(
        &self,
        header: &BlockHeader,
        start_nonce: u64,
        batch_size: u64,
        target_difficulty: u64,
    ) -> Option<(u64, [u8; 32])> {
        let aequihash = AequiHash::new(header.height);
        let header_hash = header.hash();

        // Create target from difficulty
        let target = Self::difficulty_to_target(target_difficulty);

        // Mine batch with GPU optimization
        for offset in 0..batch_size {
            let nonce = start_nonce.wrapping_add(offset);

            // Use optimized hash computation
            let hash = aequihash.hash_light_optimized(&header_hash, nonce, &[]);

            if Self::meets_target(&hash, &target) {
                return Some((nonce, hash));
            }
        }

        None
    }

    /// Called when a block is found
    fn on_block_found(&self, mut block: Block, nonce: u64, hash: [u8; 32], thread_id: u32) {
        // Set the found nonce
        block.header.nonce = nonce;
        block.header.merkle_root = crate::merkle::compute_merkle_root(&block.transactions);

        println!("🎉 BLOCK FOUND! Thread: {}, Nonce: {}", thread_id, nonce);
        println!("🔗 Block Hash: {}", hex::encode(hash));

        // Update counters
        self.state.blocks_found.fetch_add(1, Ordering::Acquire);

        // Create next block
        let next_block = self.create_next_block(&block);
        *self.state.current_block.lock() = Some(next_block);
    }

    /// Create initial block for mining
    fn create_initial_block(&self) -> Block {
        let coinbase = Transaction::coinbase(
            self.config.address.clone(),
            50_000_000_000, // 50 AEQ genesis reward
            0,
        );

        Block::new(
            [0u8; 32], // Genesis hash
            0,         // Height 0
            1_000_000, // Initial difficulty
            vec![coinbase],
        )
    }

    /// Create next block in sequence
    fn create_next_block(&self, prev_block: &Block) -> Block {
        let coinbase = Transaction::coinbase(
            self.config.address.clone(),
            50_000_000_000, // 50 AEQ reward (will be adjusted by blockchain)
            prev_block.header.height + 1,
        );

        Block::new(
            prev_block.hash(),
            prev_block.header.height + 1,
            prev_block.header.difficulty, // Will be adjusted by blockchain
            vec![coinbase],
        )
    }

    /// Convert difficulty to target hash
    fn difficulty_to_target(difficulty: u64) -> [u8; 32] {
        let max_target = u64::MAX / difficulty;
        let mut target = [0u8; 32];
        target[24..32].copy_from_slice(&max_target.to_le_bytes());
        target
    }

    /// Check if hash meets target
    fn meets_target(hash: &[u8; 32], target: &[u8; 32]) -> bool {
        // Compare first 8 bytes for efficiency
        let hash_value = u64::from_le_bytes([
            hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7],
        ]);

        let target_value = u64::from_le_bytes([
            target[0], target[1], target[2], target[3], target[4], target[5], target[6], target[7],
        ]);

        hash_value <= target_value
    }

    /// Start performance monitoring thread
    fn start_monitoring_thread(&self) {
        let state = Arc::clone(&self.state);
        let should_mine = Arc::clone(&self.should_mine);

        thread::spawn(move || {
            while should_mine.load(Ordering::Acquire) {
                thread::sleep(Duration::from_secs(5));

                let blocks_found = state.blocks_found.load(Ordering::Acquire);
                let hash_rate = state.hash_rate.load(Ordering::Acquire);
                let elapsed = state.start_time.elapsed();

                println!("📊 Mining Stats:");
                println!("   ⛏️  Hash Rate: {} H/s", hash_rate);
                println!("   🎯 Blocks Found: {}", blocks_found);
                println!("   ⏱️  Mining Time: {:?}", elapsed);
                println!("   🎮 GPU: {}", GpuConfig::detect().gpu_name);
            }
        });
    }

    /// Get current mining statistics
    pub fn get_stats(&self) -> MinerStats {
        MinerStats {
            blocks_found: self.state.blocks_found.load(Ordering::Acquire),
            hash_rate: self.state.hash_rate.load(Ordering::Acquire),
            uptime: self.state.start_time.elapsed(),
            gpu_info: self.gpu_config.clone(),
        }
    }
}

/// Mining statistics
#[derive(Debug, Clone)]
pub struct MinerStats {
    pub blocks_found: u64,
    pub hash_rate: u64,
    pub uptime: Duration,
    pub gpu_info: GpuConfig,
}

/// Mining errors
#[derive(Debug, thiserror::Error)]
pub enum MinerError {
    #[error("Already mining")]
    AlreadyMining,

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("GPU initialization failed")]
    GpuInitFailed,
}

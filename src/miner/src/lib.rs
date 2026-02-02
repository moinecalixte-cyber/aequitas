//! Aequitas Miner - GPU and CPU Mining
//!
//! Mining software for Aequitas cryptocurrency.

pub mod config;
pub mod worker;
pub mod stats;
pub mod stratum;

pub use config::MinerConfig;
pub use worker::MiningWorker;
pub use stats::MiningStats;

//! Aequitas Consensus - AequiHash Algorithm
//!
//! AequiHash is a GPU-friendly, ASIC-resistant proof-of-work algorithm
//! designed for fair mining on consumer GPUs like RTX 3060.

pub mod aequihash;
pub mod dag;
pub mod gpu_config;
pub mod pow;

pub use aequihash::AequiHash;
pub use dag::DAG;
pub use gpu_config::GpuConfig;
pub use pow::ProofOfWork;

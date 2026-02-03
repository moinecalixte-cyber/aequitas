//! Trust-based GPU Miner Entry Point
//!
//! Works with ANY graphics card through automatic optimization
//! - RTX 20xx/30xx/40xx series
//! - AMD RX 6000/7000 series  
//! - Intel Arc series
//! - Integrated graphics
//! - Legacy GPU support

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use anyhow::Result;
use clap::Parser;
use log::info;

use aequitas_core::{Address, Transaction};
mod trust_miner;

/// Command line arguments for miner
#[derive(Parser, Debug)]
#[command(author, version, about = "⛏️  AequiHash Trust Miner - Works with ANY GPU")]
struct Args {
    /// Mining address for rewards
    #[arg(long, short)]
    address: Address,
    
    /// Number of mining threads (auto-detect if not specified)
    #[arg(long, short = 't', default_value = "auto")]
    threads: String,
    
    /// Mining pool URL (optional)
    #[arg(long)]
    pool: Option<String>,
    
    /// Configuration file
    #[arg(long, short = 'c', default_value = "miner.toml")]
    config: String,
    
    /// Enable verbose logging
    #[arg(long, short)]
    verbose: bool,
    
    /// Show GPU detection info and exit
    #[arg(long)]
    gpu_info: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .init();
    
    // Show GPU info and exit if requested
    if args.gpu_info {
        show_gpu_info();
        return Ok(());
    }
    
    // Create miner configuration
    let threads = if args.threads == "auto" {
        None // Auto-detect
    } else {
        Some(args.threads.parse()?)
    };
    
    let miner_config = trust_miner::MinerConfig {
        address: args.address,
        pool: args.pool,
        threads,
        difficulty: None, // Will be set by blockchain
    };
    
    // Create and configure miner
    let miner = Arc::new(trust_miner::TrustMiner::new(miner_config));
    
    // Setup signal handlers for graceful shutdown
    let should_mine = Arc::new(AtomicBool::new(true));
    let should_mine_clone = Arc::clone(&should_mine);
    
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to setup Ctrl-C handler");
        info!("⏹️  Received interrupt signal");
        should_mine_clone.store(false, Ordering::Acquire);
    });
    
    // Start mining
    info!("🚀 Starting AequiHash mining with trust-based GPU optimization");
    miner.start_mining()?;
    
    // Monitor mining until interrupted
    let mut stats_interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
    
    loop {
        tokio::select! {
            _ = stats_interval.tick() => {
                if should_mine.load(Ordering::Acquire) {
                    let stats = miner.get_stats();
                    
                    println!("📊 Live Mining Stats:");
                    println!("   ⛏️  Hash Rate: {} H/s", stats.hash_rate);
                    println!("   🎯 Blocks Found: {}", stats.blocks_found);
                    println!("   ⏱️  Uptime: {:?}", stats.uptime);
                    println!("   🎮 GPU: {}", stats.gpu_info.gpu_name);
                    println!("   💾 VRAM: {}MB", stats.gpu_info.vram_mb);
                    println!();
                } else {
                    break;
                }
            }
        }
    }
    
    // Cleanup
    miner.stop_mining();
    info!("👋 Miner shutdown complete");
    
    Ok(())
}

/// Show GPU detection information
fn show_gpu_info() {
    println!("🎮 GPU Detection Results:");
    println!("{}", trust_miner::gpu_config::GpuConfig::detect().optimization_hints());
    println!();
    println!("✅ Trust-based optimization will work with ANY detected GPU!");
    println!("🚀 Auto-detection and optimization for:");
    println!("   • NVIDIA RTX series (20xx/30xx/40xx)");
    println!("   • AMD RX series (6000/7000)");
    println!("   • Intel Arc series");
    println!("   • Integrated Intel/AMD graphics");
    println!("   • Legacy GPU support");
}
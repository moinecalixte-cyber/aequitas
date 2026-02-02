//! Aequitas Miner - Main Entry Point
//!
//! Solo and pool mining for Aequitas cryptocurrency.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Duration;
use aequitas_miner::{MinerConfig, MiningWorker, MiningStats};
use aequitas_miner::worker::MiningJob;

#[derive(Parser)]
#[command(name = "aequitas-miner")]
#[command(author = "Aequitas Community")]
#[command(version = "0.1.0")]
#[command(about = "Aequitas GPU/CPU Miner - Fair mining for everyone")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Configuration file path
    #[arg(short, long, default_value = "miner.toml")]
    config: PathBuf,
    
    /// Wallet address (overrides config)
    #[arg(short, long)]
    address: Option<String>,
    
    /// Node URL (overrides config)
    #[arg(short, long)]
    node: Option<String>,
    
    /// Number of CPU threads
    #[arg(short, long)]
    threads: Option<usize>,
    
    /// Disable GPU mining
    #[arg(long)]
    no_gpu: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start mining
    Mine,
    
    /// Run benchmark
    Benchmark {
        /// Duration in seconds
        #[arg(short, long, default_value = "60")]
        duration: u64,
    },
    
    /// Generate sample config file
    Init {
        /// Output path
        #[arg(short, long, default_value = "miner.toml")]
        output: PathBuf,
    },
    
    /// Show hardware info
    Info,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Init { output }) => {
            init_config(&output)?;
        }
        Some(Commands::Benchmark { duration }) => {
            benchmark(duration)?;
        }
        Some(Commands::Info) => {
            show_info();
        }
        Some(Commands::Mine) | None => {
            mine(&cli).await?;
        }
    }
    
    Ok(())
}

/// Initialize configuration file
fn init_config(path: &PathBuf) -> anyhow::Result<()> {
    if path.exists() {
        anyhow::bail!("Config file already exists: {}", path.display());
    }
    
    MinerConfig::create_sample(path)?;
    println!("✓ Created sample config: {}", path.display());
    println!("\nEdit the file and set your wallet address, then run:");
    println!("  aequitas-miner mine");
    
    Ok(())
}

/// Run benchmark
fn benchmark(duration: u64) -> anyhow::Result<()> {
    println!("🔧 Running benchmark for {} seconds...\n", duration);
    
    let config = MinerConfig {
        address: "aeq1benchmark".to_string(),
        cpu_threads: num_cpus::get(),
        gpu_enabled: false, // CPU benchmark only for now
        ..Default::default()
    };
    
    let mut worker = MiningWorker::new(config);
    let _result_rx = worker.start()?;
    
    // Submit a test job
    let test_job = MiningJob::new([0u8; 32], 1000, 0);
    worker.submit_job(test_job)?;
    
    // Wait for benchmark duration
    std::thread::sleep(Duration::from_secs(duration));
    
    // Get results
    let hashrate = worker.hashrate();
    let total = worker.total_hashes();
    
    worker.stop();
    
    println!("\n📊 Benchmark Results:");
    println!("═══════════════════════════════════════");
    println!("  Duration:     {} seconds", duration);
    println!("  Total Hashes: {}", total);
    println!("  Hashrate:     {}", MiningStats::format_hashrate(hashrate));
    println!("  CPU Threads:  {}", num_cpus::get());
    println!("═══════════════════════════════════════\n");
    
    Ok(())
}

/// Show hardware info
fn show_info() {
    println!("\n💻 System Information:");
    println!("═══════════════════════════════════════");
    println!("  CPU Cores:    {}", num_cpus::get());
    println!("  Physical:     {}", num_cpus::get_physical());
    
    // Try to get system memory
    #[cfg(target_os = "windows")]
    {
        println!("  OS:           Windows");
    }
    
    #[cfg(target_os = "linux")]
    {
        println!("  OS:           Linux");
    }
    
    #[cfg(target_os = "macos")]
    {
        println!("  OS:           macOS");
    }
    
    // GPU info would require additional dependencies
    println!("\n🎮 GPU Detection:");
    println!("  Note: GPU support coming soon!");
    println!("  Recommended: NVIDIA RTX 3060 (6GB+)");
    println!("═══════════════════════════════════════\n");
}

/// Start mining
async fn mine(cli: &Cli) -> anyhow::Result<()> {
    // Load or create config
    let mut config = if cli.config.exists() {
        MinerConfig::load(&cli.config)?
    } else {
        println!("⚠️  No config file found. Using defaults.");
        println!("   Run 'aequitas-miner init' to create one.\n");
        MinerConfig::default()
    };
    
    // Apply CLI overrides
    if let Some(addr) = &cli.address {
        config.address = addr.clone();
    }
    if let Some(node) = &cli.node {
        config.node_url = node.clone();
    }
    if let Some(threads) = cli.threads {
        config.cpu_threads = threads;
    }
    if cli.no_gpu {
        config.gpu_enabled = false;
    }
    
    // Validate
    config.validate()?;
    
    // Print banner
    print_banner();
    
    println!("⚙️  Configuration:");
    println!("   Address:     {}", config.address);
    println!("   Node:        {}", config.node_url);
    println!("   CPU Threads: {}", config.cpu_threads);
    println!("   GPU Enabled: {}", config.gpu_enabled);
    println!();
    
    // Create worker
    let mut worker = MiningWorker::new(config.clone());
    let result_rx = worker.start()?;
    
    println!("⛏️  Mining started! Press Ctrl+C to stop.\n");
    
    // Setup signal handler
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        println!("\n\n🛑 Received shutdown signal...");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    });
    
    // Main mining loop
    let mut current_height = 0u64;
    
    while running.load(std::sync::atomic::Ordering::SeqCst) {
        // Get work from node
        match get_work_from_node(&config.node_url).await {
            Ok(job) => {
                if job.height > current_height {
                    current_height = job.height;
                    log::info!("New block template at height {}", job.height);
                    worker.submit_job(job)?;
                }
            }
            Err(e) => {
                log::warn!("Failed to get work: {}. Retrying...", e);
            }
        }
        
        // Check for solutions
        while let Ok(result) = result_rx.try_recv() {
            log::info!("🎉 Solution found! Nonce: {}", result.nonce);
            
            // Submit to node
            if let Err(e) = submit_solution(&config.node_url, &result).await {
                log::error!("Failed to submit solution: {}", e);
            } else {
                log::info!("✓ Solution accepted!");
            }
        }
        
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    // Cleanup
    worker.stop();
    
    println!("\n📊 Final Statistics:");
    println!("   Total Hashes: {}", worker.total_hashes());
    println!("   Uptime:       {}", worker.stats().uptime_string());
    println!("\nThank you for mining Aequitas! 🌟\n");
    
    Ok(())
}

/// Print startup banner
fn print_banner() {
    println!(r#"
    ___                _ __            
   /   | ___  ____ ___  _(_) /_____ ____
  / /| |/ _ \/ __ `/ / / / / __/ __ `/ ___/
 / ___ /  __/ /_/ / /_/ / / /_/ /_/ (__  ) 
/_/  |_\___/\__, /\__,_/_/\__/\__,_/____/  
              /_/                          
                                    
           ⛏️  AEQUITAS MINER v0.1.0
        Fair Mining for Everyone
    "#);
    println!();
}

/// Get work from node (placeholder - needs full RPC implementation)
async fn get_work_from_node(node_url: &str) -> anyhow::Result<MiningJob> {
    // TODO: Implement full RPC client
    // For now, create a test job
    
    let client = reqwest::Client::new();
    
    let response = client
        .post(format!("{}/getblocktemplate", node_url))
        .json(&serde_json::json!({}))
        .timeout(Duration::from_secs(5))
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            let template: serde_json::Value = resp.json().await?;
            
            let height = template["height"].as_u64().unwrap_or(1);
            let difficulty = template["difficulty"].as_u64().unwrap_or(1000);
            let header_hash = template["header_hash"].as_str().unwrap_or("");
            
            let mut hash = [0u8; 32];
            if !header_hash.is_empty() {
                hex::decode_to_slice(header_hash, &mut hash)?;
            } else {
                // Generate pseudo-random hash for testing
                use std::time::{SystemTime, UNIX_EPOCH};
                let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                hash[0..8].copy_from_slice(&time.to_le_bytes());
            }
            
            Ok(MiningJob::new(hash, difficulty, height))
        }
        Err(_) => {
            // Node not available, create test job
            use std::time::{SystemTime, UNIX_EPOCH};
            let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            
            let mut hash = [0u8; 32];
            hash[0..8].copy_from_slice(&time.to_le_bytes());
            
            Ok(MiningJob::new(hash, 10000, 1))
        }
    }
}

/// Submit solution to node
async fn submit_solution(node_url: &str, result: &aequitas_miner::worker::MiningResult) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    
    let _response = client
        .post(format!("{}/submitblock", node_url))
        .json(&serde_json::json!({
            "job_id": result.job_id,
            "nonce": result.nonce,
            "hash": hex::encode(result.hash),
        }))
        .timeout(Duration::from_secs(10))
        .send()
        .await?;
    
    Ok(())
}

//! Aequitas Node - Full Node Implementation
//!
//! Run a full Aequitas blockchain node.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use aequitas_node::{NodeConfig, Mempool};
use aequitas_node::rpc::{create_router, RpcState};
use aequitas_core::Blockchain;

#[derive(Parser)]
#[command(name = "aequitas-node")]
#[command(author = "Aequitas Community")]
#[command(version = "0.1.0")]
#[command(about = "Aequitas Full Node - Decentralized blockchain")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Configuration file path
    #[arg(short, long, default_value = "node.toml")]
    config: PathBuf,
    
    /// Data directory
    #[arg(short, long)]
    datadir: Option<PathBuf>,
    
    /// RPC address override
    #[arg(long)]
    rpc_addr: Option<String>,
    
    /// P2P address override
    #[arg(long)]
    p2p_addr: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the node
    Run,
    
    /// Initialize configuration
    Init {
        /// Output path
        #[arg(short, long, default_value = "node.toml")]
        output: PathBuf,
    },
    
    /// Show blockchain info
    Info,
    
    /// Show node status
    Status,
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
        Some(Commands::Info) => {
            show_info()?;
        }
        Some(Commands::Status) => {
            show_status(&cli.config).await?;
        }
        Some(Commands::Run) | None => {
            run_node(&cli).await?;
        }
    }
    
    Ok(())
}

/// Initialize configuration file
fn init_config(path: &PathBuf) -> anyhow::Result<()> {
    if path.exists() {
        anyhow::bail!("Config file already exists: {}", path.display());
    }
    
    NodeConfig::create_sample(path)?;
    println!("✓ Created config file: {}", path.display());
    println!("\nEdit the file, then run: aequitas-node run");
    
    Ok(())
}

/// Show blockchain info
fn show_info() -> anyhow::Result<()> {
    println!("\n📊 Aequitas Blockchain Info:\n");
    println!("═══════════════════════════════════════════════════════");
    println!("  Network:     Testnet");
    println!("  Algorithm:   AequiHash (GPU-friendly, ASIC-resistant)");
    println!("  Block Time:  30 seconds");
    println!("  Max Supply:  210,000,000 AEQ");
    println!("  Halving:     Every 2,100,000 blocks (~2 years)");
    println!("  Treasury:    2% of block rewards");
    println!("═══════════════════════════════════════════════════════\n");
    
    Ok(())
}

/// Show node status
async fn show_status(config_path: &PathBuf) -> anyhow::Result<()> {
    let config = if config_path.exists() {
        NodeConfig::load(config_path)?
    } else {
        NodeConfig::default()
    };
    
    println!("\n🔍 Node Status:\n");
    println!("═══════════════════════════════════════════════════════");
    println!("  RPC URL:    http://{}", config.rpc_addr);
    println!("  P2P Port:   {}", config.p2p_addr);
    println!("  Network:    {}", config.network);
    println!("═══════════════════════════════════════════════════════\n");
    
    // Try to connect to RPC
    let client = reqwest::Client::new();
    match client.get(format!("http://{}/info", config.rpc_addr))
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                let info: serde_json::Value = resp.json().await?;
                println!("  ✅ Node is running!");
                println!("  Height:     {}", info["height"]);
                println!("  Difficulty: {}", info["difficulty"]);
                println!("  Mempool:    {} txs", info["mempool_size"]);
            } else {
                println!("  ⚠️  Node responded with error");
            }
        }
        Err(_) => {
            println!("  ❌ Node is not running");
            println!("\n  Start with: aequitas-node run");
        }
    }
    
    println!();
    Ok(())
}

/// Run the node
async fn run_node(cli: &Cli) -> anyhow::Result<()> {
    // Load or create config
    let mut config = if cli.config.exists() {
        NodeConfig::load(&cli.config)?
    } else {
        log::info!("No config file found, using defaults");
        NodeConfig::default()
    };
    
    // Apply CLI overrides
    if let Some(ref datadir) = cli.datadir {
        config.data_dir = datadir.clone();
    }
    if let Some(ref rpc) = cli.rpc_addr {
        config.rpc_addr = rpc.clone();
    }
    if let Some(ref p2p) = cli.p2p_addr {
        config.p2p_addr = p2p.clone();
    }
    
    // Validate config
    config.validate()?;
    
    // Print banner
    print_banner();
    
    log::info!("Starting Aequitas Node...");
    log::info!("Network:  {}", config.network);
    log::info!("Data dir: {}", config.data_dir.display());
    log::info!("P2P:      {}", config.p2p_addr);
    log::info!("RPC:      {}", config.rpc_addr);
    
    // Create data directory
    std::fs::create_dir_all(&config.data_dir)?;
    
    // Initialize blockchain
    log::info!("Initializing blockchain...");
    let chain_path = config.data_dir.join("blockchain.dat");
    let blockchain = if chain_path.exists() {
        log::info!("Loading blockchain from {}...", chain_path.display());
        Arc::new(RwLock::new(Blockchain::load(&chain_path)?))
    } else {
        log::info!("Creating new blockchain...");
        let chain = Blockchain::new();
        let _ = chain.save(&chain_path);
        Arc::new(RwLock::new(chain))
    };
    let mempool = Arc::new(RwLock::new(Mempool::new()));
    
    {
        let chain = blockchain.read().await;
        log::info!("✓ Chain height:  {}", chain.height());
        log::info!("✓ Current tip:   {}", hex::encode(chain.tip()));
    }
    
    // Create broadcast channel for RPC -> P2P propagation
    let (p2p_broadcast_tx, mut p2p_broadcast_rx) = tokio::sync::mpsc::channel(100);

    // Start P2P network
    let p2p_config = aequitas_network::node::NodeConfig {
        listen_addr: config.p2p_addr.parse().unwrap_or_else(|_| "/ip4/0.0.0.0/tcp/23420".parse().unwrap()),
        bootstrap_peers: Vec::new(),
        testnet: config.network == "testnet",
        enable_mdns: true,
    };
    
    let mut p2p_node = aequitas_network::Node::new(p2p_config);
    let mut net_events = p2p_node.take_event_receiver().unwrap();
    let net_state = p2p_node.state.clone();

    // Start RPC server
    if config.rpc_enabled {
        let rpc_state = Arc::new(RpcState {
            blockchain: blockchain.clone(),
            mempool: mempool.clone(),
            broadcast_tx: p2p_broadcast_tx.clone(),
            chain_path: chain_path.clone(),
            net_state: net_state.clone(),
        });
        
        let router = create_router(rpc_state);
        let rpc_addr = config.rpc_addr.clone();
        
        tokio::spawn(async move {
            log::info!("Starting RPC server on http://{}", rpc_addr);
            let listener = tokio::net::TcpListener::bind(&rpc_addr).await.unwrap();
            axum::serve(listener, router).await.unwrap();
        });
    }
    
    let blockchain_p2p = blockchain.clone();
    let mempool_p2p = mempool.clone();
    
    tokio::spawn(async move {
        if let Err(e) = p2p_node.start(p2p_broadcast_rx).await {
            log::error!("P2P network error: {}", e);
        }
    });
    
    // Process network events
    let blockchain_ev = blockchain.clone();
    let mempool_ev = mempool.clone();
    let chain_path_ev = chain_path.clone();
    tokio::spawn(async move {
        while let Some(event) = net_events.recv().await {
            match event {
                aequitas_network::node::NetworkEvent::NewBlock(block) => {
                    log::info!("Received block {} via P2P", hex::encode(block.hash()));
                    let mut chain = blockchain_ev.write().await;
                    if let Err(e) = chain.add_block(block) {
                        log::warn!("Invalid block received: {}", e);
                    } else {
                        let _ = chain.save(&chain_path_ev);
                    }
                }
                aequitas_network::node::NetworkEvent::NewTransaction(tx) => {
                    log::info!("Received transaction {} via P2P", hex::encode(tx.hash()));
                    let mut pool = mempool_ev.write().await;
                    let _ = pool.add(tx, 0);
                }
                _ => {}
            }
        }
    });

    // Main loop
    log::info!("Node is running and public! Press Ctrl+C to stop.");
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    
    log::info!("Shutting down...");
    
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
                                    
           🌐 AEQUITAS NODE v0.1.0
       Decentralized • Fair • Resilient
    "#);
    println!();
}

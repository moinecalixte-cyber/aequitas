//! Aequitas Wallet CLI
//!
//! Command-line wallet for managing Aequitas addresses and transactions.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use aequitas_wallet::{Wallet, Keystore};
use aequitas_wallet::wallet::{format_balance, parse_balance};

#[derive(Parser)]
#[command(name = "aequitas-wallet")]
#[command(author = "Aequitas Community")]
#[command(version = "0.1.0")]
#[command(about = "Aequitas Wallet - Secure key management")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Wallet file path
    #[arg(short, long, default_value = "wallet.json")]
    wallet: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new wallet
    New {
        /// Password for encryption
        #[arg(short, long)]
        password: String,
        
        /// Optional label for the address
        #[arg(short, long)]
        label: Option<String>,
    },
    
    /// Generate a new address in existing wallet
    Generate {
        /// Password
        #[arg(short, long)]
        password: String,
        
        /// Optional label
        #[arg(short, long)]
        label: Option<String>,
    },
    
    /// List all addresses
    List,
    
    /// Show wallet info
    Info,
    
    /// Export private key (DANGEROUS!)
    Export {
        /// Address to export
        #[arg(short, long)]
        address: String,
        
        /// Password
        #[arg(short, long)]
        password: String,
    },
    
    /// Import private key
    Import {
        /// Private key in hex format
        #[arg(short, long)]
        key: String,
        
        /// Password for encryption
        #[arg(short, long)]
        password: String,
        
        /// Optional label
        #[arg(short, long)]
        label: Option<String>,
    },
    
    /// Show balance (requires node connection)
    Balance {
        /// Address to check (optional, shows all if not specified)
        #[arg(short, long)]
        address: Option<String>,
        
        /// Node RPC URL
        #[arg(short, long, default_value = "http://127.0.0.1:8080")]
        node: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::New { password, label } => {
            cmd_new(&cli.wallet, &password, label)?;
        }
        Commands::Generate { password, label } => {
            cmd_generate(&cli.wallet, &password, label)?;
        }
        Commands::List => {
            cmd_list(&cli.wallet)?;
        }
        Commands::Info => {
            cmd_info(&cli.wallet)?;
        }
        Commands::Export { address, password } => {
            cmd_export(&cli.wallet, &address, &password)?;
        }
        Commands::Import { key, password, label } => {
            cmd_import(&cli.wallet, &key, &password, label)?;
        }
        Commands::Balance { address, node } => {
            cmd_balance(&cli.wallet, address, &node).await?;
        }
    }
    
    Ok(())
}

fn cmd_new(path: &PathBuf, password: &str, label: Option<String>) -> anyhow::Result<()> {
    if path.exists() {
        anyhow::bail!("Wallet already exists: {}. Use 'generate' to add addresses.", path.display());
    }
    
    println!("\n🔐 Creating new Aequitas wallet...\n");
    
    let mut wallet = Wallet::new();
    let address = wallet.new_address(password, label.clone())?;
    wallet.save_to(path)?;
    
    println!("✅ Wallet created successfully!\n");
    println!("═══════════════════════════════════════════════════════");
    println!("  📍 Address: {}", address);
    if let Some(lbl) = label {
        println!("  🏷️  Label:   {}", lbl);
    }
    println!("  📁 File:    {}", path.display());
    println!("═══════════════════════════════════════════════════════\n");
    println!("⚠️  IMPORTANT: Keep your password safe! There is NO recovery!");
    println!("⚠️  Back up your wallet.json file in a secure location!\n");
    
    Ok(())
}

fn cmd_generate(path: &PathBuf, password: &str, label: Option<String>) -> anyhow::Result<()> {
    if !path.exists() {
        anyhow::bail!("Wallet not found: {}. Use 'new' to create one.", path.display());
    }
    
    let mut wallet = Wallet::load(path)?;
    let address = wallet.new_address(password, label.clone())?;
    wallet.save()?;
    
    println!("\n✅ New address generated!\n");
    println!("  📍 Address: {}", address);
    if let Some(lbl) = label {
        println!("  🏷️  Label:   {}", lbl);
    }
    println!();
    
    Ok(())
}

fn cmd_list(path: &PathBuf) -> anyhow::Result<()> {
    if !path.exists() {
        anyhow::bail!("Wallet not found: {}", path.display());
    }
    
    let wallet = Wallet::load(path)?;
    let addresses = wallet.addresses();
    
    println!("\n📋 Wallet Addresses ({}):\n", addresses.len());
    println!("═══════════════════════════════════════════════════════");
    
    for (i, addr) in addresses.iter().enumerate() {
        println!("  {}. {}", i + 1, addr);
    }
    
    println!("═══════════════════════════════════════════════════════\n");
    
    Ok(())
}

fn cmd_info(path: &PathBuf) -> anyhow::Result<()> {
    if !path.exists() {
        println!("\n❌ No wallet found at: {}", path.display());
        println!("\nCreate one with: aequitas-wallet new --password <PASSWORD>\n");
        return Ok(());
    }
    
    let wallet = Wallet::load(path)?;
    let addresses = wallet.addresses();
    
    println!("\n📊 Wallet Information:\n");
    println!("═══════════════════════════════════════════════════════");
    println!("  📁 File:      {}", path.display());
    println!("  🔢 Addresses: {}", addresses.len());
    if let Some(default) = wallet.default_address() {
        println!("  ⭐ Default:   {}", default);
    }
    println!("═══════════════════════════════════════════════════════\n");
    
    Ok(())
}

fn cmd_export(path: &PathBuf, address: &str, password: &str) -> anyhow::Result<()> {
    println!("\n⚠️  WARNING: Exporting private keys is dangerous!");
    println!("⚠️  Never share your private key with anyone!\n");
    
    let mut keystore = Keystore::load(path)?;
    let addr = aequitas_core::Address::from_string(address)?;
    
    keystore.unlock(&addr, password)?;
    
    if let Some(key) = keystore.get_signing_key(&addr) {
        let key_hex = hex::encode(key.to_bytes());
        println!("🔑 Private Key for {}:\n", address);
        println!("  {}\n", key_hex);
        println!("⚠️  Store this securely and NEVER share it!\n");
    } else {
        anyhow::bail!("Failed to export key");
    }
    
    Ok(())
}

fn cmd_import(path: &PathBuf, key: &str, password: &str, label: Option<String>) -> anyhow::Result<()> {
    let mut wallet = if path.exists() {
        Wallet::load(path)?
    } else {
        Wallet::new()
    };
    
    let address = wallet.import_private_key(key, password, label)?;
    wallet.save_to(path)?;
    
    println!("\n✅ Key imported successfully!\n");
    println!("  📍 Address: {}\n", address);
    
    Ok(())
}

async fn cmd_balance(path: &PathBuf, address: Option<String>, node: &str) -> anyhow::Result<()> {
    println!("\n💰 Checking balance...\n");
    println!("  Node: {}\n", node);
    
    let client = reqwest::Client::new();
    let price_eur = 0.12; // Mock price for demonstration

    let mut addresses = Vec::new();
    if let Some(addr) = address {
        addresses.push(addr);
    } else if path.exists() {
        let wallet = Wallet::load(path)?;
        for addr in wallet.addresses() {
            addresses.push(addr);
        }
    } else {
        anyhow::bail!("No address specified and no wallet.json found.");
    }

    println!("  {:<45} | {:<20} | {:<15}", "Address", "Balance (AEQ)", "Value (EUR)");
    println!("  {}", "─".repeat(85));

    for addr in addresses {
        let url = format!("{}/balance/{}", node, addr);
        match client.get(&url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    let data: serde_json::Value = resp.json().await?;
                    let balance_raw = data["balance"].as_u64().unwrap_or(0);
                    let balance_aeq = balance_raw as f64 / 1_000_000_000.0;
                    let value_eur = balance_aeq * price_eur;
                    
                    println!("  {:<45} | {:>20.9} | {:>12.2} €", addr, balance_aeq, value_eur);
                } else {
                    println!("  {:<45} | {:>20} | {:>15}", addr, "ERROR", "N/A");
                }
            }
            Err(_) => {
                println!("  {:<45} | {:>20} | {:>15}", addr, "OFFLINE", "N/A");
            }
        }
    }
    
    println!("\n  (Current estimated price: {:.2} €/AEQ)\n", price_eur);
    
    Ok(())
}

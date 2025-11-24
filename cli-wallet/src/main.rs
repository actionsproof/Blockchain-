use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

mod commands;
mod rpc_client;
mod wallet_storage;

use commands::{create_wallet, import_wallet, get_balance, send_transaction, deploy_contract};

#[derive(Parser)]
#[command(name = "act-wallet")]
#[command(about = "ACT Chain CLI Wallet - Manage your ACT tokens and interact with the blockchain", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// RPC endpoint URL (default: http://107.178.223.1:8545)
    #[arg(short, long, global = true, default_value = "http://107.178.223.1:8545")]
    rpc: String,

    /// Wallet file path (default: ~/.act-wallet/default.json)
    #[arg(short, long, global = true)]
    wallet: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new wallet
    Create {
        /// Wallet name (default: default)
        #[arg(short, long, default_value = "default")]
        name: String,
    },
    
    /// Import wallet from mnemonic
    Import {
        /// Wallet name
        #[arg(short, long, default_value = "default")]
        name: String,
    },
    
    /// Show wallet address and balance
    Balance {
        /// Show detailed account information
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Send ACT tokens
    Send {
        /// Recipient address (ACT-...)
        #[arg(short, long)]
        to: String,
        
        /// Amount in ACT
        #[arg(short, long)]
        amount: String,
        
        /// Gas limit (default: 21000)
        #[arg(short, long, default_value = "21000")]
        gas_limit: u64,
        
        /// Gas price in smallest units (default: 1000000000)
        #[arg(short = 'p', long, default_value = "1000000000")]
        gas_price: String,
    },
    
    /// Deploy a WASM contract
    Deploy {
        /// Path to WASM file
        #[arg(short, long)]
        wasm: PathBuf,
        
        /// Initial contract balance in ACT (default: 0)
        #[arg(short, long, default_value = "0")]
        value: String,
        
        /// Gas limit (default: 1000000)
        #[arg(short, long, default_value = "1000000")]
        gas_limit: u64,
        
        /// Gas price in smallest units (default: 1000000000)
        #[arg(short = 'p', long, default_value = "1000000000")]
        gas_price: String,
    },
    
    /// List all wallets
    List,
    
    /// Export wallet mnemonic (use with caution!)
    Export,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Determine wallet path
    let wallet_path = if let Some(path) = cli.wallet {
        path
    } else {
        get_default_wallet_path()?
    };
    
    match cli.command {
        Commands::Create { name } => {
            create_wallet(&name).await?;
        }
        
        Commands::Import { name } => {
            import_wallet(&name).await?;
        }
        
        Commands::Balance { detailed } => {
            get_balance(&cli.rpc, &wallet_path, detailed).await?;
        }
        
        Commands::Send { to, amount, gas_limit, gas_price } => {
            send_transaction(&cli.rpc, &wallet_path, &to, &amount, gas_limit, &gas_price).await?;
        }
        
        Commands::Deploy { wasm, value, gas_limit, gas_price } => {
            deploy_contract(&cli.rpc, &wallet_path, &wasm, &value, gas_limit, &gas_price).await?;
        }
        
        Commands::List => {
            list_wallets().await?;
        }
        
        Commands::Export => {
            export_mnemonic(&wallet_path).await?;
        }
    }
    
    Ok(())
}

fn get_default_wallet_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let wallet_dir = home.join(".act-wallet");
    std::fs::create_dir_all(&wallet_dir)?;
    Ok(wallet_dir.join("default.json"))
}

async fn list_wallets() -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let wallet_dir = home.join(".act-wallet");
    
    if !wallet_dir.exists() {
        println!("{}", "No wallets found. Create one with: act-wallet create".yellow());
        return Ok(());
    }
    
    println!("{}", "Available wallets:".bold().green());
    println!();
    
    for entry in std::fs::read_dir(&wallet_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                println!("  • {}", name.cyan());
            }
        }
    }
    
    Ok(())
}

async fn export_mnemonic(wallet_path: &PathBuf) -> Result<()> {
    use wallet_storage::WalletStorage;
    
    println!("{}", "⚠️  WARNING: Never share your mnemonic phrase!".red().bold());
    println!("{}", "Anyone with this phrase can access your funds.".red());
    println!();
    
    let password = rpassword::prompt_password("Enter wallet password: ")?;
    let storage = WalletStorage::load(wallet_path, &password)?;
    
    if let Some(mnemonic) = &storage.wallet.mnemonic {
        println!();
        println!("{}", "Your mnemonic phrase:".bold());
        println!();
        println!("  {}", mnemonic.bright_yellow().bold());
        println!();
    } else {
        println!("{}", "This is a watch-only wallet (no mnemonic)".yellow());
    }
    
    Ok(())
}

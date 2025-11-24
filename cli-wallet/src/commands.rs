use anyhow::{anyhow, Result};
use colored::Colorize;
use std::path::PathBuf;
use wallet::ActWallet;
use types::{ActAmount, Transaction, TransactionType};

use crate::rpc_client::RpcClient;
use crate::wallet_storage::WalletStorage;

/// Create a new wallet
pub async fn create_wallet(name: &str) -> Result<()> {
    println!("{}", "🔐 Creating new ACT wallet...".bold().cyan());
    println!();
    
    // Generate new wallet
    let wallet = ActWallet::new()?;
    
    // Get wallet path
    let wallet_path = get_wallet_path(name)?;
    
    // Check if wallet already exists
    if wallet_path.exists() {
        return Err(anyhow!("Wallet '{}' already exists. Use 'import' to restore from mnemonic.", name));
    }
    
    // Get password
    let password = rpassword::prompt_password("Enter password to encrypt wallet: ")?;
    let confirm = rpassword::prompt_password("Confirm password: ")?;
    
    if password != confirm {
        return Err(anyhow!("Passwords do not match"));
    }
    
    // Save wallet
    let storage = WalletStorage::new(wallet.clone(), true);
    storage.save(&wallet_path, &password)?;
    
    println!("{}", "✅ Wallet created successfully!".bold().green());
    println!();
    println!("{}", "Your wallet address:".bold());
    println!("  {}", wallet.address().to_string().bright_cyan().bold());
    println!();
    println!("{}", "Your recovery phrase (write this down!):".bold().yellow());
    println!();
    
    if let Some(mnemonic) = &wallet.mnemonic {
        println!("  {}", mnemonic.bright_yellow().bold());
    }
    
    println!();
    println!("{}", "⚠️  Keep your recovery phrase safe! It's the only way to restore your wallet.".red());
    println!();
    println!("Wallet saved to: {}", wallet_path.display().to_string().dimmed());
    
    Ok(())
}

/// Import wallet from mnemonic
pub async fn import_wallet(name: &str) -> Result<()> {
    println!("{}", "🔓 Importing ACT wallet from mnemonic...".bold().cyan());
    println!();
    
    // Get mnemonic
    println!("Enter your 12-word recovery phrase:");
    let mut mnemonic = String::new();
    std::io::stdin().read_line(&mut mnemonic)?;
    let mnemonic = mnemonic.trim();
    
    // Restore wallet
    let wallet = ActWallet::from_mnemonic(mnemonic)?;
    
    // Get wallet path
    let wallet_path = get_wallet_path(name)?;
    
    // Check if wallet already exists
    if wallet_path.exists() {
        println!("{}", "⚠️  Warning: Wallet already exists and will be overwritten.".yellow());
        let mut confirm = String::new();
        print!("Continue? (yes/no): ");
        use std::io::Write;
        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut confirm)?;
        if !confirm.trim().eq_ignore_ascii_case("yes") {
            println!("Import cancelled.");
            return Ok(());
        }
    }
    
    // Get password
    let password = rpassword::prompt_password("Enter password to encrypt wallet: ")?;
    let confirm = rpassword::prompt_password("Confirm password: ")?;
    
    if password != confirm {
        return Err(anyhow!("Passwords do not match"));
    }
    
    // Save wallet
    let storage = WalletStorage::new(wallet.clone(), true);
    storage.save(&wallet_path, &password)?;
    
    println!();
    println!("{}", "✅ Wallet imported successfully!".bold().green());
    println!();
    println!("{}", "Your wallet address:".bold());
    println!("  {}", wallet.address().to_string().bright_cyan().bold());
    println!();
    println!("Wallet saved to: {}", wallet_path.display().to_string().dimmed());
    
    Ok(())
}

/// Get account balance
pub async fn get_balance(rpc_url: &str, wallet_path: &PathBuf, detailed: bool) -> Result<()> {
    let password = rpassword::prompt_password("Enter wallet password: ")?;
    let storage = WalletStorage::load(wallet_path, &password)?;
    let wallet = &storage.wallet;
    
    let client = RpcClient::new(rpc_url.to_string());
    
    println!();
    println!("{}", "💰 Account Balance".bold().cyan());
    println!();
    println!("Address: {}", wallet.address().to_string().bright_cyan());
    
    // Get balance
    let balance = client.get_balance(&wallet.address().to_string()).await?;
    let balance_act = types::to_act(balance);
    
    println!("Balance: {} ACT", format!("{:.6}", balance_act).bright_green().bold());
    
    if detailed {
        // Get full account info
        let account = client.get_account(&wallet.address().to_string()).await?;
        println!();
        println!("{}", "Detailed Account Information:".bold());
        println!("  Nonce: {}", account.nonce);
        
        if let Some(code_hash) = account.code_hash {
            println!("  Contract Code: {}", code_hash.dimmed());
        }
        
        if let Some(storage_root) = account.storage_root {
            println!("  Storage Root: {}", storage_root.dimmed());
        }
    }
    
    println!();
    
    Ok(())
}

/// Send ACT tokens
pub async fn send_transaction(
    rpc_url: &str,
    wallet_path: &PathBuf,
    to: &str,
    amount_str: &str,
    gas_limit: u64,
    gas_price_str: &str,
) -> Result<()> {
    let password = rpassword::prompt_password("Enter wallet password: ")?;
    let storage = WalletStorage::load(wallet_path, &password)?;
    let wallet = &storage.wallet;
    
    let client = RpcClient::new(rpc_url.to_string());
    
    // Parse amount
    let amount_act: f64 = amount_str.parse()?;
    let amount = types::from_act(amount_act);
    
    // Parse gas price
    let gas_price: u128 = gas_price_str.parse()?;
    
    // Get nonce
    let nonce = client.get_nonce(&wallet.address().to_string()).await?;
    
    println!();
    println!("{}", "📤 Sending ACT Transaction".bold().cyan());
    println!();
    println!("From:      {}", wallet.address().to_string().cyan());
    println!("To:        {}", to.cyan());
    println!("Amount:    {} ACT", format!("{:.6}", amount_act).bright_green());
    println!("Gas Limit: {}", gas_limit);
    println!("Gas Price: {}", gas_price);
    println!("Nonce:     {}", nonce);
    println!();
    
    // Confirm
    let mut confirm = String::new();
    print!("Confirm transaction? (yes/no): ");
    use std::io::Write;
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut confirm)?;
    
    if !confirm.trim().eq_ignore_ascii_case("yes") {
        println!("Transaction cancelled.");
        return Ok(());
    }
    
    // Create and sign transaction
    let tx = wallet.create_transfer(to, amount, nonce, gas_limit, gas_price)?;
    
    // Send transaction
    println!();
    println!("{}", "⏳ Broadcasting transaction...".yellow());
    
    let tx_hash = client.send_transaction(&tx).await?;
    
    println!();
    println!("{}", "✅ Transaction sent successfully!".bold().green());
    println!();
    println!("Transaction hash: {}", tx_hash.bright_cyan());
    println!();
    
    Ok(())
}

/// Deploy a WASM contract
pub async fn deploy_contract(
    rpc_url: &str,
    wallet_path: &PathBuf,
    wasm_path: &PathBuf,
    value_str: &str,
    gas_limit: u64,
    gas_price_str: &str,
) -> Result<()> {
    let password = rpassword::prompt_password("Enter wallet password: ")?;
    let storage = WalletStorage::load(wallet_path, &password)?;
    let wallet = &storage.wallet;
    
    let client = RpcClient::new(rpc_url.to_string());
    
    // Read WASM file
    let wasm_code = std::fs::read(wasm_path)?;
    
    // Parse value
    let value_act: f64 = value_str.parse()?;
    let value = types::from_act(value_act);
    
    // Parse gas price
    let gas_price: u128 = gas_price_str.parse()?;
    
    // Get nonce
    let nonce = client.get_nonce(&wallet.address().to_string()).await?;
    
    println!();
    println!("{}", "📦 Deploying WASM Contract".bold().cyan());
    println!();
    println!("From:      {}", wallet.address().to_string().cyan());
    println!("WASM File: {}", wasm_path.display());
    println!("Size:      {} bytes", wasm_code.len());
    println!("Value:     {} ACT", format!("{:.6}", value_act).bright_green());
    println!("Gas Limit: {}", gas_limit);
    println!("Gas Price: {}", gas_price);
    println!("Nonce:     {}", nonce);
    println!();
    
    // Confirm
    let mut confirm = String::new();
    print!("Confirm deployment? (yes/no): ");
    use std::io::Write;
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut confirm)?;
    
    if !confirm.trim().eq_ignore_ascii_case("yes") {
        println!("Deployment cancelled.");
        return Ok(());
    }
    
    // Create transaction
    let tx_type = TransactionType::ContractDeploy {
        code: wasm_code,
        init_data: vec![], // Empty init data for now
    };
    
    let mut tx = Transaction {
        from: wallet.address().clone(),
        nonce,
        tx_type,
        gas_limit,
        gas_price,
        signature: vec![],
        pubkey: vec![],
    };
    
    // Sign transaction
    tx = wallet.sign_transaction(tx)?;
    
    // Send transaction
    println!();
    println!("{}", "⏳ Broadcasting contract deployment...".yellow());
    
    let tx_hash = client.send_transaction(&tx).await?;
    
    println!();
    println!("{}", "✅ Contract deployed successfully!".bold().green());
    println!();
    println!("Transaction hash: {}", tx_hash.bright_cyan());
    println!();
    println!("{}", "Note: Contract address will be derived from your address and nonce.".dimmed());
    
    Ok(())
}

fn get_wallet_path(name: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Cannot find home directory"))?;
    let wallet_dir = home.join(".act-wallet");
    std::fs::create_dir_all(&wallet_dir)?;
    Ok(wallet_dir.join(format!("{}.json", name)))
}

/// Example: Create and broadcast a transaction
/// 
/// Usage: cargo run --example send_transaction

use anyhow::Result;
use types::{Transaction, TransactionType};
use wallet::ActWallet;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔐 Creating ACT wallet...");
    
    // Create a new wallet
    let wallet = ActWallet::new()?;
    let address = wallet.get_address();
    
    println!("✅ Wallet created!");
    println!("   Address: {}", address);
    if let Some(mnemonic) = wallet.get_mnemonic() {
        println!("   Mnemonic: {}", mnemonic);
        println!("   ⚠️  SAVE THIS MNEMONIC SECURELY!");
    }
    
    println!("\n💸 Creating transfer transaction...");
    
    // Create a transaction to transfer 100 ACT
    let recipient = "ACT-treasury".to_string();
    let amount = 100_000_000_000_000_000_000u128; // 100 ACT (with 18 decimals)
    
    let tx = wallet.create_transfer_transaction(
        recipient.clone(),
        amount,
        21000,           // gas_limit
        1_000_000_000,   // gas_price (1 gwei)
        0,               // nonce
    )?;
    
    println!("✅ Transaction created!");
    println!("   From: {}", tx.from);
    println!("   To: {}", recipient);
    println!("   Amount: {} ACT", amount / 1_000_000_000_000_000_000);
    println!("   Gas Limit: {}", tx.gas_limit);
    println!("   Gas Price: {}", tx.gas_price);
    println!("   Nonce: {}", tx.nonce);
    println!("   Hash: {}", tx.hash());
    
    println!("\n📤 To broadcast this transaction:");
    println!("   1. Run a node: cargo run --release");
    println!("   2. Send transaction via RPC (Phase 4)");
    println!("   3. Or integrate with node's mempool directly");
    
    println!("\n💾 Transaction JSON:");
    println!("{}", serde_json::to_string_pretty(&tx)?);
    
    Ok(())
}

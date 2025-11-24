/// Example: Test RPC endpoints
/// 
/// Usage: cargo run --example rpc_client

use anyhow::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = "http://localhost:8545";
    
    println!("🔌 Connecting to ACT RPC at {}", rpc_url);
    
    // Test 1: Get balance
    println!("\n📊 Test 1: Get Balance");
    let request = json!({
        "jsonrpc": "2.0",
        "method": "act_getBalance",
        "params": {
            "address": "ACT-validator1"
        },
        "id": 1
    });
    
    let client = reqwest::Client::new();
    let response = client
        .post(rpc_url)
        .json(&request)
        .send()
        .await?;
    
    let result: serde_json::Value = response.json().await?;
    println!("Response: {}", serde_json::to_string_pretty(&result)?);
    
    // Test 2: Get Account Info
    println!("\n📋 Test 2: Get Account Info");
    let request = json!({
        "jsonrpc": "2.0",
        "method": "act_getAccount",
        "params": {
            "address": "ACT-treasury"
        },
        "id": 2
    });
    
    let response = client
        .post(rpc_url)
        .json(&request)
        .send()
        .await?;
    
    let result: serde_json::Value = response.json().await?;
    println!("Response: {}", serde_json::to_string_pretty(&result)?);
    
    // Test 3: Get Nonce
    println!("\n🔢 Test 3: Get Nonce");
    let request = json!({
        "jsonrpc": "2.0",
        "method": "act_getNonce",
        "params": {
            "address": "ACT-validator1"
        },
        "id": 3
    });
    
    let response = client
        .post(rpc_url)
        .json(&request)
        .send()
        .await?;
    
    let result: serde_json::Value = response.json().await?;
    println!("Response: {}", serde_json::to_string_pretty(&result)?);
    
    // Test 4: Get Mempool Status
    println!("\n💾 Test 4: Get Mempool Status");
    let request = json!({
        "jsonrpc": "2.0",
        "method": "act_getMempoolStatus",
        "params": {},
        "id": 4
    });
    
    let response = client
        .post(rpc_url)
        .json(&request)
        .send()
        .await?;
    
    let result: serde_json::Value = response.json().await?;
    println!("Response: {}", serde_json::to_string_pretty(&result)?);
    
    // Test 5: Health Check
    println!("\n💚 Test 5: Health Check");
    let response = client
        .get(&format!("{}/health", rpc_url))
        .send()
        .await?;
    
    let result: serde_json::Value = response.json().await?;
    println!("Response: {}", serde_json::to_string_pretty(&result)?);
    
    println!("\n✅ All RPC tests completed!");
    
    Ok(())
}

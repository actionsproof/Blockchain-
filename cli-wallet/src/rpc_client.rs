use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use types::ActAmount;

#[derive(Clone)]
pub struct RpcClient {
    client: Client,
    url: String,
    request_id: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[derive(Debug, Deserialize)]
pub struct AccountInfo {
    pub address: String,
    pub balance: ActAmount,
    pub nonce: u64,
    pub code_hash: Option<String>,
    pub storage_root: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MempoolStatus {
    pub total_transactions: usize,
    pub unique_senders: usize,
    pub avg_gas_price: ActAmount,
}

impl RpcClient {
    pub fn new(url: String) -> Self {
        Self {
            client: Client::new(),
            url,
            request_id: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
        }
    }
    
    fn next_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
    
    async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: self.next_id(),
        };
        
        let response = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("RPC request failed: {}", response.status()));
        }
        
        let rpc_response: JsonRpcResponse = response.json().await?;
        
        if let Some(error) = rpc_response.error {
            return Err(anyhow!("RPC error {}: {}", error.code, error.message));
        }
        
        rpc_response.result.ok_or_else(|| anyhow!("No result in RPC response"))
    }
    
    /// Get account balance
    pub async fn get_balance(&self, address: &str) -> Result<ActAmount> {
        let result = self.call("act_getBalance", json!([address])).await?;
        let balance: ActAmount = serde_json::from_value(result)?;
        Ok(balance)
    }
    
    /// Get full account information
    pub async fn get_account(&self, address: &str) -> Result<AccountInfo> {
        let result = self.call("act_getAccount", json!([address])).await?;
        let account: AccountInfo = serde_json::from_value(result)?;
        Ok(account)
    }
    
    /// Get account nonce
    pub async fn get_nonce(&self, address: &str) -> Result<u64> {
        let result = self.call("act_getNonce", json!([address])).await?;
        let nonce: u64 = serde_json::from_value(result)?;
        Ok(nonce)
    }
    
    /// Send a signed transaction
    pub async fn send_transaction(&self, tx: &types::Transaction) -> Result<String> {
        let result = self.call("act_sendTransaction", json!([tx])).await?;
        let tx_hash: String = serde_json::from_value(result)?;
        Ok(tx_hash)
    }
    
    /// Get transaction by hash
    pub async fn get_transaction(&self, hash: &str) -> Result<Option<types::Transaction>> {
        let result = self.call("act_getTransaction", json!([hash])).await?;
        
        if result.is_null() {
            return Ok(None);
        }
        
        let tx: types::Transaction = serde_json::from_value(result)?;
        Ok(Some(tx))
    }
    
    /// Get pending transactions
    pub async fn get_pending_transactions(&self) -> Result<Vec<types::Transaction>> {
        let result = self.call("act_getPendingTransactions", json!([])).await?;
        let txs: Vec<types::Transaction> = serde_json::from_value(result)?;
        Ok(txs)
    }
    
    /// Get mempool status
    pub async fn get_mempool_status(&self) -> Result<MempoolStatus> {
        let result = self.call("act_getMempoolStatus", json!([])).await?;
        let status: MempoolStatus = serde_json::from_value(result)?;
        Ok(status)
    }
}

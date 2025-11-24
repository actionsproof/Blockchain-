use anyhow::Result;
use axum::{
    extract::State as AxumState,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use mempool::Mempool;
use state::StateManager;
use types::{ActAmount, Transaction};

/// RPC Server state
#[derive(Clone)]
pub struct RpcState {
    pub state_manager: Arc<StateManager>,
    pub mempool: Arc<Mempool>,
}

/// JSON-RPC 2.0 Request
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

/// Balance query parameters
#[derive(Debug, Deserialize)]
pub struct GetBalanceParams {
    pub address: String,
}

/// Send transaction parameters
#[derive(Debug, Deserialize)]
pub struct SendTransactionParams {
    pub transaction: Transaction,
}

/// Account info response
#[derive(Debug, Serialize)]
pub struct AccountInfo {
    pub address: String,
    pub balance: ActAmount,
    pub nonce: u64,
}

/// Transaction receipt
#[derive(Debug, Serialize)]
pub struct TransactionReceipt {
    pub tx_hash: String,
    pub status: String,
}

/// Mempool status
#[derive(Debug, Serialize)]
pub struct MempoolStatus {
    pub pending_transactions: usize,
    pub unique_senders: usize,
    pub avg_gas_price: ActAmount,
}

impl RpcState {
    pub fn new(state_manager: Arc<StateManager>, mempool: Arc<Mempool>) -> Self {
        Self {
            state_manager,
            mempool,
        }
    }
}

/// Custom error type for RPC responses
struct RpcError(String);

impl IntoResponse for RpcError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32000,
                    message: self.0,
                }),
                id: serde_json::Value::Null,
            }),
        )
            .into_response()
    }
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "ACT Blockchain RPC",
        "version": "0.1.0"
    }))
}

/// Handle JSON-RPC requests
async fn handle_rpc(
    AxumState(state): AxumState<RpcState>,
    Json(request): Json<JsonRpcRequest>,
) -> Result<Json<JsonRpcResponse>, RpcError> {
    println!("📨 RPC request: {} (id: {})", request.method, request.id);

    let result = match request.method.as_str() {
        "act_getBalance" => {
            let params: GetBalanceParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let balance = state
                .state_manager
                .get_balance(&params.address)
                .map_err(|e| RpcError(format!("Failed to get balance: {}", e)))?;
            
            serde_json::to_value(balance)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "act_getAccount" => {
            let params: GetBalanceParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let balance = state
                .state_manager
                .get_balance(&params.address)
                .unwrap_or(0);
            
            let nonce = state
                .state_manager
                .get_nonce(&params.address)
                .unwrap_or(0);
            
            let account_info = AccountInfo {
                address: params.address,
                balance,
                nonce,
            };
            
            serde_json::to_value(account_info)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "act_getNonce" => {
            let params: GetBalanceParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let nonce = state
                .state_manager
                .get_nonce(&params.address)
                .map_err(|e| RpcError(format!("Failed to get nonce: {}", e)))?;
            
            serde_json::to_value(nonce)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "act_sendTransaction" => {
            let params: SendTransactionParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let tx_hash = state
                .mempool
                .add_transaction(params.transaction, &state.state_manager)
                .map_err(|e| RpcError(format!("Transaction rejected: {}", e)))?;
            
            println!("✅ Transaction accepted: {}...", &tx_hash[..16]);
            
            let receipt = TransactionReceipt {
                tx_hash,
                status: "pending".to_string(),
            };
            
            serde_json::to_value(receipt)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "act_getTransaction" => {
            let params: serde_json::Map<String, serde_json::Value> =
                serde_json::from_value(request.params)
                    .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let tx_hash = params
                .get("tx_hash")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError("Missing tx_hash parameter".to_string()))?;
            
            let tx = state.mempool.get_transaction(tx_hash);
            
            serde_json::to_value(tx)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "act_getPendingTransactions" => {
            let params: GetBalanceParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let txs = state.mempool.get_pending_transactions(&params.address);
            
            serde_json::to_value(txs)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "act_getMempoolStatus" => {
            let stats = state.mempool.get_stats();
            
            let status = MempoolStatus {
                pending_transactions: stats.total_transactions,
                unique_senders: stats.unique_senders,
                avg_gas_price: stats.avg_gas_price,
            };
            
            serde_json::to_value(status)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        _ => {
            return Err(RpcError(format!("Method not found: {}", request.method)));
        }
    };

    Ok(Json(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(result),
        error: None,
        id: request.id,
    }))
}

/// Start the RPC server
pub async fn start_rpc_server(state: RpcState, port: u16) -> Result<()> {
    let app = Router::new()
        .route("/", post(handle_rpc))
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("🌐 RPC server starting on http://{}", addr);
    println!("📡 Available methods:");
    println!("   - act_getBalance");
    println!("   - act_getAccount");
    println!("   - act_getNonce");
    println!("   - act_sendTransaction");
    println!("   - act_getTransaction");
    println!("   - act_getPendingTransactions");
    println!("   - act_getMempoolStatus");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_request_deserialization() {
        let json = r#"{
            "jsonrpc": "2.0",
            "method": "act_getBalance",
            "params": {"address": "ACT-test123"},
            "id": 1
        }"#;

        let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.method, "act_getBalance");
    }
}

use anyhow::Result;
use axum::{
    extract::{Path, State as AxumState},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

mod rpc_client;
use rpc_client::NodeRpcClient;

/// Explorer API state
#[derive(Clone)]
pub struct ExplorerState {
    rpc_client: Arc<NodeRpcClient>,
}

/// Block information response
#[derive(Debug, Serialize)]
pub struct BlockInfo {
    pub height: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub validator: String,
    pub transaction_count: usize,
    pub state_root: String,
    pub reward: u128,
}

/// Transaction information response
#[derive(Debug, Serialize)]
pub struct TransactionInfo {
    pub hash: String,
    pub from: String,
    pub tx_type: String,
    pub gas_limit: u64,
    pub gas_price: u128,
    pub nonce: u64,
    pub block_height: Option<u64>,
    pub status: String,
}

/// Account information response
#[derive(Debug, Serialize)]
pub struct AccountInfo {
    pub address: String,
    pub balance: u128,
    pub nonce: u64,
    pub is_contract: bool,
    pub code_hash: Option<String>,
}

/// Network statistics response
#[derive(Debug, Serialize)]
pub struct NetworkStats {
    pub latest_block: u64,
    pub total_transactions: usize,
    pub total_accounts: usize,
    pub pending_transactions: usize,
    pub avg_block_time: f64,
    pub total_supply: u128,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Starting ACT Chain Block Explorer...");
    
    // Get RPC endpoint from env or use default
    let rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "http://localhost:8545".to_string());
    
    println!("📡 Connecting to node: {}", rpc_url);
    
    let rpc_client = Arc::new(NodeRpcClient::new(rpc_url));
    let state = ExplorerState { rpc_client };
    
    // Build API router
    let app = Router::new()
        .route("/", get(index))
        .route("/api/blocks", get(get_latest_blocks))
        .route("/api/blocks/:height", get(get_block))
        .route("/api/transactions/:hash", get(get_transaction))
        .route("/api/accounts/:address", get(get_account))
        .route("/api/stats", get(get_stats))
        .route("/api/search/:query", get(search))
        .layer(CorsLayer::permissive())
        .with_state(state);
    
    let addr = "0.0.0.0:3001";
    println!("🌐 Block Explorer API running on http://{}", addr);
    println!("📊 Available endpoints:");
    println!("   GET /                          - Web UI");
    println!("   GET /api/blocks                - Latest blocks");
    println!("   GET /api/blocks/:height        - Block by height");
    println!("   GET /api/transactions/:hash    - Transaction by hash");
    println!("   GET /api/accounts/:address     - Account information");
    println!("   GET /api/stats                 - Network statistics");
    println!("   GET /api/search/:query         - Search blocks/txs/accounts");
    println!();
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Serve web UI
async fn index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

/// Get latest blocks
async fn get_latest_blocks(
    AxumState(state): AxumState<ExplorerState>,
) -> Result<Json<Vec<BlockInfo>>, AppError> {
    // For now, return mock data
    // TODO: Integrate with actual blockchain storage
    let blocks = vec![
        BlockInfo {
            height: 100,
            hash: "0x1234...".to_string(),
            parent_hash: "0xabcd...".to_string(),
            timestamp: 1700000000,
            validator: "ACT-validator1...".to_string(),
            transaction_count: 5,
            state_root: "0xstate...".to_string(),
            reward: 1000000000000000000, // 1 ACT
        },
    ];
    
    Ok(Json(blocks))
}

/// Get block by height
async fn get_block(
    AxumState(state): AxumState<ExplorerState>,
    Path(height): Path<u64>,
) -> Result<Json<BlockInfo>, AppError> {
    // TODO: Query actual block from storage
    let block = BlockInfo {
        height,
        hash: format!("0xblock{}", height),
        parent_hash: format!("0xblock{}", height - 1),
        timestamp: 1700000000 + (height * 30),
        validator: "ACT-validator1...".to_string(),
        transaction_count: 3,
        state_root: "0xstate...".to_string(),
        reward: 1000000000000000000,
    };
    
    Ok(Json(block))
}

/// Get transaction by hash
async fn get_transaction(
    AxumState(state): AxumState<ExplorerState>,
    Path(hash): Path<String>,
) -> Result<Json<TransactionInfo>, AppError> {
    // Try to get from RPC
    match state.rpc_client.get_transaction(&hash).await {
        Ok(Some(tx)) => {
            let tx_type = match tx.tx_type {
                types::TransactionType::Transfer { .. } => "Transfer",
                types::TransactionType::ContractDeploy { .. } => "ContractDeploy",
                types::TransactionType::ContractCall { .. } => "ContractCall",
            };
            
            Ok(Json(TransactionInfo {
                hash: hash.clone(),
                from: tx.from.to_string(),
                tx_type: tx_type.to_string(),
                gas_limit: tx.gas_limit,
                gas_price: tx.gas_price,
                nonce: tx.nonce,
                block_height: None, // TODO: Track block height
                status: "Pending".to_string(),
            }))
        }
        Ok(None) => Err(AppError::NotFound("Transaction not found".to_string())),
        Err(e) => Err(AppError::Internal(e.to_string())),
    }
}

/// Get account information
async fn get_account(
    AxumState(state): AxumState<ExplorerState>,
    Path(address): Path<String>,
) -> Result<Json<AccountInfo>, AppError> {
    // Query balance and nonce from RPC
    match state.rpc_client.get_account(&address).await {
        Ok(account) => Ok(Json(AccountInfo {
            address: account.address,
            balance: account.balance,
            nonce: account.nonce,
            is_contract: account.code_hash.is_some(),
            code_hash: account.code_hash,
        })),
        Err(e) => Err(AppError::Internal(e.to_string())),
    }
}

/// Get network statistics
async fn get_stats(
    AxumState(state): AxumState<ExplorerState>,
) -> Result<Json<NetworkStats>, AppError> {
    // Get mempool status
    let mempool = state.rpc_client.get_mempool_status().await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    Ok(Json(NetworkStats {
        latest_block: 100, // TODO: Get from node
        total_transactions: 1500,
        total_accounts: 50,
        pending_transactions: mempool.total_transactions,
        avg_block_time: 30.0,
        total_supply: 13000000000000000000000000, // 13M ACT
    }))
}

/// Search for blocks, transactions, or accounts
async fn search(
    AxumState(state): AxumState<ExplorerState>,
    Path(query): Path<String>,
) -> Result<Json<SearchResult>, AppError> {
    // Determine what type of search this is
    if query.starts_with("ACT-") {
        // Search for account
        match state.rpc_client.get_account(&query).await {
            Ok(account) => Ok(Json(SearchResult::Account(AccountInfo {
                address: account.address,
                balance: account.balance,
                nonce: account.nonce,
                is_contract: account.code_hash.is_some(),
                code_hash: account.code_hash,
            }))),
            Err(_) => Err(AppError::NotFound("Account not found".to_string())),
        }
    } else if query.starts_with("0x") {
        // Search for transaction hash
        match state.rpc_client.get_transaction(&query).await {
            Ok(Some(tx)) => {
                let tx_type = match tx.tx_type {
                    types::TransactionType::Transfer { .. } => "Transfer",
                    types::TransactionType::ContractDeploy { .. } => "ContractDeploy",
                    types::TransactionType::ContractCall { .. } => "ContractCall",
                };
                
                Ok(Json(SearchResult::Transaction(TransactionInfo {
                    hash: query.clone(),
                    from: tx.from.to_string(),
                    tx_type: tx_type.to_string(),
                    gas_limit: tx.gas_limit,
                    gas_price: tx.gas_price,
                    nonce: tx.nonce,
                    block_height: None,
                    status: "Pending".to_string(),
                })))
            }
            Ok(None) => Err(AppError::NotFound("Transaction not found".to_string())),
            Err(e) => Err(AppError::Internal(e.to_string())),
        }
    } else if let Ok(height) = query.parse::<u64>() {
        // Search for block by height
        let block = BlockInfo {
            height,
            hash: format!("0xblock{}", height),
            parent_hash: format!("0xblock{}", height.saturating_sub(1)),
            timestamp: 1700000000 + (height * 30),
            validator: "ACT-validator1...".to_string(),
            transaction_count: 3,
            state_root: "0xstate...".to_string(),
            reward: 1000000000000000000,
        };
        Ok(Json(SearchResult::Block(block)))
    } else {
        Err(AppError::BadRequest("Invalid search query".to_string()))
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum SearchResult {
    Block(BlockInfo),
    Transaction(TransactionInfo),
    Account(AccountInfo),
}

/// Error handling
enum AppError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

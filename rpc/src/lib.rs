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
use staking::StakingManager;
use governance::GovernanceManager;
use types::{ActAmount, EventLog, Transaction};

/// RPC Server state
#[derive(Clone)]
pub struct RpcState {
    pub state_manager: Arc<StateManager>,
    pub mempool: Arc<Mempool>,
    pub staking_manager: Arc<tokio::sync::Mutex<StakingManager>>,
    pub governance_manager: Arc<tokio::sync::Mutex<GovernanceManager>>,
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

/// Staking deposit parameters
#[derive(Debug, Deserialize)]
pub struct StakeDepositParams {
    pub address: String,
    pub amount: u64,
    pub commission_rate: u8,
}

/// Delegate parameters
#[derive(Debug, Deserialize)]
pub struct DelegateParams {
    pub delegator_address: String,
    pub validator_address: String,
    pub amount: u64,
}

/// Unstake parameters
#[derive(Debug, Deserialize)]
pub struct UnstakeParams {
    pub address: String,
    pub amount: u64,
}

/// Undelegate parameters
#[derive(Debug, Deserialize)]
pub struct UndelegateParams {
    pub delegator_address: String,
    pub validator_address: String,
    pub amount: u64,
}

/// Claim parameters
#[derive(Debug, Deserialize)]
pub struct ClaimParams {
    pub address: String,
}

/// Get validator parameters
#[derive(Debug, Deserialize)]
pub struct GetValidatorParams {
    pub address: String,
}

/// Get validators parameters
#[derive(Debug, Deserialize)]
pub struct GetValidatorsParams {
    pub active_only: bool,
}

/// Governance propose parameters
#[derive(Debug, Deserialize)]
pub struct ProposeParams {
    pub proposer: String,
    pub proposal_type: serde_json::Value,
    pub title: String,
    pub description: String,
}

/// Governance vote parameters
#[derive(Debug, Deserialize)]
pub struct VoteParams {
    pub proposal_id: u64,
    pub voter: String,
    pub vote_option: String,
}

/// Get proposal parameters
#[derive(Debug, Deserialize)]
pub struct GetProposalParams {
    pub proposal_id: u64,
}

/// List proposals parameters
#[derive(Debug, Deserialize)]
pub struct ListProposalsParams {
    pub status: Option<String>,
}

/// Get vote parameters
#[derive(Debug, Deserialize)]
pub struct GetVoteParams {
    pub proposal_id: u64,
    pub voter: String,
}

/// Get voting power parameters
#[derive(Debug, Deserialize)]
pub struct GetVotingPowerParams {
    pub address: String,
    pub snapshot_height: Option<u64>,
}

/// Get tally result parameters
#[derive(Debug, Deserialize)]
pub struct GetTallyParams {
    pub proposal_id: u64,
}

/// Get logs parameters
#[derive(Debug, Deserialize)]
pub struct GetLogsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<String>>,
    pub from_block: u64,
    pub to_block: u64,
}

/// Get receipt parameters
#[derive(Debug, Deserialize)]
pub struct GetReceiptParams {
    pub tx_hash: String,
}

impl RpcState {
    pub fn new(state_manager: Arc<StateManager>, mempool: Arc<Mempool>, staking_manager: Arc<tokio::sync::Mutex<StakingManager>>, governance_manager: Arc<tokio::sync::Mutex<GovernanceManager>>) -> Self {
        Self {
            state_manager,
            mempool,
            staking_manager,
            governance_manager,
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

        "act_getLogs" => {
            let params: GetLogsParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let logs = state
                .state_manager
                .query_logs(
                    params.address.as_deref(),
                    params.topics,
                    params.from_block,
                    params.to_block,
                )
                .map_err(|e| RpcError(format!("Failed to query logs: {}", e)))?;
            
            println!("📜 Queried {} event logs", logs.len());
            
            serde_json::to_value(logs)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "act_getTransactionReceipt" => {
            let params: GetReceiptParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let receipt = state
                .state_manager
                .get_receipt(&params.tx_hash)
                .map_err(|e| RpcError(format!("Failed to get receipt: {}", e)))?;
            
            serde_json::to_value(receipt)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        // Ethereum-compatible RPC methods
        "eth_blockNumber" => {
            // Return latest block height in hex
            let height = 0u64; // TODO: Get from storage
            serde_json::to_value(format!("0x{:x}", height))
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "eth_getBalance" => {
            let params: serde_json::Value = request.params;
            let address = params
                .get(0)
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError("Missing address parameter".to_string()))?;
            
            // Convert Ethereum address to ACT address or query directly
            let balance = state
                .state_manager
                .get_balance(address)
                .unwrap_or(0);
            
            // Return balance in hex
            serde_json::to_value(format!("0x{:x}", balance))
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "eth_getTransactionCount" => {
            let params: serde_json::Value = request.params;
            let address = params
                .get(0)
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError("Missing address parameter".to_string()))?;
            
            let nonce = state
                .state_manager
                .get_nonce(address)
                .unwrap_or(0);
            
            serde_json::to_value(format!("0x{:x}", nonce))
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "eth_sendRawTransaction" => {
            // TODO: Decode RLP-encoded Ethereum transaction
            serde_json::to_value("0x0000000000000000000000000000000000000000000000000000000000000000")
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "eth_call" => {
            // TODO: Execute read-only contract call
            serde_json::to_value("0x")
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "eth_chainId" => {
            // Return ACT Chain ID (e.g., 0xACT = 2755)
            serde_json::to_value("0xac7")
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "net_version" => {
            // Network version (same as chain ID)
            serde_json::to_value("2755")
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        // Staking methods
        "stake_deposit" => {
            let params: StakeDepositParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let mut staking = state.staking_manager.lock().await;
            let validator_address = staking
                .stake(params.address, params.amount, params.commission_rate)
                .map_err(|e| RpcError(format!("Stake failed: {}", e)))?;
            
            serde_json::to_value(validator_address)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "stake_delegate" => {
            let params: DelegateParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let mut staking = state.staking_manager.lock().await;
            staking
                .delegate(params.delegator_address, params.validator_address, params.amount)
                .map_err(|e| RpcError(format!("Delegation failed: {}", e)))?;
            
            serde_json::to_value("Delegation successful")
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "stake_unstake" => {
            let params: UnstakeParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let mut staking = state.staking_manager.lock().await;
            let request_id = staking
                .unstake(params.address, params.amount)
                .map_err(|e| RpcError(format!("Unstake failed: {}", e)))?;
            
            serde_json::to_value(request_id)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "stake_undelegate" => {
            let params: UndelegateParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let mut staking = state.staking_manager.lock().await;
            let request_id = staking
                .undelegate(params.delegator_address, params.validator_address, params.amount)
                .map_err(|e| RpcError(format!("Undelegation failed: {}", e)))?;
            
            serde_json::to_value(request_id)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "stake_claimUnstaked" => {
            let params: ClaimParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let mut staking = state.staking_manager.lock().await;
            let amount = staking
                .claim_unstaked(params.address)
                .map_err(|e| RpcError(format!("Claim failed: {}", e)))?;
            
            serde_json::to_value(amount)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "stake_claimRewards" => {
            let params: ClaimParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let mut staking = state.staking_manager.lock().await;
            let rewards = staking
                .claim_rewards(params.address)
                .map_err(|e| RpcError(format!("Claim rewards failed: {}", e)))?;
            
            serde_json::to_value(rewards)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "stake_getValidator" => {
            let params: GetValidatorParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let staking = state.staking_manager.lock().await;
            let validator = staking.get_validator(&params.address);
            
            serde_json::to_value(validator)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "stake_getValidators" => {
            let params: GetValidatorsParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let staking = state.staking_manager.lock().await;
            let validators = if params.active_only {
                staking.get_active_validators()
            } else {
                staking.get_all_validators()
            };
            
            serde_json::to_value(validators)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "stake_getDelegations" => {
            let params: ClaimParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let staking = state.staking_manager.lock().await;
            let delegations = staking.get_delegations(&params.address);
            
            serde_json::to_value(delegations)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "stake_getUnstakeRequests" => {
            let params: ClaimParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let staking = state.staking_manager.lock().await;
            let requests = staking.get_unstake_requests(&params.address);
            
            serde_json::to_value(requests)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "stake_getRewards" => {
            let params: ClaimParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let staking = state.staking_manager.lock().await;
            let rewards = staking.get_unclaimed_rewards(&params.address);
            
            serde_json::to_value(rewards)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        // Governance methods
        "gov_propose" => {
            let params: ProposeParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let proposal_type: governance::ProposalType = serde_json::from_value(params.proposal_type)
                .map_err(|e| RpcError(format!("Invalid proposal type: {}", e)))?;
            
            let proposer_balance = state
                .state_manager
                .get_balance(&params.proposer)
                .unwrap_or(0);
            
            // TODO: Get actual total supply from state
            let total_supply = 13_000_000_000_000_000; // Placeholder
            
            let mut governance = state.governance_manager.lock().await;
            let proposal_id = governance
                .create_proposal(
                    params.proposer,
                    proposal_type,
                    params.title,
                    params.description,
                    proposer_balance.try_into().unwrap_or(u64::MAX),
                    total_supply,
                )
                .map_err(|e| RpcError(format!("Proposal creation failed: {}", e)))?;;
            
            serde_json::to_value(proposal_id)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "gov_vote" => {
            let params: VoteParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let vote_option = match params.vote_option.as_str() {
                "Yes" => governance::VoteOption::Yes,
                "No" => governance::VoteOption::No,
                "Abstain" => governance::VoteOption::Abstain,
                _ => return Err(RpcError("Invalid vote option. Use 'Yes', 'No', or 'Abstain'".to_string())),
            };
            
            // Calculate vote power (balance + staking)
            let balance = state
                .state_manager
                .get_balance(&params.voter)
                .unwrap_or(0);
            
            let staking = state.staking_manager.lock().await;
            let validator_stake = staking
                .get_validator(&params.voter)
                .map(|v| v.stake + v.delegated_stake)
                .unwrap_or(0);
            
            let delegator_stake: u64 = staking
                .get_delegations(&params.voter)
                .iter()
                .map(|d| d.amount)
                .sum();
            
            let vote_power = balance + (validator_stake as u128) + (delegator_stake as u128);
            drop(staking);
            
            let mut governance = state.governance_manager.lock().await;
            governance
                .cast_vote(params.proposal_id, params.voter, vote_option, vote_power.try_into().unwrap_or(u64::MAX))
                .map_err(|e| RpcError(format!("Vote failed: {}", e)))?;;
            
            serde_json::to_value("Vote cast successfully")
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "gov_getProposal" => {
            let params: GetProposalParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let governance = state.governance_manager.lock().await;
            let proposal = governance.get_proposal(params.proposal_id);
            
            serde_json::to_value(proposal)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "gov_listProposals" => {
            let params: ListProposalsParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let status_filter = params.status.and_then(|s| {
                match s.as_str() {
                    "Review" => Some(governance::ProposalStatus::Review),
                    "Active" => Some(governance::ProposalStatus::Active),
                    "Passed" => Some(governance::ProposalStatus::Passed),
                    "Rejected" => Some(governance::ProposalStatus::Rejected),
                    "Expired" => Some(governance::ProposalStatus::Expired),
                    "Executed" => Some(governance::ProposalStatus::Executed),
                    "Failed" => Some(governance::ProposalStatus::Failed),
                    "Vetoed" => Some(governance::ProposalStatus::Vetoed),
                    _ => None,
                }
            });
            
            let governance = state.governance_manager.lock().await;
            let proposals = governance.list_proposals(status_filter);
            
            serde_json::to_value(proposals)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "gov_getVote" => {
            let params: GetVoteParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let governance = state.governance_manager.lock().await;
            let vote = governance.get_vote(params.proposal_id, &params.voter);
            
            serde_json::to_value(vote)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "gov_getVotingPower" => {
            let params: GetVotingPowerParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            // Calculate voting power
            let balance = state
                .state_manager
                .get_balance(&params.address)
                .unwrap_or(0);
            
            let staking = state.staking_manager.lock().await;
            let validator_stake = staking
                .get_validator(&params.address)
                .map(|v| v.stake + v.delegated_stake)
                .unwrap_or(0);
            
            let delegator_stake: u64 = staking
                .get_delegations(&params.address)
                .iter()
                .map(|d| d.amount)
                .sum();
            
            let total_power = balance + (validator_stake as u128) + (delegator_stake as u128);
            
            serde_json::to_value(total_power)
                .map_err(|e| RpcError(format!("Serialization error: {}", e)))?
        }

        "gov_getTallyResult" => {
            let params: GetTallyParams = serde_json::from_value(request.params)
                .map_err(|e| RpcError(format!("Invalid params: {}", e)))?;
            
            let governance = state.governance_manager.lock().await;
            let tally = governance
                .get_tally_result(params.proposal_id)
                .map_err(|e| RpcError(format!("Failed to get tally: {}", e)))?;
            
            serde_json::to_value(tally)
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
    println!("   ACT Native:");
    println!("   - act_getBalance");
    println!("   - act_getAccount");
    println!("   - act_getNonce");
    println!("   - act_sendTransaction");
    println!("   - act_getTransaction");
    println!("   - act_getPendingTransactions");
    println!("   - act_getMempoolStatus");
    println!("   - act_getLogs");
    println!("   - act_getTransactionReceipt");
    println!("   Ethereum Compatible:");
    println!("   - eth_blockNumber");
    println!("   - eth_getBalance");
    println!("   - eth_getTransactionCount");
    println!("   - eth_sendRawTransaction");
    println!("   - eth_call");
    println!("   - eth_chainId");
    println!("   - net_version");
    println!();
    println!("   Staking:");
    println!("   - stake_deposit");
    println!("   - stake_delegate");
    println!("   - stake_unstake");
    println!("   - stake_undelegate");
    println!("   - stake_claimUnstaked");
    println!("   - stake_claimRewards");
    println!("   - stake_getValidator");
    println!("   - stake_getValidators");
    println!("   - stake_getDelegations");
    println!("   - stake_getUnstakeRequests");
    println!("   - stake_getRewards");
    println!();
    println!("   Governance:");
    println!("   - gov_propose");
    println!("   - gov_vote");
    println!("   - gov_getProposal");
    println!("   - gov_listProposals");
    println!("   - gov_getVote");
    println!("   - gov_getVotingPower");
    println!("   - gov_getTallyResult");

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

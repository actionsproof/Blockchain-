use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHealth {
    pub status: HealthStatus,
    pub uptime_seconds: u64,
    pub current_block: u64,
    pub last_block_time: u64,
    pub peer_count: usize,
    pub mempool_size: usize,
    pub sync_status: SyncStatus,
    pub validator_status: Option<ValidatorStatus>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub is_synced: bool,
    pub behind_blocks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorStatus {
    pub is_validator: bool,
    pub active: bool,
    pub blocks_proposed: u64,
    pub last_proposed: Option<u64>,
    pub stake_amount: String,
}

pub struct HealthMonitor {
    start_time: SystemTime,
    state: Arc<RwLock<crate::JsonRpcState>>,
}

impl HealthMonitor {
    pub fn new(state: Arc<RwLock<crate::JsonRpcState>>) -> Self {
        Self {
            start_time: SystemTime::now(),
            state,
        }
    }

    pub async fn check_health(&self) -> NodeHealth {
        let mut warnings = Vec::new();
        let state = self.state.read().await;

        // Calculate uptime
        let uptime_seconds = self
            .start_time
            .elapsed()
            .unwrap_or_default()
            .as_secs();

        // Get current block info
        let current_block = state.storage.get_latest_block_number().unwrap_or(0);
        
        let last_block_time = if let Ok(Some(block)) = state.storage.get_block(current_block) {
            block.timestamp
        } else {
            0
        };

        // Check if we're producing blocks
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let time_since_last_block = now.saturating_sub(last_block_time);
        
        if time_since_last_block > 120 {
            warnings.push(format!(
                "No new blocks in {} seconds (expected 30s block time)",
                time_since_last_block
            ));
        }

        // Get peer count
        let peer_count = 0; // TODO: Get from P2P layer when available

        if peer_count == 0 {
            warnings.push("No peers connected".to_string());
        }

        // Get mempool size
        let mempool_size = state.mempool.get_pending_count();

        if mempool_size > 10000 {
            warnings.push(format!(
                "Large mempool size: {} transactions",
                mempool_size
            ));
        }

        // Check sync status
        let sync_status = SyncStatus {
            is_synced: true, // TODO: Implement proper sync check
            behind_blocks: 0,
        };

        // Check validator status
        let validator_status = state.staking.get_validators()
            .into_iter()
            .find(|v| v.address == state.validator_address)
            .map(|v| ValidatorStatus {
                is_validator: true,
                active: v.active,
                blocks_proposed: v.blocks_proposed,
                last_proposed: Some(current_block),
                stake_amount: v.stake.to_string(),
            });

        if let Some(ref vs) = validator_status {
            if !vs.active {
                warnings.push("Validator is inactive".to_string());
            }
        }

        // Determine overall health status
        let status = if warnings.is_empty() {
            HealthStatus::Healthy
        } else if warnings.len() <= 2 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };

        NodeHealth {
            status,
            uptime_seconds,
            current_block,
            last_block_time,
            peer_count,
            mempool_size,
            sync_status,
            validator_status,
            warnings,
        }
    }

    pub async fn get_stats(&self) -> NodeStats {
        let state = self.state.read().await;
        
        let current_block = state.storage.get_latest_block_number().unwrap_or(0);
        
        // Calculate TPS (transactions per second) over last 100 blocks
        let mut total_txs = 0u64;
        let mut total_time = 0u64;
        
        for i in current_block.saturating_sub(100)..=current_block {
            if let Ok(Some(block)) = state.storage.get_block(i) {
                total_txs += block.transactions.len() as u64;
                if i > 0 {
                    if let Ok(Some(prev_block)) = state.storage.get_block(i - 1) {
                        total_time += block.timestamp.saturating_sub(prev_block.timestamp);
                    }
                }
            }
        }

        let tps = if total_time > 0 {
            total_txs as f64 / total_time as f64
        } else {
            0.0
        };

        // Get staking stats
        let validators = state.staking.get_validators();
        let total_staked: u128 = validators.iter().map(|v| v.stake).sum();
        
        // Get governance stats
        let proposals = state.governance.get_proposals();
        let active_proposals = proposals
            .iter()
            .filter(|p| matches!(p.status, crate::governance::ProposalStatus::Active))
            .count();

        NodeStats {
            current_block,
            total_transactions: total_txs,
            tps,
            validator_count: validators.len(),
            total_staked: total_staked.to_string(),
            proposal_count: proposals.len(),
            active_proposals,
            mempool_size: state.mempool.get_pending_count(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStats {
    pub current_block: u64,
    pub total_transactions: u64,
    pub tps: f64,
    pub validator_count: usize,
    pub total_staked: String,
    pub proposal_count: usize,
    pub active_proposals: usize,
    pub mempool_size: usize,
}

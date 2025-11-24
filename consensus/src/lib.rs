use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use types::{Action, BlockHeader};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub pubkey: String,
    pub stake: u64,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct ConsensusState {
    pub validators: HashMap<String, Validator>,
    pub current_proposer_index: usize,
    pub block_height: u64,
    pub finalized_height: u64,
}

impl ConsensusState {
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            current_proposer_index: 0,
            block_height: 0,
            finalized_height: 0,
        }
    }

    pub fn add_validator(&mut self, pubkey: String, stake: u64) {
        self.validators.insert(
            pubkey.clone(),
            Validator {
                pubkey,
                stake,
                active: true,
            },
        );
    }

    pub fn get_current_proposer(&self) -> Option<&Validator> {
        let active_validators: Vec<&Validator> = self
            .validators
            .values()
            .filter(|v| v.active)
            .collect();
        
        if active_validators.is_empty() {
            return None;
        }
        
        let index = self.current_proposer_index % active_validators.len();
        Some(active_validators[index])
    }

    pub fn rotate_proposer(&mut self) {
        self.current_proposer_index += 1;
    }

    pub fn increment_height(&mut self) {
        self.block_height += 1;
    }
}

pub struct ConsensusEngine {
    state: Arc<RwLock<ConsensusState>>,
}

impl ConsensusEngine {
    pub fn new() -> Self {
        let mut state = ConsensusState::new();
        
        // Initialize with some default validators for testing
        state.add_validator("validator1".to_string(), 1000);
        state.add_validator("validator2".to_string(), 1000);
        state.add_validator("validator3".to_string(), 1000);
        
        Self {
            state: Arc::new(RwLock::new(state)),
        }
    }

    pub async fn propose_block(&self, action: Action) -> Result<BlockHeader, String> {
        let mut state = self.state.write().await;
        
        let proposer = state
            .get_current_proposer()
            .ok_or("No active validators")?;
        
        // Calculate action hash
        let action_data = serde_json::to_string(&action)
            .map_err(|e| format!("Failed to serialize action: {}", e))?;
        let mut hasher = Sha256::new();
        hasher.update(action_data.as_bytes());
        let action_hash = hex::encode(hasher.finalize());
        
        // Create block header
        let block_header = BlockHeader {
            parent_hash: format!("block_{}", state.block_height),
            action_hash,
            actor_pubkey: action.actor.clone(),
            state_root: format!("state_root_{}", state.block_height + 1),
            receipts_root: format!("receipts_root_{}", state.block_height + 1),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            validator_commitment: proposer.pubkey.clone(),
            reward: 100, // Fixed reward for now
        };
        
        state.increment_height();
        state.rotate_proposer();
        
        Ok(block_header)
    }

    pub async fn validate_block(&self, header: &BlockHeader) -> Result<bool, String> {
        let state = self.state.read().await;
        
        // Check if the validator exists and is active
        if let Some(validator) = state.validators.get(&header.validator_commitment) {
            if !validator.active {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }
        
        // Basic validation passed
        Ok(true)
    }

    pub async fn finalize_block(&self, height: u64) -> Result<(), String> {
        let mut state = self.state.write().await;
        
        if height > state.finalized_height {
            state.finalized_height = height;
            println!("✅ Block {} finalized", height);
        }
        
        Ok(())
    }

    pub async fn get_block_height(&self) -> u64 {
        self.state.read().await.block_height
    }
}

pub async fn start_consensus(engine: Arc<ConsensusEngine>) {
    println!("🎯 Consensus engine started");
    
    // Consensus loop - for now just log status
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        let height = engine.get_block_height().await;
        println!("📊 Current block height: {}", height);
    }
}

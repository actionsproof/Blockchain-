//! Proof of Action (PoA) Consensus Engine
//! 
//! Implements a Proof of Action consensus mechanism where validators are selected
//! based on their stake and activity. Provides block proposal, validation, and finality.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Consensus configuration
#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    pub min_validators: usize,
    pub max_validators: usize,
    pub block_time_secs: u64,
    pub finality_threshold: usize, // Number of blocks for finality
    pub validator_rotation_blocks: u64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            min_validators: 3,
            max_validators: 100,
            block_time_secs: 30,
            finality_threshold: 10,
            validator_rotation_blocks: 100,
        }
    }
}

/// Validator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub address: String,
    pub stake: u64,
    pub commission_rate: u8,
    pub is_active: bool,
    pub last_block_produced: u64,
    pub blocks_produced: u64,
    pub missed_blocks: u64,
}

/// Block proposal for consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockProposal {
    pub height: u64,
    pub proposer: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub transactions: Vec<String>,
    pub state_root: String,
}

/// Vote on a block proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub block_height: u64,
    pub block_hash: String,
    pub validator: String,
    pub signature: Vec<u8>,
}

/// Consensus state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusState {
    pub current_height: u64,
    pub current_proposer: String,
    pub finalized_height: u64,
    pub validator_set: Vec<Validator>,
}

/// Consensus engine
pub struct ConsensusEngine {
    config: ConsensusConfig,
    state: Arc<RwLock<ConsensusState>>,
    pending_proposals: Arc<RwLock<HashMap<u64, BlockProposal>>>,
    votes: Arc<RwLock<HashMap<u64, Vec<Vote>>>>,
    finalized_blocks: Arc<RwLock<VecDeque<u64>>>,
}

impl ConsensusEngine {
    /// Create new consensus engine
    pub fn new() -> Self {
        Self::with_config(ConsensusConfig::default())
    }
    
    /// Create new consensus engine with custom config
    pub fn with_config(config: ConsensusConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(ConsensusState {
                current_height: 0,
                current_proposer: String::new(),
                finalized_height: 0,
                validator_set: Vec::new(),
            })),
            pending_proposals: Arc::new(RwLock::new(HashMap::new())),
            votes: Arc::new(RwLock::new(HashMap::new())),
            finalized_blocks: Arc::new(RwLock::new(VecDeque::new())),
        }
    }
    
    /// Add or update validator
    pub async fn add_validator(&self, validator: Validator) -> Result<()> {
        let mut state = self.state.write().await;
        
        // Check if validator already exists
        if let Some(existing) = state.validator_set.iter_mut()
            .find(|v| v.address == validator.address) {
            *existing = validator;
        } else {
            state.validator_set.push(validator);
        }
        
        // Sort by stake (descending)
        state.validator_set.sort_by(|a, b| b.stake.cmp(&a.stake));
        
        Ok(())
    }
    
    /// Remove validator
    pub async fn remove_validator(&self, address: &str) -> Result<()> {
        let mut state = self.state.write().await;
        state.validator_set.retain(|v| v.address != address);
        Ok(())
    }
    
    /// Get active validators
    pub async fn get_active_validators(&self) -> Vec<Validator> {
        let state = self.state.read().await;
        state.validator_set.iter()
            .filter(|v| v.is_active)
            .cloned()
            .collect()
    }
    
    /// Select block proposer for given height
    pub async fn select_proposer(&self, height: u64) -> Result<String> {
        let state = self.state.read().await;
        let active_validators: Vec<_> = state.validator_set.iter()
            .filter(|v| v.is_active)
            .collect();
        
        if active_validators.is_empty() {
            return Err(anyhow!("No active validators"));
        }
        
        if active_validators.len() < self.config.min_validators {
            return Err(anyhow!("Not enough active validators"));
        }
        
        // Round-robin selection weighted by stake
        let total_stake: u64 = active_validators.iter().map(|v| v.stake).sum();
        if total_stake == 0 {
            return Err(anyhow!("No staked validators"));
        }
        
        // Deterministic selection based on height and stake
        let mut accumulated_stake = 0u64;
        let target = (height * 31) % total_stake; // Simple deterministic selection
        
        for validator in active_validators {
            accumulated_stake += validator.stake;
            if accumulated_stake > target {
                return Ok(validator.address.clone());
            }
        }
        
        // Fallback to first validator
        Ok(active_validators[0].address.clone())
    }
    
    /// Propose a new block
    pub async fn propose_block(&self, proposal: BlockProposal) -> Result<()> {
        let state = self.state.read().await;
        
        // Verify proposer is the selected one
        let expected_proposer = drop(state);
        let expected_proposer = self.select_proposer(proposal.height).await?;
        
        if proposal.proposer != expected_proposer {
            return Err(anyhow!("Invalid proposer for height {}", proposal.height));
        }
        
        // Store proposal
        let mut proposals = self.pending_proposals.write().await;
        proposals.insert(proposal.height, proposal);
        
        Ok(())
    }
    
    /// Cast vote on a block proposal
    pub async fn vote(&self, vote: Vote) -> Result<()> {
        let state = self.state.read().await;
        
        // Verify voter is an active validator
        let is_validator = state.validator_set.iter()
            .any(|v| v.address == vote.validator && v.is_active);
        
        if !is_validator {
            return Err(anyhow!("Voter is not an active validator"));
        }
        
        drop(state);
        
        // Store vote
        let mut votes = self.votes.write().await;
        votes.entry(vote.block_height)
            .or_insert_with(Vec::new)
            .push(vote);
        
        // Check if we have enough votes for finality
        self.check_finality(votes.get(&vote.block_height).unwrap().len()).await?;
        
        Ok(())
    }
    
    /// Check if block can be finalized
    async fn check_finality(&self, vote_count: usize) -> Result<()> {
        let state = self.state.read().await;
        let active_count = state.validator_set.iter()
            .filter(|v| v.is_active)
            .count();
        
        // Need 2/3+ votes for finality
        let required_votes = (active_count * 2) / 3 + 1;
        
        if vote_count >= required_votes {
            // Block can be finalized
            return Ok(());
        }
        
        Ok(())
    }
    
    /// Finalize block at given height
    pub async fn finalize_block(&self, height: u64) -> Result<()> {
        let votes = self.votes.read().await;
        let block_votes = votes.get(&height)
            .ok_or_else(|| anyhow!("No votes for height {}", height))?;
        
        let state = self.state.read().await;
        let active_count = state.validator_set.iter()
            .filter(|v| v.is_active)
            .count();
        
        let required_votes = (active_count * 2) / 3 + 1;
        
        if block_votes.len() < required_votes {
            return Err(anyhow!("Not enough votes for finality"));
        }
        
        drop(state);
        drop(votes);
        
        // Update finalized height
        let mut state = self.state.write().await;
        state.finalized_height = height;
        
        // Add to finalized blocks queue
        let mut finalized = self.finalized_blocks.write().await;
        finalized.push_back(height);
        
        // Keep only recent finalized blocks
        while finalized.len() > self.config.finality_threshold {
            finalized.pop_front();
        }
        
        Ok(())
    }
    
    /// Get current consensus state
    pub async fn get_state(&self) -> ConsensusState {
        self.state.read().await.clone()
    }
    
    /// Update current height
    pub async fn set_height(&self, height: u64) -> Result<()> {
        let mut state = self.state.write().await;
        state.current_height = height;
        
        // Select proposer for new height
        drop(state);
        let proposer = self.select_proposer(height).await?;
        
        let mut state = self.state.write().await;
        state.current_proposer = proposer;
        
        Ok(())
    }
    
    /// Record that a validator produced a block
    pub async fn record_block_production(&self, validator: &str, height: u64) -> Result<()> {
        let mut state = self.state.write().await;
        
        if let Some(v) = state.validator_set.iter_mut()
            .find(|v| v.address == validator) {
            v.blocks_produced += 1;
            v.last_block_produced = height;
        }
        
        Ok(())
    }
    
    /// Record that a validator missed their block proposal
    pub async fn record_missed_block(&self, validator: &str) -> Result<()> {
        let mut state = self.state.write().await;
        
        if let Some(v) = state.validator_set.iter_mut()
            .find(|v| v.address == validator) {
            v.missed_blocks += 1;
            
            // Deactivate validator if they miss too many blocks
            if v.missed_blocks > 10 {
                v.is_active = false;
            }
        }
        
        Ok(())
    }
    
    /// Rotate validators (called periodically)
    pub async fn rotate_validators(&self) -> Result<()> {
        let mut state = self.state.write().await;
        
        // Deactivate validators with low performance
        for validator in &mut state.validator_set {
            if validator.blocks_produced > 0 {
                let miss_rate = validator.missed_blocks as f64 / validator.blocks_produced as f64;
                if miss_rate > 0.3 {
                    validator.is_active = false;
                }
            }
        }
        
        // Ensure we have minimum validators active
        let active_count = state.validator_set.iter()
            .filter(|v| v.is_active)
            .count();
        
        if active_count < self.config.min_validators {
            // Activate top validators by stake
            let mut sorted = state.validator_set.clone();
            sorted.sort_by(|a, b| b.stake.cmp(&a.stake));
            
            for (i, validator) in sorted.iter().enumerate() {
                if i < self.config.min_validators {
                    if let Some(v) = state.validator_set.iter_mut()
                        .find(|v| v.address == validator.address) {
                        v.is_active = true;
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Start consensus process
pub async fn start_consensus(engine: Arc<ConsensusEngine>) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        // Consensus logic runs in the node's block production loop
        // This function can be used for periodic maintenance tasks
        
        let state = engine.get_state().await;
        
        // Rotate validators every N blocks
        if state.current_height > 0 && 
           state.current_height % engine.config.validator_rotation_blocks == 0 {
            let _ = engine.rotate_validators().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_validator_management() {
        let engine = ConsensusEngine::new();
        
        let validator = Validator {
            address: "ACT-validator1".to_string(),
            stake: 100_000,
            commission_rate: 10,
            is_active: true,
            last_block_produced: 0,
            blocks_produced: 0,
            missed_blocks: 0,
        };
        
        engine.add_validator(validator).await.unwrap();
        
        let validators = engine.get_active_validators().await;
        assert_eq!(validators.len(), 1);
        assert_eq!(validators[0].address, "ACT-validator1");
    }
    
    #[tokio::test]
    async fn test_proposer_selection() {
        let engine = ConsensusEngine::new();
        
        for i in 1..=3 {
            let validator = Validator {
                address: format!("ACT-validator{}", i),
                stake: 100_000 * i as u64,
                commission_rate: 10,
                is_active: true,
                last_block_produced: 0,
                blocks_produced: 0,
                missed_blocks: 0,
            };
            engine.add_validator(validator).await.unwrap();
        }
        
        let proposer = engine.select_proposer(1).await.unwrap();
        assert!(!proposer.is_empty());
    }
    
    #[tokio::test]
    async fn test_block_finalization() {
        let engine = ConsensusEngine::new();
        
        // Add 3 validators
        for i in 1..=3 {
            let validator = Validator {
                address: format!("ACT-validator{}", i),
                stake: 100_000,
                commission_rate: 10,
                is_active: true,
                last_block_produced: 0,
                blocks_produced: 0,
                missed_blocks: 0,
            };
            engine.add_validator(validator).await.unwrap();
        }
        
        // Cast votes
        for i in 1..=3 {
            let vote = Vote {
                block_height: 1,
                block_hash: "0xblock1".to_string(),
                validator: format!("ACT-validator{}", i),
                signature: vec![],
            };
            engine.vote(vote).await.unwrap();
        }
        
        // Should be able to finalize
        engine.finalize_block(1).await.unwrap();
        
        let state = engine.get_state().await;
        assert_eq!(state.finalized_height, 1);
    }
}

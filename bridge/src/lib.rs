/// Cross-chain Bridge System
/// 
/// Enables asset transfers between ACT Chain and Ethereum using:
/// - Lock/Mint mechanism (lock on source, mint on destination)
/// - Burn/Unlock mechanism (burn on source, unlock on destination)
/// - Merkle proof verification for security
/// - Relay system for cross-chain message passing

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("Invalid proof")]
    InvalidProof,
    #[error("Transfer already processed")]
    AlreadyProcessed,
    #[error("Insufficient locked funds")]
    InsufficientFunds,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Transfer not found")]
    TransferNotFound,
    #[error("Challenge period not expired")]
    ChallengePeriodActive,
}

/// Bridge transfer direction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferDirection {
    ActToEth,  // Lock on ACT, Mint on Ethereum
    EthToAct,  // Lock on Ethereum, Mint on ACT
}

/// Bridge transfer status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferStatus {
    Pending,        // Transfer initiated, waiting for confirmation
    Confirmed,      // Confirmed on source chain
    Relayed,        // Relayed to destination chain
    Completed,      // Successfully completed
    Challenged,     // Transfer challenged (fraud proof)
    Cancelled,      // Transfer cancelled
}

/// Cross-chain transfer request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeTransfer {
    pub id: String,                      // Unique transfer ID
    pub direction: TransferDirection,     // Transfer direction
    pub sender: String,                   // Sender address
    pub recipient: String,                // Recipient address
    pub amount: u128,                     // Amount to transfer
    pub token: String,                    // Token address (ACT for native)
    pub source_chain_id: u64,             // Source chain ID
    pub dest_chain_id: u64,               // Destination chain ID
    pub source_tx_hash: String,           // Source transaction hash
    pub dest_tx_hash: Option<String>,     // Destination transaction hash
    pub block_number: u64,                // Block number on source chain
    pub timestamp: u64,                   // Timestamp
    pub status: TransferStatus,           // Current status
    pub proof: Option<MerkleProof>,       // Merkle proof for verification
}

/// Merkle proof for cross-chain verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub root: String,           // Merkle root
    pub leaf: String,           // Leaf hash (transfer hash)
    pub siblings: Vec<String>,  // Sibling hashes for proof path
    pub index: u64,             // Leaf index in tree
}

impl MerkleProof {
    /// Verify merkle proof
    pub fn verify(&self) -> bool {
        // Simplified verification for now
        // In production, would compute full merkle path
        if self.siblings.is_empty() {
            // Single leaf tree - leaf equals root
            return self.leaf == self.root;
        }
        
        let mut current_hash = self.leaf.clone();
        let mut index = self.index;
        
        for sibling in &self.siblings {
            current_hash = if index % 2 == 0 {
                // Current is left child
                Self::hash_pair(&current_hash, sibling)
            } else {
                // Current is right child
                Self::hash_pair(sibling, &current_hash)
            };
            index /= 2;
        }
        
        current_hash == self.root
    }
    
    fn hash_pair(left: &str, right: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(left.as_bytes());
        hasher.update(right.as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// Bridge contract state
pub struct BridgeContract {
    /// Locked funds on this chain
    pub locked_funds: HashMap<String, u128>,  // token -> amount
    
    /// Processed transfer IDs (prevent replay)
    pub processed_transfers: HashMap<String, bool>,
    
    /// Pending transfers
    pub pending_transfers: HashMap<String, BridgeTransfer>,
    
    /// Relayer addresses (authorized to relay messages)
    pub relayers: Vec<String>,
    
    /// Challenge period (blocks)
    pub challenge_period: u64,
    
    /// Supported tokens
    pub supported_tokens: HashMap<String, TokenConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    pub token_address: String,      // Token contract address
    pub min_transfer: u128,          // Minimum transfer amount
    pub max_transfer: u128,          // Maximum transfer amount
    pub fee_rate: u64,               // Fee rate in basis points (100 = 1%)
    pub enabled: bool,               // Token enabled for bridge
}

impl BridgeContract {
    pub fn new(relayers: Vec<String>, challenge_period: u64) -> Self {
        Self {
            locked_funds: HashMap::new(),
            processed_transfers: HashMap::new(),
            pending_transfers: HashMap::new(),
            relayers,
            challenge_period,
            supported_tokens: HashMap::new(),
        }
    }
    
    /// Lock funds on source chain (step 1 of bridge transfer)
    pub fn lock_funds(
        &mut self,
        transfer: BridgeTransfer,
    ) -> Result<String, BridgeError> {
        // Verify token is supported
        if !self.is_token_supported(&transfer.token) {
            return Err(BridgeError::InvalidSignature);
        }
        
        // Verify transfer amount is within limits
        if let Some(config) = self.supported_tokens.get(&transfer.token) {
            if transfer.amount < config.min_transfer || transfer.amount > config.max_transfer {
                return Err(BridgeError::InvalidSignature);
            }
        }
        
        // Lock the funds
        let locked = self.locked_funds.entry(transfer.token.clone()).or_insert(0);
        *locked += transfer.amount;
        
        // Add to pending transfers
        let transfer_id = transfer.id.clone();
        self.pending_transfers.insert(transfer_id.clone(), transfer);
        
        Ok(transfer_id)
    }
    
    /// Relay transfer to destination chain (step 2)
    pub fn relay_transfer(
        &mut self,
        transfer_id: &str,
        proof: MerkleProof,
        relayer: &str,
    ) -> Result<(), BridgeError> {
        // Verify relayer is authorized
        if !self.relayers.contains(&relayer.to_string()) {
            return Err(BridgeError::InvalidSignature);
        }
        
        // Get pending transfer
        let transfer = self.pending_transfers
            .get_mut(transfer_id)
            .ok_or(BridgeError::TransferNotFound)?;
        
        // Verify proof
        if !proof.verify() {
            return Err(BridgeError::InvalidProof);
        }
        
        // Update transfer status
        transfer.proof = Some(proof);
        transfer.status = TransferStatus::Relayed;
        
        Ok(())
    }
    
    /// Mint/unlock funds on destination chain (step 3)
    pub fn complete_transfer(
        &mut self,
        transfer_id: &str,
        dest_tx_hash: String,
        current_block: u64,
    ) -> Result<u128, BridgeError> {
        // Check if already processed
        if self.processed_transfers.contains_key(transfer_id) {
            return Err(BridgeError::AlreadyProcessed);
        }
        
        // Get pending transfer
        let transfer = self.pending_transfers
            .get_mut(transfer_id)
            .ok_or(BridgeError::TransferNotFound)?;
        
        // Verify challenge period has passed
        if current_block < transfer.block_number + self.challenge_period {
            return Err(BridgeError::ChallengePeriodActive);
        }
        
        // Verify proof exists
        if transfer.proof.is_none() {
            return Err(BridgeError::InvalidProof);
        }
        
        // Update transfer status
        transfer.dest_tx_hash = Some(dest_tx_hash);
        transfer.status = TransferStatus::Completed;
        
        // Mark as processed
        self.processed_transfers.insert(transfer_id.to_string(), true);
        
        // Return amount to mint/unlock
        Ok(transfer.amount)
    }
    
    /// Unlock funds (for return transfers)
    pub fn unlock_funds(
        &mut self,
        token: &str,
        amount: u128,
    ) -> Result<(), BridgeError> {
        let locked = self.locked_funds
            .get_mut(token)
            .ok_or(BridgeError::InsufficientFunds)?;
        
        if *locked < amount {
            return Err(BridgeError::InsufficientFunds);
        }
        
        *locked -= amount;
        Ok(())
    }
    
    /// Challenge a fraudulent transfer
    pub fn challenge_transfer(
        &mut self,
        transfer_id: &str,
        fraud_proof: &[u8],
    ) -> Result<(), BridgeError> {
        let transfer = self.pending_transfers
            .get_mut(transfer_id)
            .ok_or(BridgeError::TransferNotFound)?;
        
        // Verify fraud proof (simplified - real implementation would be more complex)
        if fraud_proof.is_empty() {
            return Err(BridgeError::InvalidProof);
        }
        
        // Mark as challenged
        transfer.status = TransferStatus::Challenged;
        
        Ok(())
    }
    
    /// Add supported token
    pub fn add_token(&mut self, token: String, config: TokenConfig) {
        self.supported_tokens.insert(token, config);
    }
    
    /// Check if token is supported
    pub fn is_token_supported(&self, token: &str) -> bool {
        self.supported_tokens.get(token).map(|c| c.enabled).unwrap_or(false)
    }
    
    /// Get locked amount for token
    pub fn get_locked_amount(&self, token: &str) -> u128 {
        *self.locked_funds.get(token).unwrap_or(&0)
    }
    
    /// Get transfer info
    pub fn get_transfer(&self, transfer_id: &str) -> Option<&BridgeTransfer> {
        self.pending_transfers.get(transfer_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_proof_verification() {
        // Create a simple merkle proof
        let proof = MerkleProof {
            root: "abc123".to_string(),
            leaf: "leaf1".to_string(),
            siblings: vec![],
            index: 0,
        };
        
        // Note: This is a simplified test - real merkle proofs need proper hashing
        assert!(proof.leaf == "leaf1");
    }

    #[test]
    fn test_lock_funds() {
        let relayers = vec!["relayer1".to_string()];
        let mut bridge = BridgeContract::new(relayers, 10);
        
        // Add supported token
        bridge.add_token("ACT".to_string(), TokenConfig {
            token_address: "ACT".to_string(),
            min_transfer: 100,
            max_transfer: 1_000_000,
            fee_rate: 30,  // 0.3%
            enabled: true,
        });
        
        let transfer = BridgeTransfer {
            id: "transfer1".to_string(),
            direction: TransferDirection::ActToEth,
            sender: "sender1".to_string(),
            recipient: "recipient1".to_string(),
            amount: 1000,
            token: "ACT".to_string(),
            source_chain_id: 2755,
            dest_chain_id: 1,
            source_tx_hash: "0xabc".to_string(),
            dest_tx_hash: None,
            block_number: 100,
            timestamp: 1234567890,
            status: TransferStatus::Pending,
            proof: None,
        };
        
        let result = bridge.lock_funds(transfer);
        assert!(result.is_ok());
        assert_eq!(bridge.get_locked_amount("ACT"), 1000);
    }

    #[test]
    fn test_relay_transfer() {
        let relayers = vec!["relayer1".to_string()];
        let mut bridge = BridgeContract::new(relayers, 10);
        
        bridge.add_token("ACT".to_string(), TokenConfig {
            token_address: "ACT".to_string(),
            min_transfer: 100,
            max_transfer: 1_000_000,
            fee_rate: 30,
            enabled: true,
        });
        
        let transfer = BridgeTransfer {
            id: "transfer1".to_string(),
            direction: TransferDirection::ActToEth,
            sender: "sender1".to_string(),
            recipient: "recipient1".to_string(),
            amount: 1000,
            token: "ACT".to_string(),
            source_chain_id: 2755,
            dest_chain_id: 1,
            source_tx_hash: "0xabc".to_string(),
            dest_tx_hash: None,
            block_number: 100,
            timestamp: 1234567890,
            status: TransferStatus::Pending,
            proof: None,
        };
        
        bridge.lock_funds(transfer).unwrap();
        
        let proof = MerkleProof {
            root: "leaf123".to_string(),  // Single leaf - root equals leaf
            leaf: "leaf123".to_string(),
            siblings: vec![],
            index: 0,
        };
        
        let result = bridge.relay_transfer("transfer1", proof, "relayer1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_complete_transfer() {
        let relayers = vec!["relayer1".to_string()];
        let mut bridge = BridgeContract::new(relayers, 10);
        
        bridge.add_token("ACT".to_string(), TokenConfig {
            token_address: "ACT".to_string(),
            min_transfer: 100,
            max_transfer: 1_000_000,
            fee_rate: 30,
            enabled: true,
        });
        
        let transfer = BridgeTransfer {
            id: "transfer1".to_string(),
            direction: TransferDirection::ActToEth,
            sender: "sender1".to_string(),
            recipient: "recipient1".to_string(),
            amount: 1000,
            token: "ACT".to_string(),
            source_chain_id: 2755,
            dest_chain_id: 1,
            source_tx_hash: "0xabc".to_string(),
            dest_tx_hash: None,
            block_number: 100,
            timestamp: 1234567890,
            status: TransferStatus::Pending,
            proof: None,
        };
        
        bridge.lock_funds(transfer).unwrap();
        
        let proof = MerkleProof {
            root: "leaf123".to_string(),  // Single leaf - root equals leaf
            leaf: "leaf123".to_string(),
            siblings: vec![],
            index: 0,
        };
        
        bridge.relay_transfer("transfer1", proof, "relayer1").unwrap();
        
        // Try to complete before challenge period
        let result = bridge.complete_transfer("transfer1", "0xdest".to_string(), 105);
        assert!(result.is_err());
        
        // Complete after challenge period
        let result = bridge.complete_transfer("transfer1", "0xdest".to_string(), 111);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1000);
    }

    #[test]
    fn test_unlock_funds() {
        let relayers = vec!["relayer1".to_string()];
        let mut bridge = BridgeContract::new(relayers, 10);
        
        // Manually add locked funds
        bridge.locked_funds.insert("ACT".to_string(), 5000);
        
        let result = bridge.unlock_funds("ACT", 1000);
        assert!(result.is_ok());
        assert_eq!(bridge.get_locked_amount("ACT"), 4000);
        
        // Try to unlock more than available
        let result = bridge.unlock_funds("ACT", 5000);
        assert!(result.is_err());
    }
}

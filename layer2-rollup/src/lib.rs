use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RollupError {
    #[error("Invalid batch: {0}")]
    InvalidBatch(String),
    #[error("Fraud detected: {0}")]
    FraudDetected(String),
    #[error("Challenge period not expired")]
    ChallengePeriodActive,
    #[error("Invalid state root")]
    InvalidStateRoot,
    #[error("Batch not found")]
    BatchNotFound,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Invalid fraud proof")]
    InvalidFraudProof,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BatchStatus {
    Pending,
    Challenged,
    Finalized,
    Reverted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u128,
    pub nonce: u64,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch {
    pub batch_id: u64,
    pub transactions: Vec<Transaction>,
    pub prev_state_root: String,
    pub post_state_root: String,
    pub timestamp: u64,
    pub sequencer: String,
    pub status: BatchStatus,
    pub challenge_deadline: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudProof {
    pub batch_id: u64,
    pub transaction_index: usize,
    pub claimed_state_root: String,
    pub correct_state_root: String,
    pub proof_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub batch_id: u64,
    pub challenger: String,
    pub fraud_proof: FraudProof,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L1ToL2Message {
    pub message_id: u64,
    pub sender: String,
    pub target: String,
    pub value: u128,
    pub data: Vec<u8>,
    pub l1_block: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2ToL1Message {
    pub message_id: u64,
    pub sender: String,
    pub target: String,
    pub value: u128,
    pub data: Vec<u8>,
    pub l2_batch: u64,
    pub finalized: bool,
}

pub struct RollupContract {
    pub batches: HashMap<u64, Batch>,
    pub next_batch_id: u64,
    pub challenges: HashMap<u64, Challenge>,
    pub l1_to_l2_messages: HashMap<u64, L1ToL2Message>,
    pub l2_to_l1_messages: HashMap<u64, L2ToL1Message>,
    pub next_message_id: u64,
    pub challenge_period: u64, // in seconds
    pub sequencers: Vec<String>,
    pub state_roots: HashMap<u64, String>, // batch_id -> state_root
}

impl RollupContract {
    pub fn new(challenge_period: u64) -> Self {
        Self {
            batches: HashMap::new(),
            next_batch_id: 1,
            challenges: HashMap::new(),
            l1_to_l2_messages: HashMap::new(),
            l2_to_l1_messages: HashMap::new(),
            next_message_id: 1,
            challenge_period,
            sequencers: Vec::new(),
            state_roots: HashMap::new(),
        }
    }

    pub fn add_sequencer(&mut self, sequencer: String) {
        if !self.sequencers.contains(&sequencer) {
            self.sequencers.push(sequencer);
        }
    }

    pub fn is_sequencer(&self, address: &str) -> bool {
        self.sequencers.contains(&address.to_string())
    }

    pub fn submit_batch(
        &mut self,
        transactions: Vec<Transaction>,
        prev_state_root: String,
        post_state_root: String,
        sequencer: String,
        current_time: u64,
    ) -> Result<u64, RollupError> {
        if !self.is_sequencer(&sequencer) {
            return Err(RollupError::InvalidBatch("Not a sequencer".to_string()));
        }

        if transactions.is_empty() {
            return Err(RollupError::InvalidBatch("Empty batch".to_string()));
        }

        let batch_id = self.next_batch_id;
        self.next_batch_id += 1;

        let batch = Batch {
            batch_id,
            transactions,
            prev_state_root,
            post_state_root: post_state_root.clone(),
            timestamp: current_time,
            sequencer,
            status: BatchStatus::Pending,
            challenge_deadline: current_time + self.challenge_period,
        };

        self.state_roots.insert(batch_id, post_state_root);
        self.batches.insert(batch_id, batch);

        Ok(batch_id)
    }

    pub fn challenge_batch(
        &mut self,
        batch_id: u64,
        fraud_proof: FraudProof,
        challenger: String,
        current_time: u64,
    ) -> Result<(), RollupError> {
        let batch = self
            .batches
            .get(&batch_id)
            .ok_or(RollupError::BatchNotFound)?;

        if batch.status != BatchStatus::Pending {
            return Err(RollupError::InvalidBatch(
                "Batch not in pending state".to_string(),
            ));
        }

        if current_time > batch.challenge_deadline {
            return Err(RollupError::ChallengePeriodActive);
        }

        // Verify fraud proof
        if !self.verify_fraud_proof(&fraud_proof, batch)? {
            return Err(RollupError::InvalidFraudProof);
        }

        let challenge = Challenge {
            batch_id,
            challenger,
            fraud_proof,
            timestamp: current_time,
        };

        self.challenges.insert(batch_id, challenge);

        // Update batch status
        if let Some(batch) = self.batches.get_mut(&batch_id) {
            batch.status = BatchStatus::Challenged;
        }

        Ok(())
    }

    fn verify_fraud_proof(&self, proof: &FraudProof, batch: &Batch) -> Result<bool, RollupError> {
        // Simplified fraud proof verification
        // In a real implementation, this would:
        // 1. Re-execute the transaction at the given index
        // 2. Compare the resulting state root with the claimed state root
        // 3. If they differ, the fraud proof is valid

        if proof.batch_id != batch.batch_id {
            return Ok(false);
        }

        if proof.transaction_index >= batch.transactions.len() {
            return Ok(false);
        }

        // For this implementation, we check if the claimed state root matches
        // the batch's post_state_root and if the correct state root differs
        if proof.claimed_state_root != batch.post_state_root {
            return Ok(false);
        }

        // In a real system, we'd re-execute and verify the correct_state_root
        Ok(proof.claimed_state_root != proof.correct_state_root)
    }

    pub fn finalize_batch(&mut self, batch_id: u64, current_time: u64) -> Result<(), RollupError> {
        let batch = self
            .batches
            .get(&batch_id)
            .ok_or(RollupError::BatchNotFound)?;

        if batch.status == BatchStatus::Challenged {
            return Err(RollupError::FraudDetected(
                "Batch was challenged".to_string(),
            ));
        }

        if current_time < batch.challenge_deadline {
            return Err(RollupError::ChallengePeriodActive);
        }

        if let Some(batch) = self.batches.get_mut(&batch_id) {
            batch.status = BatchStatus::Finalized;
        }

        // Finalize any L2->L1 messages in this batch
        for message in self.l2_to_l1_messages.values_mut() {
            if message.l2_batch == batch_id && !message.finalized {
                message.finalized = true;
            }
        }

        Ok(())
    }

    pub fn revert_batch(&mut self, batch_id: u64) -> Result<(), RollupError> {
        let batch = self
            .batches
            .get(&batch_id)
            .ok_or(RollupError::BatchNotFound)?;

        if batch.status != BatchStatus::Challenged {
            return Err(RollupError::InvalidBatch(
                "Batch not challenged".to_string(),
            ));
        }

        if let Some(batch) = self.batches.get_mut(&batch_id) {
            batch.status = BatchStatus::Reverted;
        }

        // Remove state root
        self.state_roots.remove(&batch_id);

        Ok(())
    }

    pub fn send_l1_to_l2_message(
        &mut self,
        sender: String,
        target: String,
        value: u128,
        data: Vec<u8>,
        l1_block: u64,
    ) -> u64 {
        let message_id = self.next_message_id;
        self.next_message_id += 1;

        let message = L1ToL2Message {
            message_id,
            sender,
            target,
            value,
            data,
            l1_block,
        };

        self.l1_to_l2_messages.insert(message_id, message);
        message_id
    }

    pub fn send_l2_to_l1_message(
        &mut self,
        sender: String,
        target: String,
        value: u128,
        data: Vec<u8>,
        l2_batch: u64,
    ) -> u64 {
        let message_id = self.next_message_id;
        self.next_message_id += 1;

        let message = L2ToL1Message {
            message_id,
            sender,
            target,
            value,
            data,
            l2_batch,
            finalized: false,
        };

        self.l2_to_l1_messages.insert(message_id, message);
        message_id
    }

    pub fn get_batch(&self, batch_id: u64) -> Option<&Batch> {
        self.batches.get(&batch_id)
    }

    pub fn get_challenge(&self, batch_id: u64) -> Option<&Challenge> {
        self.challenges.get(&batch_id)
    }

    pub fn get_l2_to_l1_message(&self, message_id: u64) -> Option<&L2ToL1Message> {
        self.l2_to_l1_messages.get(&message_id)
    }

    pub fn calculate_state_root(transactions: &[Transaction]) -> String {
        let mut hasher = Sha256::new();
        for tx in transactions {
            let tx_bytes = serde_json::to_vec(tx).unwrap_or_default();
            hasher.update(&tx_bytes);
        }
        hex::encode(hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_submission() {
        let mut rollup = RollupContract::new(7 * 24 * 3600); // 7 days
        rollup.add_sequencer("sequencer1".to_string());

        let transactions = vec![Transaction {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 100,
            nonce: 1,
            data: vec![],
        }];

        let prev_root = "prev_root".to_string();
        let post_root = RollupContract::calculate_state_root(&transactions);

        let batch_id = rollup
            .submit_batch(transactions, prev_root, post_root, "sequencer1".to_string(), 1000)
            .unwrap();

        assert_eq!(batch_id, 1);
        let batch = rollup.get_batch(batch_id).unwrap();
        assert_eq!(batch.status, BatchStatus::Pending);
    }

    #[test]
    fn test_batch_finalization() {
        let mut rollup = RollupContract::new(100); // 100 seconds challenge period
        rollup.add_sequencer("sequencer1".to_string());

        let transactions = vec![Transaction {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 100,
            nonce: 1,
            data: vec![],
        }];

        let post_root = RollupContract::calculate_state_root(&transactions);
        let batch_id = rollup
            .submit_batch(
                transactions,
                "prev".to_string(),
                post_root,
                "sequencer1".to_string(),
                1000,
            )
            .unwrap();

        // Try to finalize before challenge period
        assert!(rollup.finalize_batch(batch_id, 1050).is_err());

        // Finalize after challenge period
        assert!(rollup.finalize_batch(batch_id, 1101).is_ok());

        let batch = rollup.get_batch(batch_id).unwrap();
        assert_eq!(batch.status, BatchStatus::Finalized);
    }

    #[test]
    fn test_fraud_challenge() {
        let mut rollup = RollupContract::new(100);
        rollup.add_sequencer("sequencer1".to_string());

        let transactions = vec![Transaction {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 100,
            nonce: 1,
            data: vec![],
        }];

        let post_root = RollupContract::calculate_state_root(&transactions);
        let batch_id = rollup
            .submit_batch(
                transactions,
                "prev".to_string(),
                post_root.clone(),
                "sequencer1".to_string(),
                1000,
            )
            .unwrap();

        let fraud_proof = FraudProof {
            batch_id,
            transaction_index: 0,
            claimed_state_root: post_root,
            correct_state_root: "different_root".to_string(),
            proof_data: vec![],
        };

        assert!(rollup
            .challenge_batch(batch_id, fraud_proof, "challenger".to_string(), 1050)
            .is_ok());

        let batch = rollup.get_batch(batch_id).unwrap();
        assert_eq!(batch.status, BatchStatus::Challenged);
    }

    #[test]
    fn test_l1_to_l2_message() {
        let mut rollup = RollupContract::new(100);

        let message_id = rollup.send_l1_to_l2_message(
            "alice".to_string(),
            "contract".to_string(),
            1000,
            vec![1, 2, 3],
            100,
        );

        assert_eq!(message_id, 1);
        let message = rollup.l1_to_l2_messages.get(&message_id).unwrap();
        assert_eq!(message.sender, "alice");
        assert_eq!(message.value, 1000);
    }

    #[test]
    fn test_l2_to_l1_message() {
        let mut rollup = RollupContract::new(100);

        let message_id = rollup.send_l2_to_l1_message(
            "contract".to_string(),
            "bob".to_string(),
            500,
            vec![4, 5, 6],
            1,
        );

        assert_eq!(message_id, 1);
        let message = rollup.get_l2_to_l1_message(message_id).unwrap();
        assert_eq!(message.target, "bob");
        assert_eq!(message.finalized, false);
    }

    #[test]
    fn test_batch_revert() {
        let mut rollup = RollupContract::new(100);
        rollup.add_sequencer("sequencer1".to_string());

        let transactions = vec![Transaction {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 100,
            nonce: 1,
            data: vec![],
        }];

        let post_root = RollupContract::calculate_state_root(&transactions);
        let batch_id = rollup
            .submit_batch(
                transactions,
                "prev".to_string(),
                post_root.clone(),
                "sequencer1".to_string(),
                1000,
            )
            .unwrap();

        // Challenge the batch
        let fraud_proof = FraudProof {
            batch_id,
            transaction_index: 0,
            claimed_state_root: post_root,
            correct_state_root: "correct".to_string(),
            proof_data: vec![],
        };

        rollup
            .challenge_batch(batch_id, fraud_proof, "challenger".to_string(), 1050)
            .unwrap();

        // Revert the challenged batch
        assert!(rollup.revert_batch(batch_id).is_ok());

        let batch = rollup.get_batch(batch_id).unwrap();
        assert_eq!(batch.status, BatchStatus::Reverted);
    }

    #[test]
    fn test_state_root_calculation() {
        let transactions = vec![
            Transaction {
                from: "alice".to_string(),
                to: "bob".to_string(),
                amount: 100,
                nonce: 1,
                data: vec![],
            },
            Transaction {
                from: "bob".to_string(),
                to: "charlie".to_string(),
                amount: 50,
                nonce: 1,
                data: vec![],
            },
        ];

        let root1 = RollupContract::calculate_state_root(&transactions);
        let root2 = RollupContract::calculate_state_root(&transactions);

        // Same transactions should produce same root
        assert_eq!(root1, root2);

        // Different transactions should produce different root
        let different_txs = vec![Transaction {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 200,
            nonce: 1,
            data: vec![],
        }];
        let root3 = RollupContract::calculate_state_root(&different_txs);
        assert_ne!(root1, root3);
    }
}

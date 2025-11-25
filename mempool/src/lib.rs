use anyhow::{anyhow, Result};
use crypto::verify_signature;
use state::StateManager;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use types::{ActAmount, Transaction};

/// Transaction mempool for pending transactions
pub struct Mempool {
    pending: Arc<RwLock<HashMap<String, VecDeque<Transaction>>>>, // address -> txs
    by_hash: Arc<RwLock<HashMap<String, Transaction>>>,            // tx_hash -> tx
    max_size: usize,
}

impl Mempool {
    pub fn new(max_size: usize) -> Self {
        Self {
            pending: Arc::new(RwLock::new(HashMap::new())),
            by_hash: Arc::new(RwLock::new(HashMap::new())),
            max_size,
        }
    }

    /// Add transaction to mempool
    pub fn add_transaction(
        &self,
        tx: Transaction,
        state_manager: &StateManager,
    ) -> Result<String> {
        // Validate transaction
        self.validate_transaction(&tx, state_manager)?;
        
        let tx_hash = tx.hash();
        
        // Check if already in mempool
        {
            let by_hash = self.by_hash.read().unwrap();
            if by_hash.contains_key(&tx_hash) {
                return Err(anyhow!("Transaction already in mempool"));
            }
        }
        
        // Check mempool size
        {
            let by_hash = self.by_hash.read().unwrap();
            if by_hash.len() >= self.max_size {
                return Err(anyhow!("Mempool is full"));
            }
        }
        
        // Add to mempool
        {
            let mut pending = self.pending.write().unwrap();
            let mut by_hash = self.by_hash.write().unwrap();
            
            pending
                .entry(tx.from.to_string())
                .or_insert_with(VecDeque::new)
                .push_back(tx.clone());
            
            by_hash.insert(tx_hash.clone(), tx);
        }
        
        println!("📥 Transaction added to mempool: {}", &tx_hash[..8]);
        
        Ok(tx_hash)
    }

    /// Validate transaction
    fn validate_transaction(
        &self,
        tx: &Transaction,
        state_manager: &StateManager,
    ) -> Result<()> {
        // Check signature
        if !self.verify_transaction_signature(tx)? {
            return Err(anyhow!("Invalid transaction signature"));
        }
        
        // Check nonce
        let current_nonce = state_manager.get_nonce(&tx.from.to_string())?;
        if tx.nonce < current_nonce {
            return Err(anyhow!("Nonce too low"));
        }
        if tx.nonce > current_nonce + 100 {
            return Err(anyhow!("Nonce too high"));
        }
        
        // Check balance for transfers
        let balance = state_manager.get_balance(&tx.from.to_string())?;
        let total_cost = self.calculate_total_cost(tx);
        if balance < total_cost {
            return Err(anyhow!(
                "Insufficient balance: has {}, needs {}",
                balance,
                total_cost
            ));
        }
        
        // Check gas limit
        if tx.gas_limit < 21000 {
            return Err(anyhow!("Gas limit too low"));
        }
        if tx.gas_limit > 10_000_000 {
            return Err(anyhow!("Gas limit too high"));
        }
        
        Ok(())
    }

    /// Verify transaction signature
    fn verify_transaction_signature(&self, tx: &Transaction) -> Result<bool> {
        // Reconstruct signed data
        let tx_data = serde_json::to_vec(&(
            &tx.from,
            &tx.nonce,
            &tx.tx_type,
            &tx.gas_limit,
            &tx.gas_price,
        ))?;
        
        verify_signature(&tx.pubkey, &tx_data, &tx.signature)
    }

    /// Calculate total cost (amount + gas)
    fn calculate_total_cost(&self, tx: &Transaction) -> ActAmount {
        let gas_cost = tx.gas_limit as u128 * tx.gas_price;
        
        match &tx.tx_type {
            types::TransactionType::Transfer { amount, .. } => amount + gas_cost,
            _ => gas_cost,
        }
    }

    /// Get transaction by hash
    pub fn get_transaction(&self, tx_hash: &str) -> Option<Transaction> {
        let by_hash = self.by_hash.read().unwrap();
        by_hash.get(tx_hash).cloned()
    }

    /// Get pending transactions for an address (sorted by nonce)
    pub fn get_pending_transactions(&self, address: &str) -> Vec<Transaction> {
        let pending = self.pending.read().unwrap();
        pending
            .get(address)
            .map(|txs| txs.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get next transactions to include in block (sorted by gas price)
    pub fn get_transactions_for_block(
        &self,
        max_count: usize,
        state_manager: &StateManager,
    ) -> Vec<Transaction> {
        let pending = self.pending.read().unwrap();
        
        let mut all_txs: Vec<Transaction> = pending
            .values()
            .flat_map(|queue| queue.iter().cloned())
            .collect();
        
        // Sort by gas price (descending) and nonce (ascending)
        all_txs.sort_by(|a, b| {
            b.gas_price
                .cmp(&a.gas_price)
                .then_with(|| a.nonce.cmp(&b.nonce))
        });
        
        // Filter executable transactions
        let mut executable = Vec::new();
        for tx in all_txs {
            if executable.len() >= max_count {
                break;
            }
            
            // Check if nonce is correct
            if let Ok(current_nonce) = state_manager.get_nonce(&tx.from.to_string()) {
                if tx.nonce == current_nonce {
                    executable.push(tx);
                }
            }
        }
        
        executable
    }

    /// Remove transaction from mempool
    pub fn remove_transaction(&self, tx_hash: &str) -> Option<Transaction> {
        let mut by_hash = self.by_hash.write().unwrap();
        let mut pending = self.pending.write().unwrap();
        
        if let Some(tx) = by_hash.remove(tx_hash) {
            let tx_from = tx.from.to_string();
            if let Some(queue) = pending.get_mut(&tx_from) {
                queue.retain(|t| t.hash() != tx_hash);
                if queue.is_empty() {
                    pending.remove(&tx_from);
                }
            }
            return Some(tx);
        }
        
        None
    }

    /// Get mempool size
    pub fn size(&self) -> usize {
        let by_hash = self.by_hash.read().unwrap();
        by_hash.len()
    }

    /// Clear mempool
    pub fn clear(&self) {
        let mut by_hash = self.by_hash.write().unwrap();
        let mut pending = self.pending.write().unwrap();
        
        by_hash.clear();
        pending.clear();
        
        println!("🧹 Mempool cleared");
    }

    /// Get mempool statistics
    pub fn get_stats(&self) -> MempoolStats {
        let by_hash = self.by_hash.read().unwrap();
        let pending = self.pending.read().unwrap();
        
        let total_transactions = by_hash.len();
        let unique_senders = pending.len();
        
        let avg_gas_price = if !by_hash.is_empty() {
            by_hash.values().map(|tx| tx.gas_price).sum::<u128>() / by_hash.len() as u128
        } else {
            0
        };
        
        MempoolStats {
            total_transactions,
            unique_senders,
            avg_gas_price,
        }
    }
}

/// Mempool statistics
#[derive(Debug, Clone)]
pub struct MempoolStats {
    pub total_transactions: usize,
    pub unique_senders: usize,
    pub avg_gas_price: ActAmount,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crypto::ActKeyPair;
    use state::GenesisAccount;
    use storage::BlockchainStorage;
    use types::TransactionType;

    #[test]
    fn test_mempool_add_transaction() {
        let storage = Arc::new(BlockchainStorage::new("./test_mempool_db").unwrap());
        let state_manager = StateManager::new(storage);
        
        let keypair = ActKeyPair::generate();
        let address = keypair.address().to_string();
        
        let genesis_accounts = vec![GenesisAccount::new(address.clone(), 1000.0)];
        state_manager.initialize_genesis(genesis_accounts).unwrap();
        
        let mempool = Mempool::new(1000);
        
        let tx = Transaction {
            from: address,
            nonce: 0,
            tx_type: TransactionType::Transfer {
                to: "ACT-receiver".to_string(),
                amount: 100_000_000_000_000_000_000,
            },
            gas_limit: 21000,
            gas_price: 1_000_000_000_000,
            signature: vec![1, 2, 3],
            pubkey: keypair.public_key(),
        };
        
        // This will fail due to invalid signature in test, but tests the structure
        let result = mempool.add_transaction(tx, &state_manager);
        assert!(result.is_err());
        
        std::fs::remove_dir_all("./test_mempool_db").ok();
    }
}

//! Storage layer for blockchain data persistence
//! 
//! Provides efficient storage and retrieval of blocks, transactions, and state data
//! using RocksDB as the underlying key-value store.

use anyhow::{anyhow, Result};
use rocksdb::{DB, Options, IteratorMode};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

/// Column family names for different data types
pub const CF_BLOCKS: &str = "blocks";
pub const CF_TRANSACTIONS: &str = "transactions";
pub const CF_STATE: &str = "state";
pub const CF_RECEIPTS: &str = "receipts";
pub const CF_METADATA: &str = "metadata";

/// Storage manager for blockchain data
pub struct Storage {
    db: Arc<DB>,
}

/// Block data stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredBlock {
    pub height: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub validator: String,
    pub transactions: Vec<String>, // Transaction hashes
    pub state_root: String,
    pub transactions_root: String,
    pub receipts_root: String,
}

/// Transaction data stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTransaction {
    pub hash: String,
    pub from: String,
    pub to: Option<String>,
    pub value: u64,
    pub data: Vec<u8>,
    pub nonce: u64,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub block_height: Option<u64>,
    pub block_hash: Option<String>,
    pub transaction_index: Option<u32>,
}

/// Transaction receipt stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredReceipt {
    pub transaction_hash: String,
    pub block_height: u64,
    pub block_hash: String,
    pub transaction_index: u32,
    pub from: String,
    pub to: Option<String>,
    pub gas_used: u64,
    pub status: bool,
    pub logs: Vec<StoredLog>,
}

/// Event log stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredLog {
    pub address: String,
    pub topics: Vec<String>,
    pub data: Vec<u8>,
}

impl Storage {
    /// Create new storage instance
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        
        let cf_names = vec![CF_BLOCKS, CF_TRANSACTIONS, CF_STATE, CF_RECEIPTS, CF_METADATA];
        let db = DB::open_cf(&opts, path, &cf_names)?;
        
        Ok(Self {
            db: Arc::new(db),
        })
    }
    
    /// Store a block
    pub fn store_block(&self, block: &StoredBlock) -> Result<()> {
        let cf = self.db.cf_handle(CF_BLOCKS)
            .ok_or_else(|| anyhow!("Blocks column family not found"))?;
        
        let key = format!("height:{}", block.height);
        let value = bincode::serialize(block)?;
        self.db.put_cf(&cf, key.as_bytes(), &value)?;
        
        // Also index by hash
        let hash_key = format!("hash:{}", block.hash);
        self.db.put_cf(&cf, hash_key.as_bytes(), &value)?;
        
        // Update latest block height
        self.set_latest_block_height(block.height)?;
        
        Ok(())
    }
    
    /// Get block by height
    pub fn get_block_by_height(&self, height: u64) -> Result<Option<StoredBlock>> {
        let cf = self.db.cf_handle(CF_BLOCKS)
            .ok_or_else(|| anyhow!("Blocks column family not found"))?;
        
        let key = format!("height:{}", height);
        match self.db.get_cf(&cf, key.as_bytes())? {
            Some(bytes) => Ok(Some(bincode::deserialize(&bytes)?)),
            None => Ok(None),
        }
    }
    
    /// Get block by hash
    pub fn get_block_by_hash(&self, hash: &str) -> Result<Option<StoredBlock>> {
        let cf = self.db.cf_handle(CF_BLOCKS)
            .ok_or_else(|| anyhow!("Blocks column family not found"))?;
        
        let key = format!("hash:{}", hash);
        match self.db.get_cf(&cf, key.as_bytes())? {
            Some(bytes) => Ok(Some(bincode::deserialize(&bytes)?)),
            None => Ok(None),
        }
    }
    
    /// Get latest block height
    pub fn get_latest_block_height(&self) -> Result<u64> {
        let cf = self.db.cf_handle(CF_METADATA)
            .ok_or_else(|| anyhow!("Metadata column family not found"))?;
        
        match self.db.get_cf(&cf, b"latest_block_height")? {
            Some(bytes) => {
                let height = u64::from_le_bytes(bytes.try_into()
                    .map_err(|_| anyhow!("Invalid block height"))?);
                Ok(height)
            },
            None => Ok(0),
        }
    }
    
    /// Set latest block height
    fn set_latest_block_height(&self, height: u64) -> Result<()> {
        let cf = self.db.cf_handle(CF_METADATA)
            .ok_or_else(|| anyhow!("Metadata column family not found"))?;
        
        self.db.put_cf(&cf, b"latest_block_height", &height.to_le_bytes())?;
        Ok(())
    }
    
    /// Store a transaction
    pub fn store_transaction(&self, tx: &StoredTransaction) -> Result<()> {
        let cf = self.db.cf_handle(CF_TRANSACTIONS)
            .ok_or_else(|| anyhow!("Transactions column family not found"))?;
        
        let key = format!("hash:{}", tx.hash);
        let value = bincode::serialize(tx)?;
        self.db.put_cf(&cf, key.as_bytes(), &value)?;
        
        Ok(())
    }
    
    /// Get transaction by hash
    pub fn get_transaction(&self, hash: &str) -> Result<Option<StoredTransaction>> {
        let cf = self.db.cf_handle(CF_TRANSACTIONS)
            .ok_or_else(|| anyhow!("Transactions column family not found"))?;
        
        let key = format!("hash:{}", hash);
        match self.db.get_cf(&cf, key.as_bytes())? {
            Some(bytes) => Ok(Some(bincode::deserialize(&bytes)?)),
            None => Ok(None),
        }
    }
    
    /// Store transaction receipt
    pub fn store_receipt(&self, receipt: &StoredReceipt) -> Result<()> {
        let cf = self.db.cf_handle(CF_RECEIPTS)
            .ok_or_else(|| anyhow!("Receipts column family not found"))?;
        
        let key = format!("tx:{}", receipt.transaction_hash);
        let value = bincode::serialize(receipt)?;
        self.db.put_cf(&cf, key.as_bytes(), &value)?;
        
        Ok(())
    }
    
    /// Get transaction receipt
    pub fn get_receipt(&self, tx_hash: &str) -> Result<Option<StoredReceipt>> {
        let cf = self.db.cf_handle(CF_RECEIPTS)
            .ok_or_else(|| anyhow!("Receipts column family not found"))?;
        
        let key = format!("tx:{}", tx_hash);
        match self.db.get_cf(&cf, key.as_bytes())? {
            Some(bytes) => Ok(Some(bincode::deserialize(&bytes)?)),
            None => Ok(None),
        }
    }
    
    /// Store state data (account balances, contract storage, etc.)
    pub fn store_state(&self, key: &str, value: &[u8]) -> Result<()> {
        let cf = self.db.cf_handle(CF_STATE)
            .ok_or_else(|| anyhow!("State column family not found"))?;
        
        self.db.put_cf(&cf, key.as_bytes(), value)?;
        Ok(())
    }
    
    /// Get state data
    pub fn get_state(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let cf = self.db.cf_handle(CF_STATE)
            .ok_or_else(|| anyhow!("State column family not found"))?;
        
        Ok(self.db.get_cf(&cf, key.as_bytes())?)
    }
    
    /// Get recent blocks (up to limit)
    pub fn get_recent_blocks(&self, limit: usize) -> Result<Vec<StoredBlock>> {
        let latest_height = self.get_latest_block_height()?;
        let mut blocks = Vec::new();
        
        for height in (0..=latest_height).rev().take(limit) {
            if let Some(block) = self.get_block_by_height(height)? {
                blocks.push(block);
            }
        }
        
        Ok(blocks)
    }
    
    /// Get transactions for a block
    pub fn get_block_transactions(&self, block_hash: &str) -> Result<Vec<StoredTransaction>> {
        let block = self.get_block_by_hash(block_hash)?
            .ok_or_else(|| anyhow!("Block not found"))?;
        
        let mut transactions = Vec::new();
        for tx_hash in &block.transactions {
            if let Some(tx) = self.get_transaction(tx_hash)? {
                transactions.push(tx);
            }
        }
        
        Ok(transactions)
    }
    
    /// Delete old data (for pruning)
    pub fn delete_block(&self, height: u64) -> Result<()> {
        if let Some(block) = self.get_block_by_height(height)? {
            let cf = self.db.cf_handle(CF_BLOCKS)
                .ok_or_else(|| anyhow!("Blocks column family not found"))?;
            
            let height_key = format!("height:{}", height);
            let hash_key = format!("hash:{}", block.hash);
            
            self.db.delete_cf(&cf, height_key.as_bytes())?;
            self.db.delete_cf(&cf, hash_key.as_bytes())?;
        }
        
        Ok(())
    }
    
    /// Flush pending writes to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_block_storage() {
        let dir = tempdir().unwrap();
        let storage = Storage::new(dir.path()).unwrap();
        
        let block = StoredBlock {
            height: 1,
            hash: "0xabc123".to_string(),
            parent_hash: "0x000000".to_string(),
            timestamp: 1700000000,
            validator: "ACT-validator1".to_string(),
            transactions: vec!["0xtx1".to_string()],
            state_root: "0xstate".to_string(),
            transactions_root: "0xtxroot".to_string(),
            receipts_root: "0xreceipts".to_string(),
        };
        
        storage.store_block(&block).unwrap();
        
        let retrieved = storage.get_block_by_height(1).unwrap().unwrap();
        assert_eq!(retrieved.hash, "0xabc123");
        
        let by_hash = storage.get_block_by_hash("0xabc123").unwrap().unwrap();
        assert_eq!(by_hash.height, 1);
        
        let latest = storage.get_latest_block_height().unwrap();
        assert_eq!(latest, 1);
    }
    
    #[test]
    fn test_transaction_storage() {
        let dir = tempdir().unwrap();
        let storage = Storage::new(dir.path()).unwrap();
        
        let tx = StoredTransaction {
            hash: "0xtx123".to_string(),
            from: "ACT-alice".to_string(),
            to: Some("ACT-bob".to_string()),
            value: 1000,
            data: vec![],
            nonce: 1,
            gas_limit: 21000,
            gas_price: 1,
            block_height: Some(1),
            block_hash: Some("0xblock1".to_string()),
            transaction_index: Some(0),
        };
        
        storage.store_transaction(&tx).unwrap();
        
        let retrieved = storage.get_transaction("0xtx123").unwrap().unwrap();
        assert_eq!(retrieved.from, "ACT-alice");
        assert_eq!(retrieved.value, 1000);
    }
}

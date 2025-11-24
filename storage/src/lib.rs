use anyhow::{anyhow, Result};
use rocksdb::{DB, Options};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use types::{Action, BlockHeader};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredBlock {
    pub header: BlockHeader,
    pub action: Action,
    pub height: u64,
}

pub struct BlockchainStorage {
    db: Arc<DB>,
}

impl BlockchainStorage {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        
        let db = DB::open(&opts, path)?;
        
        Ok(Self {
            db: Arc::new(db),
        })
    }

    pub fn store_block(&self, block: &StoredBlock) -> Result<()> {
        let key = format!("block_{}", block.height);
        let value = serde_json::to_vec(block)?;
        
        self.db.put(key.as_bytes(), value)?;
        
        // Also store latest height
        self.db.put(b"latest_height", block.height.to_be_bytes())?;
        
        println!("💾 Stored block {} to database", block.height);
        Ok(())
    }

    pub fn get_block(&self, height: u64) -> Result<Option<StoredBlock>> {
        let key = format!("block_{}", height);
        
        match self.db.get(key.as_bytes())? {
            Some(data) => {
                let block: StoredBlock = serde_json::from_slice(&data)?;
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    pub fn get_latest_height(&self) -> Result<Option<u64>> {
        match self.db.get(b"latest_height")? {
            Some(data) => {
                if data.len() == 8 {
                    let height = u64::from_be_bytes(data.as_slice().try_into().unwrap());
                    Ok(Some(height))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    pub fn store_state(&self, key: &str, value: &[u8]) -> Result<()> {
        let state_key = format!("state_{}", key);
        self.db.put(state_key.as_bytes(), value)?;
        Ok(())
    }

    pub fn get_state(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let state_key = format!("state_{}", key);
        Ok(self.db.get(state_key.as_bytes())?)
    }

    pub fn get_block_count(&self) -> Result<u64> {
        match self.get_latest_height()? {
            Some(height) => Ok(height + 1),
            None => Ok(0),
        }
    }
}

pub fn store_block(header: &BlockHeader, action: &Action, height: u64) -> Result<()> {
    let storage = BlockchainStorage::new("./blockchain_data")?;
    
    let block = StoredBlock {
        header: header.clone(),
        action: action.clone(),
        height,
    };
    
    storage.store_block(&block)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_storage_operations() {
        let test_path = "./test_blockchain_data";
        
        // Clean up test directory
        let _ = fs::remove_dir_all(test_path);
        
        let storage = BlockchainStorage::new(test_path).unwrap();
        
        let action = Action {
            actor: "test_actor".to_string(),
            payload: vec![1, 2, 3],
            nonce: 1,
        };
        
        let header = BlockHeader {
            parent_hash: "parent".to_string(),
            action_hash: "hash".to_string(),
            actor_pubkey: "pubkey".to_string(),
            state_root: "state".to_string(),
            receipts_root: "receipts".to_string(),
            timestamp: 1234567890,
            validator_commitment: "validator1".to_string(),
            reward: 100,
        };
        
        let block = StoredBlock {
            header,
            action,
            height: 0,
        };
        
        storage.store_block(&block).unwrap();
        
        let retrieved = storage.get_block(0).unwrap();
        assert!(retrieved.is_some());
        
        let count = storage.get_block_count().unwrap();
        assert_eq!(count, 1);
        
        // Clean up
        let _ = fs::remove_dir_all(test_path);
    }
}

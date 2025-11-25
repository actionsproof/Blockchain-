use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use storage::BlockchainStorage;
use types::{Account, ActAmount, EventLog, Transaction, TransactionReceipt, TransactionType};

/// Cache entry with TTL
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Instant::now() + ttl,
        }
    }
    
    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// State manager for ACT Chain accounts and balances
pub struct StateManager {
    accounts: Arc<RwLock<HashMap<String, Account>>>,
    storage: Arc<BlockchainStorage>,
    receipts: Arc<RwLock<HashMap<String, TransactionReceipt>>>,  // tx_hash -> receipt
    balance_cache: Arc<RwLock<HashMap<String, CacheEntry<ActAmount>>>>,  // address -> balance cache
    nonce_cache: Arc<RwLock<HashMap<String, CacheEntry<u64>>>>,  // address -> nonce cache
}

impl StateManager {
    pub fn new(storage: Arc<BlockchainStorage>) -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            storage,
            receipts: Arc::new(RwLock::new(HashMap::new())),
            balance_cache: Arc::new(RwLock::new(HashMap::new())),
            nonce_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize genesis state with pre-funded accounts
    pub fn initialize_genesis(&self, genesis_accounts: Vec<GenesisAccount>) -> Result<()> {
        let mut accounts = self.accounts.write().unwrap();
        
        for genesis_account in genesis_accounts {
            let account = Account {
                address: genesis_account.address.clone(),
                balance: genesis_account.balance,
                nonce: 0,
                code_hash: None,
                storage_root: None,
            };
            
            accounts.insert(genesis_account.address.clone(), account.clone());
            
            // Persist to storage
            self.save_account_to_storage(&account)?;
            
            println!(
                "💰 Genesis account created: {} with {} ACT",
                genesis_account.address,
                genesis_account.balance as f64 / 1_000_000_000_000_000_000.0
            );
        }
        
        Ok(())
    }

    /// Get account by address
    pub fn get_account(&self, address: &str) -> Result<Account> {
        let accounts = self.accounts.read().unwrap();
        
        if let Some(account) = accounts.get(address) {
            return Ok(account.clone());
        }
        
        // Try loading from storage
        if let Some(account) = self.load_account_from_storage(address)? {
            drop(accounts);
            let mut accounts = self.accounts.write().unwrap();
            accounts.insert(address.to_string(), account.clone());
            return Ok(account);
        }
        
        // Create new empty account
        Ok(Account::new(address.to_string()))
    }

    /// Get account balance
    pub fn get_balance(&self, address: &str) -> Result<ActAmount> {
        // Check cache first
        {
            let cache = self.balance_cache.read().unwrap();
            if let Some(entry) = cache.get(address) {
                if !entry.is_expired() {
                    return Ok(entry.value);
                }
            }
        }
        
        // Cache miss or expired - get from state
        let account = self.get_account(address)?;
        let balance = account.balance;
        
        // Update cache (5 second TTL)
        {
            let mut cache = self.balance_cache.write().unwrap();
            cache.insert(address.to_string(), CacheEntry::new(balance, Duration::from_secs(5)));
        }
        
        Ok(balance)
    }

    /// Get account nonce
    pub fn get_nonce(&self, address: &str) -> Result<u64> {
        // Check cache first
        {
            let cache = self.nonce_cache.read().unwrap();
            if let Some(entry) = cache.get(address) {
                if !entry.is_expired() {
                    return Ok(entry.value);
                }
            }
        }
        
        // Cache miss or expired - get from state
        let account = self.get_account(address)?;
        let nonce = account.nonce;
        
        // Update cache (5 second TTL)
        {
            let mut cache = self.nonce_cache.write().unwrap();
            cache.insert(address.to_string(), CacheEntry::new(nonce, Duration::from_secs(5)));
        }
        
        Ok(nonce)
    }
    
    /// Invalidate cache for an address (call after state changes)
    fn invalidate_cache(&self, address: &str) {
        {
            let mut balance_cache = self.balance_cache.write().unwrap();
            balance_cache.remove(address);
        }
        {
            let mut nonce_cache = self.nonce_cache.write().unwrap();
            nonce_cache.remove(address);
        }
    }

    /// Transfer ACT between accounts
    pub fn transfer(&self, from: &str, to: &str, amount: ActAmount) -> Result<()> {
        let mut accounts = self.accounts.write().unwrap();
        
        // Get or create accounts
        let mut from_account = accounts
            .get(from)
            .cloned()
            .unwrap_or_else(|| Account::new(from.to_string()));
        
        let mut to_account = accounts
            .get(to)
            .cloned()
            .unwrap_or_else(|| Account::new(to.to_string()));
        
        // Check sufficient balance
        if from_account.balance < amount {
            return Err(anyhow!(
                "Insufficient balance: has {}, needs {}",
                from_account.balance,
                amount
            ));
        }
        
        // Perform transfer
        from_account.balance -= amount;
        to_account.balance += amount;
        
        // Update accounts
        accounts.insert(from.to_string(), from_account.clone());
        accounts.insert(to.to_string(), to_account.clone());
        
        // Persist to storage
        drop(accounts);
        self.save_account_to_storage(&from_account)?;
        self.save_account_to_storage(&to_account)?;
        
        // Invalidate cache
        self.invalidate_cache(from);
        self.invalidate_cache(to);
        
        println!("💸 Transfer: {} ACT from {} to {}", 
            amount as f64 / 1_000_000_000_000_000_000.0, from, to);
        
        Ok(())
    }

    /// Increment account nonce
    pub fn increment_nonce(&self, address: &str) -> Result<()> {
        let mut accounts = self.accounts.write().unwrap();
        
        let mut account = accounts
            .get(address)
            .cloned()
            .unwrap_or_else(|| Account::new(address.to_string()));
        
        account.nonce += 1;
        accounts.insert(address.to_string(), account.clone());
        
        drop(accounts);
        self.save_account_to_storage(&account)?;
        
        // Invalidate cache
        self.invalidate_cache(address);
        
        Ok(())
    }

    /// Deploy a contract
    pub fn deploy_contract(
        &self,
        deployer: &str,
        code: &[u8],
        initial_balance: ActAmount,
    ) -> Result<String> {
        let mut accounts = self.accounts.write().unwrap();
        
        // Calculate contract address from deployer + nonce
        let deployer_account = accounts
            .get(deployer)
            .ok_or_else(|| anyhow!("Deployer account not found"))?;
        
        let contract_address = self.calculate_contract_address(deployer, deployer_account.nonce);
        
        // Calculate code hash
        let mut hasher = Sha256::new();
        hasher.update(code);
        let code_hash = hex::encode(hasher.finalize());
        
        // Create contract account
        let contract_account = Account {
            address: contract_address.clone(),
            balance: initial_balance,
            nonce: 0,
            code_hash: Some(code_hash),
            storage_root: Some("empty_storage".to_string()),
        };
        
        accounts.insert(contract_address.clone(), contract_account.clone());
        
        drop(accounts);
        self.save_account_to_storage(&contract_account)?;
        
        // Store contract code
        self.storage.store_state(
            &format!("contract_code_{}", contract_address),
            code,
        )?;
        
        println!("📜 Contract deployed at: {}", contract_address);
        
        Ok(contract_address)
    }

    /// Calculate contract address
    fn calculate_contract_address(&self, deployer: &str, nonce: u64) -> String {
        let mut hasher = Sha256::new();
        hasher.update(deployer.as_bytes());
        hasher.update(nonce.to_le_bytes());
        let hash = hasher.finalize();
        
        let address_bytes = &hash[..20];
        let encoded = bs58::encode(address_bytes).into_string();
        
        format!("ACT-CONTRACT-{}", encoded)
    }

    /// Calculate state root (simplified Merkle root)
    pub fn calculate_state_root(&self) -> Result<String> {
        let accounts = self.accounts.read().unwrap();
        
        let mut account_hashes = Vec::new();
        for account in accounts.values() {
            let account_data = serde_json::to_vec(account)?;
            let mut hasher = Sha256::new();
            hasher.update(&account_data);
            account_hashes.push(hasher.finalize().to_vec());
        }
        
        account_hashes.sort();
        
        let mut hasher = Sha256::new();
        for hash in account_hashes {
            hasher.update(&hash);
        }
        
        Ok(hex::encode(hasher.finalize()))
    }

    /// Save account to persistent storage
    fn save_account_to_storage(&self, account: &Account) -> Result<()> {
        let key = format!("account_{}", account.address);
        let value = serde_json::to_vec(account)?;
        self.storage.store_state(&key, &value)?;
        Ok(())
    }

    /// Load account from persistent storage
    fn load_account_from_storage(&self, address: &str) -> Result<Option<Account>> {
        let key = format!("account_{}", address);
        if let Some(data) = self.storage.get_state(&key)? {
            let account: Account = serde_json::from_slice(&data)?;
            Ok(Some(account))
        } else {
            Ok(None)
        }
    }

    /// Get total supply
    pub fn get_total_supply(&self) -> ActAmount {
        let accounts = self.accounts.read().unwrap();
        accounts.values().map(|a| a.balance).sum()
    }
    
    /// Store transaction receipt with event logs
    pub fn store_receipt(&self, receipt: TransactionReceipt) -> Result<()> {
        let mut receipts = self.receipts.write().unwrap();
        let tx_hash = receipt.transaction_hash.clone();
        
        receipts.insert(tx_hash.clone(), receipt.clone());
        
        // Persist to storage
        let key = format!("receipt_{}", tx_hash);
        let value = serde_json::to_vec(&receipt)?;
        self.storage.store_state(&key, &value)?;
        
        // Index logs by contract address
        for log in &receipt.logs {
            self.index_event_log(&log)?;
        }
        
        Ok(())
    }
    
    /// Index event log for efficient querying
    fn index_event_log(&self, log: &EventLog) -> Result<()> {
        // Index by contract address
        let address_key = format!("logs_by_address_{}_{}", log.address, log.block_height);
        let existing = self.storage.get_state(&address_key)?;
        let mut log_list: Vec<EventLog> = if let Some(data) = existing {
            serde_json::from_slice(&data)?
        } else {
            Vec::new()
        };
        log_list.push(log.clone());
        self.storage.store_state(&address_key, &serde_json::to_vec(&log_list)?)?;
        
        // Index by topic (first topic only for efficiency)
        if let Some(topic) = log.topics.first() {
            let topic_key = format!("logs_by_topic_{}_{}", topic, log.block_height);
            let existing = self.storage.get_state(&topic_key)?;
            let mut log_list: Vec<EventLog> = if let Some(data) = existing {
                serde_json::from_slice(&data)?
            } else {
                Vec::new()
            };
            log_list.push(log.clone());
            self.storage.store_state(&topic_key, &serde_json::to_vec(&log_list)?)?;
        }
        
        Ok(())
    }
    
    /// Get transaction receipt
    pub fn get_receipt(&self, tx_hash: &str) -> Result<Option<TransactionReceipt>> {
        let receipts = self.receipts.read().unwrap();
        
        if let Some(receipt) = receipts.get(tx_hash) {
            return Ok(Some(receipt.clone()));
        }
        
        // Try loading from storage
        let key = format!("receipt_{}", tx_hash);
        if let Some(data) = self.storage.get_state(&key)? {
            let receipt: TransactionReceipt = serde_json::from_slice(&data)?;
            Ok(Some(receipt))
        } else {
            Ok(None)
        }
    }
    
    /// Query event logs by filter
    pub fn query_logs(
        &self,
        contract_address: Option<&str>,
        topics: Option<Vec<String>>,
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<EventLog>> {
        let mut all_logs = Vec::new();
        
        // Query by contract address
        if let Some(address) = contract_address {
            for block_height in from_block..=to_block {
                let key = format!("logs_by_address_{}_{}", address, block_height);
                if let Some(data) = self.storage.get_state(&key)? {
                    let logs: Vec<EventLog> = serde_json::from_slice(&data)?;
                    all_logs.extend(logs);
                }
            }
        }
        
        // Filter by topics if specified
        if let Some(topic_filters) = topics {
            all_logs.retain(|log| {
                topic_filters.iter().all(|filter_topic| {
                    log.topics.contains(filter_topic)
                })
            });
        }
        
        Ok(all_logs)
    }
}

/// Genesis account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisAccount {
    pub address: String,
    pub balance: ActAmount,
}

impl GenesisAccount {
    pub fn new(address: String, balance_in_act: f64) -> Self {
        Self {
            address,
            balance: (balance_in_act * 1_000_000_000_000_000_000.0) as ActAmount,
        }
    }
}

/// Gas configuration
#[derive(Debug, Clone)]
pub struct GasConfig {
    pub base_fee: ActAmount,
    pub transfer_cost: u64,
    pub contract_deploy_base: u64,
    pub contract_call_base: u64,
    pub storage_write_cost: u64,
}

impl Default for GasConfig {
    fn default() -> Self {
        Self {
            base_fee: 1_000_000_000_000, // 0.000001 ACT
            transfer_cost: 21000,
            contract_deploy_base: 53000,
            contract_call_base: 25000,
            storage_write_cost: 20000,
        }
    }
}

/// Calculate gas cost for a transaction
pub fn calculate_gas_cost(tx: &Transaction, gas_config: &GasConfig) -> u64 {
    match &tx.tx_type {
        TransactionType::Transfer { .. } => gas_config.transfer_cost,
        TransactionType::ContractDeploy { code, .. } => {
            gas_config.contract_deploy_base + (code.len() as u64 * 200)
        }
        TransactionType::ContractCall { args, .. } => {
            gas_config.contract_call_base + (args.len() as u64 * 100)
        }
        TransactionType::EthereumLegacy { data, .. } => {
            // Ethereum legacy transaction gas calculation
            gas_config.transfer_cost + (data.len() as u64 * 16)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_initialization() {
        let storage = Arc::new(BlockchainStorage::new("./test_state_db").unwrap());
        let state_manager = StateManager::new(storage);
        
        let genesis_accounts = vec![
            GenesisAccount::new("ACT-test1".to_string(), 1000000.0),
            GenesisAccount::new("ACT-test2".to_string(), 500000.0),
        ];
        
        state_manager.initialize_genesis(genesis_accounts).unwrap();
        
        let balance1 = state_manager.get_balance("ACT-test1").unwrap();
        assert!(balance1 > 0);
        
        std::fs::remove_dir_all("./test_state_db").ok();
    }

    #[test]
    fn test_transfer() {
        let storage = Arc::new(BlockchainStorage::new("./test_transfer_db").unwrap());
        let state_manager = StateManager::new(storage);
        
        let genesis_accounts = vec![GenesisAccount::new("ACT-sender".to_string(), 1000.0)];
        state_manager.initialize_genesis(genesis_accounts).unwrap();
        
        state_manager
            .transfer("ACT-sender", "ACT-receiver", 500_000_000_000_000_000_000)
            .unwrap();
        
        let sender_balance = state_manager.get_balance("ACT-sender").unwrap();
        let receiver_balance = state_manager.get_balance("ACT-receiver").unwrap();
        
        assert!(sender_balance < 1000_000_000_000_000_000_000);
        assert_eq!(receiver_balance, 500_000_000_000_000_000_000);
        
        std::fs::remove_dir_all("./test_transfer_db").ok();
    }
}

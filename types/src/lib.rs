use serde::{Serialize, Deserialize};

/// Native ACT token amount (in smallest unit: 1 ACT = 10^18 units)
pub type ActAmount = u128;

/// ACT token decimals (18 decimals like Ethereum)
pub const ACT_DECIMALS: u32 = 18;

/// Convert ACT to smallest units
pub fn from_act(act: f64) -> ActAmount {
    (act * 10_f64.powi(ACT_DECIMALS as i32)) as ActAmount
}

/// Convert smallest units to ACT
pub fn to_act(amount: ActAmount) -> f64 {
    amount as f64 / 10_f64.powi(ACT_DECIMALS as i32)
}

/// Transaction types in ACT Chain
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionType {
    Transfer {
        to: String,
        amount: ActAmount,
    },
    ContractDeploy {
        code: Vec<u8>,
        init_data: Vec<u8>,
    },
    ContractCall {
        contract: String,
        method: String,
        args: Vec<u8>,
    },
}

/// ACT Chain transaction
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub from: crypto::ActAddress,  // ACT address
    pub nonce: u64,                // Account nonce
    pub tx_type: TransactionType,
    pub gas_limit: u64,
    pub gas_price: ActAmount,
    pub signature: Vec<u8>,
    pub pubkey: Vec<u8>,
}

impl Transaction {
    pub fn hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let data = serde_json::to_vec(self).unwrap();
        let hash = Sha256::digest(&data);
        hex::encode(hash)
    }
}

/// Legacy action type (will be replaced by Transaction)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Action {
    pub actor: String,
    pub payload: Vec<u8>,
    pub nonce: u64,
}

/// Block header with ACT chain metadata
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    pub parent_hash: String,
    pub action_hash: String,        // Will become tx_root
    pub actor_pubkey: String,       // Block proposer
    pub state_root: String,
    pub receipts_root: String,
    pub timestamp: u64,
    pub validator_commitment: String,
    pub reward: ActAmount,          // Reward in ACT
    pub height: u64,
}

/// Account state in ACT Chain
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Account {
    pub address: String,
    pub balance: ActAmount,
    pub nonce: u64,
    pub code_hash: Option<String>,   // For contract accounts
    pub storage_root: Option<String>, // For contract storage
}

impl Account {
    pub fn new(address: String) -> Self {
        Self {
            address,
            balance: 0,
            nonce: 0,
            code_hash: None,
            storage_root: None,
        }
    }
    
    pub fn is_contract(&self) -> bool {
        self.code_hash.is_some()
    }
}

/// Event log emitted by smart contracts
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventLog {
    pub address: String,           // Contract address that emitted the event
    pub topics: Vec<String>,       // Indexed event parameters (up to 4 topics)
    pub data: Vec<u8>,             // Non-indexed event data
    pub block_height: u64,         // Block where event was emitted
    pub transaction_hash: String,  // Transaction that emitted the event
    pub log_index: u32,            // Index within the transaction
}

impl EventLog {
    pub fn new(
        address: String,
        topics: Vec<String>,
        data: Vec<u8>,
        block_height: u64,
        transaction_hash: String,
        log_index: u32,
    ) -> Self {
        Self {
            address,
            topics,
            data,
            block_height,
            transaction_hash,
            log_index,
        }
    }
}

/// Transaction receipt with execution details
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionReceipt {
    pub transaction_hash: String,
    pub block_height: u64,
    pub from: String,
    pub to: Option<String>,           // None for contract deployments
    pub contract_address: Option<String>, // For contract deployments
    pub status: bool,                 // true = success, false = failed
    pub gas_used: u64,
    pub logs: Vec<EventLog>,          // Event logs emitted
    pub logs_bloom: Option<Vec<u8>>,  // Bloom filter for efficient log searching
}

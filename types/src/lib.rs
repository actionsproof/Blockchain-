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

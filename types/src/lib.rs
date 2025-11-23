use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Action {
    pub actor: String,
    pub payload: Vec<u8>,
    pub nonce: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    pub parent_hash: String,
    pub action_hash: String,
    pub actor_pubkey: String,
    pub state_root: String,
    pub receipts_root: String,
    pub timestamp: u64,
    pub validator_commitment: String,
    pub reward: u64,
}

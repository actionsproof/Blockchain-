use anyhow::{anyhow, Result};
use bip39::{Language, Mnemonic};
use crypto::{ActAddress, ActKeyPair};
use serde::{Deserialize, Serialize};
use types::{ActAmount, Transaction, TransactionType};

/// ACT Chain wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActWallet {
    pub keypair: Option<ActKeyPair>,
    pub address: ActAddress,
    pub mnemonic: Option<String>,
}

impl ActWallet {
    /// Create a new wallet with random mnemonic
    pub fn new() -> Result<Self> {
        let mnemonic = Mnemonic::generate_in(Language::English, 12)?;
        let mnemonic_phrase = mnemonic.to_string();
        
        // Derive seed from mnemonic
        let seed = mnemonic.to_seed("");
        let keypair_seed: [u8; 32] = seed[..32].try_into()?;
        
        let keypair = ActKeyPair::from_seed(&keypair_seed);
        let address = keypair.address().clone();
        
        Ok(Self {
            keypair: Some(keypair),
            address,
            mnemonic: Some(mnemonic_phrase),
        })
    }
    
    /// Restore wallet from mnemonic phrase
    pub fn from_mnemonic(phrase: &str) -> Result<Self> {
        let mnemonic = Mnemonic::parse_in(Language::English, phrase)?;
        let seed = mnemonic.to_seed("");
        let keypair_seed: [u8; 32] = seed[..32].try_into()?;
        
        let keypair = ActKeyPair::from_seed(&keypair_seed);
        let address = keypair.address().clone();
        
        Ok(Self {
            keypair: Some(keypair),
            address,
            mnemonic: Some(phrase.to_string()),
        })
    }
    
    /// Create watch-only wallet from address
    pub fn from_address(address: &str) -> Result<Self> {
        let address = ActAddress::from_string(address)?;
        
        Ok(Self {
            keypair: None,
            address,
            mnemonic: None,
        })
    }
    
    /// Get wallet address
    pub fn address(&self) -> &ActAddress {
        &self.address
    }
    
    /// Sign a transaction
    pub fn sign_transaction(&self, mut tx: Transaction) -> Result<Transaction> {
        let keypair = self
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow!("Cannot sign with watch-only wallet"))?;
        
        // Serialize transaction data for signing (without signature)
        let tx_data = serde_json::to_vec(&(
            &tx.from,
            &tx.nonce,
            &tx.tx_type,
            &tx.gas_limit,
            &tx.gas_price,
        ))?;
        
        // Sign the transaction
        let signature = keypair.sign(&tx_data);
        tx.signature = signature;
        tx.pubkey = keypair.public_key();
        
        Ok(tx)
    }
    
    /// Create a transfer transaction
    pub fn create_transfer(
        &self,
        to: &str,
        amount: ActAmount,
        nonce: u64,
        gas_limit: u64,
        gas_price: ActAmount,
    ) -> Result<Transaction> {
        let tx = Transaction {
            from: self.address.to_string(),
            nonce,
            tx_type: TransactionType::Transfer {
                to: to.to_string(),
                amount,
            },
            gas_limit,
            gas_price,
            signature: vec![],
            pubkey: vec![],
        };
        
        self.sign_transaction(tx)
    }
    
    /// Create a contract deployment transaction
    pub fn create_contract_deploy(
        &self,
        code: Vec<u8>,
        init_data: Vec<u8>,
        nonce: u64,
        gas_limit: u64,
        gas_price: ActAmount,
    ) -> Result<Transaction> {
        let tx = Transaction {
            from: self.address.to_string(),
            nonce,
            tx_type: TransactionType::ContractDeploy { code, init_data },
            gas_limit,
            gas_price,
            signature: vec![],
            pubkey: vec![],
        };
        
        self.sign_transaction(tx)
    }
}

/// Utility functions for ACT amounts
pub mod act_units {
    use types::ActAmount;
    
    /// Convert ACT to smallest unit (1 ACT = 10^18 units)
    pub fn act_to_units(act: f64) -> ActAmount {
        (act * 1_000_000_000_000_000_000.0) as ActAmount
    }
    
    /// Convert smallest unit to ACT
    pub fn units_to_act(units: ActAmount) -> f64 {
        units as f64 / 1_000_000_000_000_000_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = ActWallet::new().unwrap();
        assert!(wallet.address.to_string().starts_with("ACT-"));
        assert!(wallet.mnemonic.is_some());
    }

    #[test]
    fn test_wallet_restore() {
        let wallet1 = ActWallet::new().unwrap();
        let mnemonic = wallet1.mnemonic.as_ref().unwrap();
        
        let wallet2 = ActWallet::from_mnemonic(mnemonic).unwrap();
        assert_eq!(wallet1.address, wallet2.address);
    }

    #[test]
    fn test_transaction_creation() {
        let wallet = ActWallet::new().unwrap();
        let tx = wallet.create_transfer(
            "ACT-recipient123",
            act_units::act_to_units(10.0),
            0,
            21000,
            act_units::act_to_units(0.000001),
        ).unwrap();
        
        assert_eq!(tx.from, wallet.address.to_string());
        assert!(!tx.signature.is_empty());
    }
}

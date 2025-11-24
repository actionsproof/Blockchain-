use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use wallet::ActWallet;

/// Encrypted wallet storage
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletStorage {
    #[serde(flatten)]
    pub wallet: ActWallet,
    pub encrypted: bool,
}

impl WalletStorage {
    /// Save wallet to file (with basic password encryption)
    pub fn save(&self, path: &Path, password: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        
        // Simple XOR encryption (not cryptographically secure, just basic protection)
        let encrypted = if self.encrypted {
            xor_encrypt(json.as_bytes(), password.as_bytes())
        } else {
            json.into_bytes()
        };
        
        std::fs::write(path, encrypted)?;
        Ok(())
    }
    
    /// Load wallet from file
    pub fn load(path: &Path, password: &str) -> Result<Self> {
        let encrypted = std::fs::read(path)?;
        
        // Try to decrypt
        let decrypted = xor_encrypt(&encrypted, password.as_bytes());
        
        // Try to parse as JSON
        let wallet: WalletStorage = serde_json::from_slice(&decrypted)
            .or_else(|_| serde_json::from_slice(&encrypted))?;
        
        Ok(wallet)
    }
    
    /// Create new wallet storage
    pub fn new(wallet: ActWallet, encrypted: bool) -> Self {
        Self { wallet, encrypted }
    }
}

/// Simple XOR encryption (for demonstration - use proper crypto in production)
fn xor_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    if key.is_empty() {
        return data.to_vec();
    }
    
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_encrypt_decrypt() {
        let data = b"Hello, World!";
        let key = b"secret";
        
        let encrypted = xor_encrypt(data, key);
        let decrypted = xor_encrypt(&encrypted, key);
        
        assert_eq!(data, &decrypted[..]);
    }
}

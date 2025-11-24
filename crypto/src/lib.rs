use anyhow::{anyhow, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::{rngs::OsRng, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// ACT Chain native address format
/// Format: ACT-{base58(pubkey_hash)}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActAddress(pub String);

impl ActAddress {
    pub fn from_pubkey(pubkey: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(pubkey);
        let hash = hasher.finalize();
        
        // Take first 20 bytes of hash (like Ethereum)
        let address_bytes = &hash[..20];
        let encoded = bs58::encode(address_bytes).into_string();
        
        ActAddress(format!("ACT-{}", encoded))
    }
    
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
    
    pub fn from_string(s: &str) -> Result<Self> {
        if !s.starts_with("ACT-") {
            return Err(anyhow!("Invalid ACT address format"));
        }
        Ok(ActAddress(s.to_string()))
    }
}

impl std::fmt::Display for ActAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// ACT Chain keypair
#[derive(Debug, Clone)]
pub struct ActKeyPair {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
    pub address: ActAddress,
}

impl ActKeyPair {
    /// Generate a new random keypair
    pub fn generate() -> Self {
        let signing_key = SigningKey::from_bytes(&OsRng.gen::<[u8; 32]>());
        let verifying_key = signing_key.verifying_key();
        let address = ActAddress::from_pubkey(verifying_key.as_bytes());
        
        Self {
            signing_key,
            verifying_key,
            address,
        }
    }
    
    /// Create keypair from seed bytes
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        let verifying_key = signing_key.verifying_key();
        let address = ActAddress::from_pubkey(verifying_key.as_bytes());
        
        Self {
            signing_key,
            verifying_key,
            address,
        }
    }
    
    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signing_key.sign(message).to_bytes().to_vec()
    }
    
    /// Get public key bytes
    pub fn public_key(&self) -> Vec<u8> {
        self.verifying_key.as_bytes().to_vec()
    }
    
    /// Get address
    pub fn address(&self) -> &ActAddress {
        &self.address
    }
}

/// Sign a message with a keypair
pub fn sign_message(keypair: &ActKeyPair, message: &[u8]) -> Vec<u8> {
    keypair.sign(message)
}

/// Verify a signature
pub fn verify_signature(pubkey_bytes: &[u8], message: &[u8], signature: &[u8]) -> Result<bool> {
    let verifying_key = VerifyingKey::from_bytes(
        pubkey_bytes
            .try_into()
            .map_err(|_| anyhow!("Invalid public key length"))?,
    )?;
    
    let signature = Signature::from_bytes(
        signature
            .try_into()
            .map_err(|_| anyhow!("Invalid signature length"))?,
    );
    
    Ok(verifying_key.verify(message, &signature).is_ok())
}

/// Hash data using SHA-256
pub fn hash_data(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = ActKeyPair::generate();
        assert!(keypair.address.0.starts_with("ACT-"));
    }

    #[test]
    fn test_signing_and_verification() {
        let keypair = ActKeyPair::generate();
        let message = b"Hello ACT Chain";
        
        let signature = sign_message(&keypair, message);
        let verified = verify_signature(&keypair.public_key(), message, &signature).unwrap();
        
        assert!(verified);
    }

    #[test]
    fn test_address_format() {
        let keypair = ActKeyPair::generate();
        let address = keypair.address();
        
        assert!(address.to_string().starts_with("ACT-"));
        assert!(address.to_string().len() > 10);
    }
}

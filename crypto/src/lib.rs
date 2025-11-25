use anyhow::{anyhow, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::{rngs::OsRng, Rng};
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tiny_keccak::{Hasher, Keccak};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActKeyPair {
    #[serde(with = "signing_key_serde")]
    pub signing_key: SigningKey,
    pub address: ActAddress,
}

mod signing_key_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(key: &SigningKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&key.to_bytes())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SigningKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = serde::Deserialize::deserialize(deserializer)?;
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(SigningKey::from_bytes(&arr))
    }
}

impl ActKeyPair {
    /// Generate a new random keypair
    pub fn generate() -> Self {
        let signing_key = SigningKey::from_bytes(&OsRng.gen::<[u8; 32]>());
        let verifying_key = signing_key.verifying_key();
        let address = ActAddress::from_pubkey(verifying_key.as_bytes());
        
        Self {
            signing_key,
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
            address,
        }
    }
    
    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signing_key.sign(message).to_bytes().to_vec()
    }
    
    /// Get public key bytes
    pub fn public_key(&self) -> Vec<u8> {
        self.signing_key.verifying_key().as_bytes().to_vec()
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

/// Ethereum-style address format (0x...)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EthAddress(pub String);

impl EthAddress {
    /// Create Ethereum address from public key (last 20 bytes of keccak256 hash)
    pub fn from_pubkey(pubkey: &[u8]) -> Self {
        let mut hasher = Keccak::v256();
        hasher.update(pubkey);
        let mut hash = [0u8; 32];
        hasher.finalize(&mut hash);
        
        // Take last 20 bytes
        let address_bytes = &hash[12..];
        let address = format!("0x{}", hex::encode(address_bytes));
        
        EthAddress(address)
    }
    
    pub fn from_string(s: &str) -> Result<Self> {
        if !s.starts_with("0x") || s.len() != 42 {
            return Err(anyhow!("Invalid Ethereum address format"));
        }
        Ok(EthAddress(s.to_lowercase()))
    }
    
    pub fn to_bytes(&self) -> Result<[u8; 20]> {
        let hex_str = self.0.trim_start_matches("0x");
        let bytes = hex::decode(hex_str)?;
        let mut arr = [0u8; 20];
        arr.copy_from_slice(&bytes);
        Ok(arr)
    }
}

impl std::fmt::Display for EthAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Ethereum-compatible keypair (secp256k1)
#[derive(Debug, Clone)]
pub struct EthKeyPair {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
    pub address: EthAddress,
}

impl EthKeyPair {
    /// Generate new Ethereum keypair
    pub fn generate() -> Result<Self> {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        
        // Get uncompressed public key (65 bytes, skip first byte)
        let pubkey_bytes = public_key.serialize_uncompressed();
        let pubkey_hash = &pubkey_bytes[1..]; // Skip 0x04 prefix
        
        let address = EthAddress::from_pubkey(pubkey_hash);
        
        Ok(Self {
            secret_key,
            public_key,
            address,
        })
    }
    
    /// Create from secret key bytes
    pub fn from_secret_bytes(secret_bytes: &[u8; 32]) -> Result<Self> {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(secret_bytes)?;
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        
        let pubkey_bytes = public_key.serialize_uncompressed();
        let pubkey_hash = &pubkey_bytes[1..];
        let address = EthAddress::from_pubkey(pubkey_hash);
        
        Ok(Self {
            secret_key,
            public_key,
            address,
        })
    }
    
    /// Sign message with Ethereum signature
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        let secp = Secp256k1::new();
        
        // Ethereum uses keccak256 for message hashing
        let mut hasher = Keccak::v256();
        hasher.update(message);
        let mut hash = [0u8; 32];
        hasher.finalize(&mut hash);
        
        let message = Message::from_digest(hash);
        let signature = secp.sign_ecdsa(&message, &self.secret_key);
        
        Ok(signature.serialize_compact().to_vec())
    }
    
    /// Get address
    pub fn address(&self) -> &EthAddress {
        &self.address
    }
}

/// Verify Ethereum signature
pub fn verify_eth_signature(
    pubkey_bytes: &[u8],
    message: &[u8],
    signature: &[u8],
) -> Result<bool> {
    let secp = Secp256k1::new();
    
    // Hash message with keccak256
    let mut hasher = Keccak::v256();
    hasher.update(message);
    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);
    
    let message = Message::from_digest(hash);
    let signature = secp256k1::ecdsa::Signature::from_compact(signature)?;
    let pubkey = PublicKey::from_slice(pubkey_bytes)?;
    
    Ok(secp.verify_ecdsa(&message, &signature, &pubkey).is_ok())
}

/// Keccak256 hash (Ethereum-style)
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    hasher.update(data);
    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);
    hash
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

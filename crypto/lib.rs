//! Cryptographic Utilities
//! 
//! Provides signing, verification, hashing, and key derivation functions
//! for both Ed25519 (native ACT) and secp256k1 (Ethereum compatibility).

use anyhow::{anyhow, Result};
use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey, Signature as Ed25519Signature};
use secp256k1::{Secp256k1, Message as Secp256k1Message, SecretKey, PublicKey};
use secp256k1::ecdsa::Signature as Secp256k1Signature;
use sha2::{Sha256, Digest};
use sha3::{Keccak256};
use ripemd::{Ripemd160};
use rand::rngs::OsRng;

/// Sign a message using Ed25519
pub fn sign_message_ed25519(private_key: &[u8], message: &[u8]) -> Result<Vec<u8>> {
    if private_key.len() != 32 {
        return Err(anyhow!("Invalid Ed25519 private key length"));
    }
    
    let secret = SigningKey::from_bytes(private_key.try_into()?);
    let signature = secret.sign(message);
    
    Ok(signature.to_bytes().to_vec())
}

/// Verify an Ed25519 signature
pub fn verify_signature_ed25519(public_key: &[u8], message: &[u8], signature: &[u8]) -> Result<bool> {
    if public_key.len() != 32 {
        return Err(anyhow!("Invalid Ed25519 public key length"));
    }
    
    if signature.len() != 64 {
        return Err(anyhow!("Invalid Ed25519 signature length"));
    }
    
    let public = VerifyingKey::from_bytes(public_key.try_into()?)?;
    let sig = Ed25519Signature::from_bytes(signature.try_into()?);
    
    Ok(public.verify(message, &sig).is_ok())
}

/// Sign a message using secp256k1 (Ethereum style)
pub fn sign_message_secp256k1(private_key: &[u8], message: &[u8]) -> Result<Vec<u8>> {
    if private_key.len() != 32 {
        return Err(anyhow!("Invalid secp256k1 private key length"));
    }
    
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(private_key)?;
    
    // Hash message with Keccak256 (Ethereum style)
    let message_hash = keccak256(message);
    let message = Secp256k1Message::from_digest_slice(&message_hash)?;
    
    let signature = secp.sign_ecdsa(&message, &secret_key);
    
    Ok(signature.serialize_compact().to_vec())
}

/// Verify a secp256k1 signature
pub fn verify_signature_secp256k1(public_key: &[u8], message: &[u8], signature: &[u8]) -> Result<bool> {
    if signature.len() != 64 {
        return Err(anyhow!("Invalid secp256k1 signature length"));
    }
    
    let secp = Secp256k1::new();
    let public = PublicKey::from_slice(public_key)?;
    
    // Hash message with Keccak256
    let message_hash = keccak256(message);
    let message = Secp256k1Message::from_digest_slice(&message_hash)?;
    
    let sig = Secp256k1Signature::from_compact(signature)?;
    
    Ok(secp.verify_ecdsa(&message, &sig, &public).is_ok())
}

/// Generate Ed25519 keypair
pub fn generate_ed25519_keypair() -> (Vec<u8>, Vec<u8>) {
    let mut rng = OsRng;
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();
    
    (
        signing_key.to_bytes().to_vec(),
        verifying_key.to_bytes().to_vec(),
    )
}

/// Generate secp256k1 keypair
pub fn generate_secp256k1_keypair() -> Result<(Vec<u8>, Vec<u8>)> {
    let secp = Secp256k1::new();
    let mut rng = OsRng;
    let (secret_key, public_key) = secp.generate_keypair(&mut rng);
    
    Ok((
        secret_key.secret_bytes().to_vec(),
        public_key.serialize().to_vec(),
    ))
}

/// SHA-256 hash
pub fn sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Double SHA-256 hash (Bitcoin style)
pub fn double_sha256(data: &[u8]) -> Vec<u8> {
    sha256(&sha256(data))
}

/// Keccak-256 hash (Ethereum style)
pub fn keccak256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// RIPEMD-160 hash
pub fn ripemd160(data: &[u8]) -> Vec<u8> {
    let mut hasher = Ripemd160::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Hash160 (SHA-256 + RIPEMD-160, Bitcoin style)
pub fn hash160(data: &[u8]) -> Vec<u8> {
    ripemd160(&sha256(data))
}

/// Derive Ethereum address from public key
pub fn derive_ethereum_address(public_key: &[u8]) -> Result<String> {
    // For secp256k1 public key (65 bytes uncompressed or 33 bytes compressed)
    let public_key = if public_key.len() == 65 {
        &public_key[1..] // Remove 0x04 prefix
    } else if public_key.len() == 33 {
        // Decompress if needed
        let secp = Secp256k1::new();
        let pk = PublicKey::from_slice(public_key)?;
        let uncompressed = pk.serialize_uncompressed();
        return Ok(format!("0x{}", hex::encode(&keccak256(&uncompressed[1..])[12..])));
    } else {
        return Err(anyhow!("Invalid public key length"));
    };
    
    // Take last 20 bytes of Keccak-256 hash
    let hash = keccak256(public_key);
    let address = &hash[12..];
    
    Ok(format!("0x{}", hex::encode(address)))
}

/// Derive ACT address from Ed25519 public key
pub fn derive_act_address(public_key: &[u8]) -> Result<String> {
    if public_key.len() != 32 {
        return Err(anyhow!("Invalid Ed25519 public key length"));
    }
    
    // Hash public key
    let hash = hash160(public_key);
    
    // Base58 encode with ACT prefix
    let encoded = bs58::encode(&hash).into_string();
    
    Ok(format!("ACT-{}", encoded))
}

/// PBKDF2 key derivation
pub fn pbkdf2_derive(password: &[u8], salt: &[u8], iterations: u32, key_length: usize) -> Vec<u8> {
    use pbkdf2::pbkdf2_hmac;
    
    let mut key = vec![0u8; key_length];
    pbkdf2_hmac::<Sha256>(password, salt, iterations, &mut key);
    key
}

/// Generate random bytes
pub fn random_bytes(length: usize) -> Vec<u8> {
    use rand::RngCore;
    let mut rng = OsRng;
    let mut bytes = vec![0u8; length];
    rng.fill_bytes(&mut bytes);
    bytes
}

/// Verify address checksum (Ethereum EIP-55)
pub fn verify_ethereum_checksum(address: &str) -> Result<bool> {
    if !address.starts_with("0x") || address.len() != 42 {
        return Err(anyhow!("Invalid Ethereum address format"));
    }
    
    let addr = &address[2..];
    let hash = hex::encode(keccak256(addr.to_lowercase().as_bytes()));
    
    for (i, c) in addr.chars().enumerate() {
        if c.is_ascii_alphabetic() {
            let hash_char = hash.chars().nth(i).unwrap();
            let should_be_uppercase = hash_char.to_digit(16).unwrap() >= 8;
            
            if c.is_uppercase() != should_be_uppercase {
                return Ok(false);
            }
        }
    }
    
    Ok(true)
}

/// Apply EIP-55 checksum to Ethereum address
pub fn apply_ethereum_checksum(address: &str) -> Result<String> {
    if !address.starts_with("0x") || address.len() != 42 {
        return Err(anyhow!("Invalid Ethereum address format"));
    }
    
    let addr = address[2..].to_lowercase();
    let hash = hex::encode(keccak256(addr.as_bytes()));
    
    let mut checksummed = String::from("0x");
    for (i, c) in addr.chars().enumerate() {
        let hash_char = hash.chars().nth(i).unwrap();
        if c.is_ascii_alphabetic() && hash_char.to_digit(16).unwrap() >= 8 {
            checksummed.push(c.to_uppercase().next().unwrap());
        } else {
            checksummed.push(c);
        }
    }
    
    Ok(checksummed)
}

// Legacy compatibility functions
pub fn sign_message(key: &[u8], message: &[u8]) -> Result<Vec<u8>> {
    sign_message_ed25519(key, message)
}

pub fn verify_signature(public_key: &[u8], message: &[u8], signature: &[u8]) -> Result<bool> {
    verify_signature_ed25519(public_key, message, signature)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ed25519_signing() {
        let (private_key, public_key) = generate_ed25519_keypair();
        let message = b"Hello, ACT Chain!";
        
        let signature = sign_message_ed25519(&private_key, message).unwrap();
        let valid = verify_signature_ed25519(&public_key, message, &signature).unwrap();
        
        assert!(valid);
        
        // Invalid signature should fail
        let invalid = verify_signature_ed25519(&public_key, b"Different message", &signature).unwrap();
        assert!(!invalid);
    }
    
    #[test]
    fn test_secp256k1_signing() {
        let (private_key, public_key) = generate_secp256k1_keypair().unwrap();
        let message = b"Hello, Ethereum!";
        
        let signature = sign_message_secp256k1(&private_key, message).unwrap();
        let valid = verify_signature_secp256k1(&public_key, message, &signature).unwrap();
        
        assert!(valid);
    }
    
    #[test]
    fn test_hashing() {
        let data = b"test data";
        
        let sha256_hash = sha256(data);
        assert_eq!(sha256_hash.len(), 32);
        
        let keccak_hash = keccak256(data);
        assert_eq!(keccak_hash.len(), 32);
        
        let hash160 = hash160(data);
        assert_eq!(hash160.len(), 20);
    }
    
    #[test]
    fn test_ethereum_address_derivation() {
        let (_, public_key) = generate_secp256k1_keypair().unwrap();
        let address = derive_ethereum_address(&public_key).unwrap();
        
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42);
    }
    
    #[test]
    fn test_act_address_derivation() {
        let (_, public_key) = generate_ed25519_keypair();
        let address = derive_act_address(&public_key).unwrap();
        
        assert!(address.starts_with("ACT-"));
    }
    
    #[test]
    fn test_ethereum_checksum() {
        let address = "0x5aAeb6053f3E94C9b9A09f33669435E7Ef1BeAed";
        let valid = verify_ethereum_checksum(address).unwrap();
        assert!(valid);
        
        let checksummed = apply_ethereum_checksum("0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed").unwrap();
        assert_eq!(checksummed, address);
    }
    
    #[test]
    fn test_random_bytes() {
        let bytes1 = random_bytes(32);
        let bytes2 = random_bytes(32);
        
        assert_eq!(bytes1.len(), 32);
        assert_eq!(bytes2.len(), 32);
        assert_ne!(bytes1, bytes2); // Should be different
    }
}

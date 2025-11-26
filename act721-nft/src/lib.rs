/// ACT-721 NFT Standard
/// 
/// ERC-721 compatible Non-Fungible Token standard for ACT Chain
/// Supports:
/// - Unique token ownership
/// - Transfer and approval mechanisms
/// - Metadata (name, symbol, token URI)
/// - Enumeration (totalSupply, tokenByIndex, tokenOfOwnerByIndex)
/// - Marketplace compatibility

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// NFT Token metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub name: String,
    pub description: String,
    pub image: String,          // URI to image
    pub attributes: Vec<TokenAttribute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAttribute {
    pub trait_type: String,
    pub value: String,
}

/// ACT-721 NFT Contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Act721Contract {
    // Contract metadata
    pub name: String,
    pub symbol: String,
    pub owner: String,
    
    // Token ownership
    pub owners: HashMap<u64, String>,  // token_id -> owner
    
    // Balances
    pub balances: HashMap<String, u64>,  // owner -> count
    
    // Approvals
    pub token_approvals: HashMap<u64, String>,  // token_id -> approved_address
    pub operator_approvals: HashMap<String, HashMap<String, bool>>,  // owner -> (operator -> approved)
    
    // Metadata
    pub token_uris: HashMap<u64, String>,  // token_id -> URI
    pub base_uri: String,
    
    // Enumeration
    pub all_tokens: Vec<u64>,  // All token IDs
    pub owner_tokens: HashMap<String, Vec<u64>>,  // owner -> token_ids
    
    // State
    pub next_token_id: u64,
    pub total_supply: u64,
}

impl Act721Contract {
    pub fn new(name: String, symbol: String, owner: String) -> Self {
        Self {
            name,
            symbol,
            owner,
            owners: HashMap::new(),
            balances: HashMap::new(),
            token_approvals: HashMap::new(),
            operator_approvals: HashMap::new(),
            token_uris: HashMap::new(),
            base_uri: String::new(),
            all_tokens: Vec::new(),
            owner_tokens: HashMap::new(),
            next_token_id: 1,
            total_supply: 0,
        }
    }
    
    /// Get total supply of tokens
    pub fn total_supply(&self) -> u64 {
        self.total_supply
    }
    
    /// Get owner of token
    pub fn owner_of(&self, token_id: u64) -> Option<String> {
        self.owners.get(&token_id).cloned()
    }
    
    /// Get balance of owner
    pub fn balance_of(&self, owner: &str) -> u64 {
        *self.balances.get(owner).unwrap_or(&0)
    }
    
    /// Transfer token
    pub fn transfer_from(
        &mut self,
        from: &str,
        to: &str,
        token_id: u64,
        caller: &str,
    ) -> Result<(), String> {
        // Check ownership
        let owner = self.owner_of(token_id).ok_or("Token does not exist")?;
        if owner != from {
            return Err("From address is not owner".to_string());
        }
        
        // Check authorization
        if !self.is_approved_or_owner(caller, token_id) {
            return Err("Caller is not owner nor approved".to_string());
        }
        
        // Clear approval
        self.token_approvals.remove(&token_id);
        
        // Update balances
        if let Some(balance) = self.balances.get_mut(from) {
            *balance -= 1;
        }
        *self.balances.entry(to.to_string()).or_insert(0) += 1;
        
        // Update ownership
        self.owners.insert(token_id, to.to_string());
        
        // Update enumeration
        if let Some(tokens) = self.owner_tokens.get_mut(from) {
            tokens.retain(|&id| id != token_id);
        }
        self.owner_tokens.entry(to.to_string()).or_default().push(token_id);
        
        Ok(())
    }
    
    /// Safe transfer with receiver check
    pub fn safe_transfer_from(
        &mut self,
        from: &str,
        to: &str,
        token_id: u64,
        caller: &str,
    ) -> Result<(), String> {
        // In a full implementation, would check if receiver can handle NFTs
        self.transfer_from(from, to, token_id, caller)
    }
    
    /// Approve address to transfer token
    pub fn approve(&mut self, to: &str, token_id: u64, caller: &str) -> Result<(), String> {
        let owner = self.owner_of(token_id).ok_or("Token does not exist")?;
        
        if caller != owner && !self.is_approved_for_all(&owner, caller) {
            return Err("Caller is not owner nor operator".to_string());
        }
        
        self.token_approvals.insert(token_id, to.to_string());
        Ok(())
    }
    
    /// Get approved address for token
    pub fn get_approved(&self, token_id: u64) -> Option<String> {
        self.token_approvals.get(&token_id).cloned()
    }
    
    /// Set approval for all tokens
    pub fn set_approval_for_all(&mut self, operator: &str, approved: bool, caller: &str) {
        self.operator_approvals
            .entry(caller.to_string())
            .or_default()
            .insert(operator.to_string(), approved);
    }
    
    /// Check if operator is approved for all tokens of owner
    pub fn is_approved_for_all(&self, owner: &str, operator: &str) -> bool {
        self.operator_approvals
            .get(owner)
            .and_then(|ops| ops.get(operator))
            .copied()
            .unwrap_or(false)
    }
    
    /// Mint new token
    pub fn mint(&mut self, to: &str, caller: &str) -> Result<u64, String> {
        if caller != self.owner {
            return Err("Only owner can mint".to_string());
        }
        
        let token_id = self.next_token_id;
        self.next_token_id += 1;
        
        self.owners.insert(token_id, to.to_string());
        *self.balances.entry(to.to_string()).or_insert(0) += 1;
        
        self.all_tokens.push(token_id);
        self.owner_tokens.entry(to.to_string()).or_default().push(token_id);
        
        self.total_supply += 1;
        
        Ok(token_id)
    }
    
    /// Mint token with URI
    pub fn mint_with_uri(
        &mut self,
        to: &str,
        uri: String,
        caller: &str,
    ) -> Result<u64, String> {
        let token_id = self.mint(to, caller)?;
        self.token_uris.insert(token_id, uri);
        Ok(token_id)
    }
    
    /// Burn token
    pub fn burn(&mut self, token_id: u64, caller: &str) -> Result<(), String> {
        let owner = self.owner_of(token_id).ok_or("Token does not exist")?;
        
        if !self.is_approved_or_owner(caller, token_id) {
            return Err("Caller is not owner nor approved".to_string());
        }
        
        // Clear approval
        self.token_approvals.remove(&token_id);
        
        // Update balance
        if let Some(balance) = self.balances.get_mut(&owner) {
            *balance -= 1;
        }
        
        // Remove ownership
        self.owners.remove(&token_id);
        
        // Update enumeration
        self.all_tokens.retain(|&id| id != token_id);
        if let Some(tokens) = self.owner_tokens.get_mut(&owner) {
            tokens.retain(|&id| id != token_id);
        }
        
        // Remove metadata
        self.token_uris.remove(&token_id);
        
        self.total_supply -= 1;
        
        Ok(())
    }
    
    /// Get token URI
    pub fn token_uri(&self, token_id: u64) -> Option<String> {
        if !self.owners.contains_key(&token_id) {
            return None;
        }
        
        if let Some(uri) = self.token_uris.get(&token_id) {
            Some(uri.clone())
        } else if !self.base_uri.is_empty() {
            Some(format!("{}{}", self.base_uri, token_id))
        } else {
            None
        }
    }
    
    /// Set base URI for all tokens
    pub fn set_base_uri(&mut self, base_uri: String, caller: &str) -> Result<(), String> {
        if caller != self.owner {
            return Err("Only owner can set base URI".to_string());
        }
        self.base_uri = base_uri;
        Ok(())
    }
    
    /// Get token by index (enumeration)
    pub fn token_by_index(&self, index: usize) -> Option<u64> {
        self.all_tokens.get(index).copied()
    }
    
    /// Get token of owner by index (enumeration)
    pub fn token_of_owner_by_index(&self, owner: &str, index: usize) -> Option<u64> {
        self.owner_tokens.get(owner)?.get(index).copied()
    }
    
    /// Get all tokens owned by address
    pub fn tokens_of_owner(&self, owner: &str) -> Vec<u64> {
        self.owner_tokens.get(owner).cloned().unwrap_or_default()
    }
    
    /// Check if caller is owner or approved
    fn is_approved_or_owner(&self, caller: &str, token_id: u64) -> bool {
        if let Some(owner) = self.owner_of(token_id) {
            if caller == owner {
                return true;
            }
            if let Some(approved) = self.get_approved(token_id) {
                if caller == approved {
                    return true;
                }
            }
            if self.is_approved_for_all(&owner, caller) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nft_creation() {
        let nft = Act721Contract::new(
            "Test NFT".to_string(),
            "TNFT".to_string(),
            "owner1".to_string(),
        );
        
        assert_eq!(nft.name, "Test NFT");
        assert_eq!(nft.symbol, "TNFT");
        assert_eq!(nft.total_supply(), 0);
    }

    #[test]
    fn test_mint() {
        let mut nft = Act721Contract::new(
            "Test NFT".to_string(),
            "TNFT".to_string(),
            "owner1".to_string(),
        );
        
        let token_id = nft.mint("user1", "owner1").unwrap();
        assert_eq!(token_id, 1);
        assert_eq!(nft.total_supply(), 1);
        assert_eq!(nft.owner_of(1), Some("user1".to_string()));
        assert_eq!(nft.balance_of("user1"), 1);
    }

    #[test]
    fn test_mint_unauthorized() {
        let mut nft = Act721Contract::new(
            "Test NFT".to_string(),
            "TNFT".to_string(),
            "owner1".to_string(),
        );
        
        let result = nft.mint("user1", "user2");
        assert!(result.is_err());
    }

    #[test]
    fn test_transfer() {
        let mut nft = Act721Contract::new(
            "Test NFT".to_string(),
            "TNFT".to_string(),
            "owner1".to_string(),
        );
        
        nft.mint("user1", "owner1").unwrap();
        
        let result = nft.transfer_from("user1", "user2", 1, "user1");
        assert!(result.is_ok());
        assert_eq!(nft.owner_of(1), Some("user2".to_string()));
        assert_eq!(nft.balance_of("user1"), 0);
        assert_eq!(nft.balance_of("user2"), 1);
    }

    #[test]
    fn test_approve_and_transfer() {
        let mut nft = Act721Contract::new(
            "Test NFT".to_string(),
            "TNFT".to_string(),
            "owner1".to_string(),
        );
        
        nft.mint("user1", "owner1").unwrap();
        nft.approve("user2", 1, "user1").unwrap();
        
        assert_eq!(nft.get_approved(1), Some("user2".to_string()));
        
        let result = nft.transfer_from("user1", "user3", 1, "user2");
        assert!(result.is_ok());
        assert_eq!(nft.owner_of(1), Some("user3".to_string()));
    }

    #[test]
    fn test_operator_approval() {
        let mut nft = Act721Contract::new(
            "Test NFT".to_string(),
            "TNFT".to_string(),
            "owner1".to_string(),
        );
        
        nft.mint("user1", "owner1").unwrap();
        nft.set_approval_for_all("operator1", true, "user1");
        
        assert!(nft.is_approved_for_all("user1", "operator1"));
        
        let result = nft.transfer_from("user1", "user2", 1, "operator1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_burn() {
        let mut nft = Act721Contract::new(
            "Test NFT".to_string(),
            "TNFT".to_string(),
            "owner1".to_string(),
        );
        
        nft.mint("user1", "owner1").unwrap();
        assert_eq!(nft.total_supply(), 1);
        
        nft.burn(1, "user1").unwrap();
        assert_eq!(nft.total_supply(), 0);
        assert_eq!(nft.owner_of(1), None);
    }

    #[test]
    fn test_token_uri() {
        let mut nft = Act721Contract::new(
            "Test NFT".to_string(),
            "TNFT".to_string(),
            "owner1".to_string(),
        );
        
        nft.set_base_uri("https://example.com/token/".to_string(), "owner1").unwrap();
        nft.mint("user1", "owner1").unwrap();
        
        assert_eq!(
            nft.token_uri(1),
            Some("https://example.com/token/1".to_string())
        );
    }

    #[test]
    fn test_enumeration() {
        let mut nft = Act721Contract::new(
            "Test NFT".to_string(),
            "TNFT".to_string(),
            "owner1".to_string(),
        );
        
        nft.mint("user1", "owner1").unwrap();
        nft.mint("user1", "owner1").unwrap();
        nft.mint("user2", "owner1").unwrap();
        
        assert_eq!(nft.token_by_index(0), Some(1));
        assert_eq!(nft.token_by_index(1), Some(2));
        assert_eq!(nft.token_by_index(2), Some(3));
        
        assert_eq!(nft.token_of_owner_by_index("user1", 0), Some(1));
        assert_eq!(nft.token_of_owner_by_index("user1", 1), Some(2));
        assert_eq!(nft.token_of_owner_by_index("user2", 0), Some(3));
        
        let user1_tokens = nft.tokens_of_owner("user1");
        assert_eq!(user1_tokens.len(), 2);
        assert!(user1_tokens.contains(&1));
        assert!(user1_tokens.contains(&2));
    }
}

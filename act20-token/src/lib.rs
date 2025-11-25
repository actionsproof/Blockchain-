use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ACT-20 Token Standard - ERC-20 compatible fungible token for ACT Chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Act20Token {
    /// Token name (e.g. "My Token")
    pub name: String,
    /// Token symbol (e.g. "MTK")
    pub symbol: String,
    /// Number of decimals (typically 18)
    pub decimals: u8,
    /// Total supply
    pub total_supply: u128,
    /// Balances mapping: address -> balance
    pub balances: HashMap<String, u128>,
    /// Allowances mapping: owner -> (spender -> amount)
    pub allowances: HashMap<String, HashMap<String, u128>>,
    /// Token owner (can mint/burn if configured)
    pub owner: String,
    /// Whether minting is allowed
    pub mintable: bool,
    /// Whether burning is allowed
    pub burnable: bool,
}

impl Act20Token {
    /// Create a new token
    pub fn new(
        name: String,
        symbol: String,
        decimals: u8,
        initial_supply: u128,
        owner: String,
        mintable: bool,
        burnable: bool,
    ) -> Self {
        let mut balances = HashMap::new();
        balances.insert(owner.clone(), initial_supply);

        Self {
            name,
            symbol,
            decimals,
            total_supply: initial_supply,
            balances,
            allowances: HashMap::new(),
            owner,
            mintable,
            burnable,
        }
    }

    /// Get balance of an address
    pub fn balance_of(&self, address: &str) -> u128 {
        *self.balances.get(address).unwrap_or(&0)
    }

    /// Transfer tokens from sender to recipient
    pub fn transfer(&mut self, from: &str, to: &str, amount: u128) -> Result<(), String> {
        if from == to {
            return Err("Cannot transfer to self".to_string());
        }

        let from_balance = self.balance_of(from);
        if from_balance < amount {
            return Err(format!("Insufficient balance: {} < {}", from_balance, amount));
        }

        self.balances.insert(from.to_string(), from_balance - amount);
        
        let to_balance = self.balance_of(to);
        self.balances.insert(to.to_string(), to_balance + amount);

        Ok(())
    }

    /// Approve spender to spend amount on behalf of owner
    pub fn approve(&mut self, owner: &str, spender: &str, amount: u128) -> Result<(), String> {
        if owner == spender {
            return Err("Cannot approve self".to_string());
        }

        self.allowances
            .entry(owner.to_string())
            .or_insert_with(HashMap::new)
            .insert(spender.to_string(), amount);

        Ok(())
    }

    /// Get allowance for spender from owner
    pub fn allowance(&self, owner: &str, spender: &str) -> u128 {
        self.allowances
            .get(owner)
            .and_then(|allowances| allowances.get(spender))
            .copied()
            .unwrap_or(0)
    }

    /// Transfer tokens from owner to recipient using allowance
    pub fn transfer_from(
        &mut self,
        spender: &str,
        from: &str,
        to: &str,
        amount: u128,
    ) -> Result<(), String> {
        let current_allowance = self.allowance(from, spender);
        if current_allowance < amount {
            return Err(format!(
                "Insufficient allowance: {} < {}",
                current_allowance, amount
            ));
        }

        // Reduce allowance
        self.allowances
            .get_mut(from)
            .unwrap()
            .insert(spender.to_string(), current_allowance - amount);

        // Perform transfer
        self.transfer(from, to, amount)?;

        Ok(())
    }

    /// Mint new tokens (only if mintable and caller is owner)
    pub fn mint(&mut self, caller: &str, to: &str, amount: u128) -> Result<(), String> {
        if !self.mintable {
            return Err("Token is not mintable".to_string());
        }

        if caller != self.owner {
            return Err("Only owner can mint".to_string());
        }

        let to_balance = self.balance_of(to);
        self.balances.insert(to.to_string(), to_balance + amount);
        self.total_supply += amount;

        Ok(())
    }

    /// Burn tokens from an address
    pub fn burn(&mut self, from: &str, amount: u128) -> Result<(), String> {
        if !self.burnable {
            return Err("Token is not burnable".to_string());
        }

        let from_balance = self.balance_of(from);
        if from_balance < amount {
            return Err(format!("Insufficient balance to burn: {} < {}", from_balance, amount));
        }

        self.balances.insert(from.to_string(), from_balance - amount);
        self.total_supply -= amount;

        Ok(())
    }

    /// Increase allowance
    pub fn increase_allowance(
        &mut self,
        owner: &str,
        spender: &str,
        added_value: u128,
    ) -> Result<(), String> {
        let current_allowance = self.allowance(owner, spender);
        self.approve(owner, spender, current_allowance + added_value)
    }

    /// Decrease allowance
    pub fn decrease_allowance(
        &mut self,
        owner: &str,
        spender: &str,
        subtracted_value: u128,
    ) -> Result<(), String> {
        let current_allowance = self.allowance(owner, spender);
        if current_allowance < subtracted_value {
            return Err("Decreased allowance below zero".to_string());
        }
        self.approve(owner, spender, current_allowance - subtracted_value)
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = Act20Token::new(
            "Test Token".to_string(),
            "TST".to_string(),
            18,
            1_000_000_000_000_000_000_000, // 1000 tokens
            "ACT-owner".to_string(),
            true,
            true,
        );

        assert_eq!(token.name, "Test Token");
        assert_eq!(token.symbol, "TST");
        assert_eq!(token.decimals, 18);
        assert_eq!(token.total_supply, 1_000_000_000_000_000_000_000);
        assert_eq!(token.balance_of("ACT-owner"), 1_000_000_000_000_000_000_000);
    }

    #[test]
    fn test_transfer() {
        let mut token = Act20Token::new(
            "Test".to_string(),
            "TST".to_string(),
            18,
            1000,
            "ACT-alice".to_string(),
            false,
            false,
        );

        let result = token.transfer("ACT-alice", "ACT-bob", 100);
        assert!(result.is_ok());
        assert_eq!(token.balance_of("ACT-alice"), 900);
        assert_eq!(token.balance_of("ACT-bob"), 100);
    }

    #[test]
    fn test_insufficient_balance() {
        let mut token = Act20Token::new(
            "Test".to_string(),
            "TST".to_string(),
            18,
            1000,
            "ACT-alice".to_string(),
            false,
            false,
        );

        let result = token.transfer("ACT-alice", "ACT-bob", 2000);
        assert!(result.is_err());
    }

    #[test]
    fn test_approve_and_transfer_from() {
        let mut token = Act20Token::new(
            "Test".to_string(),
            "TST".to_string(),
            18,
            1000,
            "ACT-alice".to_string(),
            false,
            false,
        );

        // Alice approves Bob to spend 100
        token.approve("ACT-alice", "ACT-bob", 100).unwrap();
        assert_eq!(token.allowance("ACT-alice", "ACT-bob"), 100);

        // Bob transfers 50 from Alice to Charlie
        token
            .transfer_from("ACT-bob", "ACT-alice", "ACT-charlie", 50)
            .unwrap();

        assert_eq!(token.balance_of("ACT-alice"), 950);
        assert_eq!(token.balance_of("ACT-charlie"), 50);
        assert_eq!(token.allowance("ACT-alice", "ACT-bob"), 50);
    }

    #[test]
    fn test_mint() {
        let mut token = Act20Token::new(
            "Test".to_string(),
            "TST".to_string(),
            18,
            1000,
            "ACT-owner".to_string(),
            true,
            false,
        );

        token.mint("ACT-owner", "ACT-alice", 500).unwrap();
        assert_eq!(token.total_supply, 1500);
        assert_eq!(token.balance_of("ACT-alice"), 500);
    }

    #[test]
    fn test_burn() {
        let mut token = Act20Token::new(
            "Test".to_string(),
            "TST".to_string(),
            18,
            1000,
            "ACT-owner".to_string(),
            false,
            true,
        );

        token.burn("ACT-owner", 200).unwrap();
        assert_eq!(token.total_supply, 800);
        assert_eq!(token.balance_of("ACT-owner"), 800);
    }

    #[test]
    fn test_increase_decrease_allowance() {
        let mut token = Act20Token::new(
            "Test".to_string(),
            "TST".to_string(),
            18,
            1000,
            "ACT-alice".to_string(),
            false,
            false,
        );

        token.approve("ACT-alice", "ACT-bob", 100).unwrap();
        token.increase_allowance("ACT-alice", "ACT-bob", 50).unwrap();
        assert_eq!(token.allowance("ACT-alice", "ACT-bob"), 150);

        token.decrease_allowance("ACT-alice", "ACT-bob", 25).unwrap();
        assert_eq!(token.allowance("ACT-alice", "ACT-bob"), 125);
    }
}

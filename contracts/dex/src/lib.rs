use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simple AMM DEX with constant product formula (x * y = k)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPool {
    pub token_a: String,
    pub token_b: String,
    pub reserve_a: u128,
    pub reserve_b: u128,
    pub total_liquidity: u128,
    pub liquidity_providers: HashMap<String, u128>,
    pub fee_rate: u32, // in basis points (30 = 0.3%)
}

impl LiquidityPool {
    pub fn new(token_a: String, token_b: String, fee_rate: u32) -> Self {
        Self {
            token_a,
            token_b,
            reserve_a: 0,
            reserve_b: 0,
            total_liquidity: 0,
            liquidity_providers: HashMap::new(),
            fee_rate,
        }
    }

    /// Add liquidity to the pool
    pub fn add_liquidity(
        &mut self,
        provider: &str,
        amount_a: u128,
        amount_b: u128,
    ) -> Result<u128, String> {
        if amount_a == 0 || amount_b == 0 {
            return Err("Amounts must be greater than zero".to_string());
        }

        let liquidity_minted = if self.total_liquidity == 0 {
            // First liquidity provision
            let liquidity = (amount_a * amount_b).sqrt();
            if liquidity == 0 {
                return Err("Insufficient liquidity minted".to_string());
            }
            liquidity
        } else {
            // Subsequent liquidity must maintain ratio
            let liquidity_a = (amount_a * self.total_liquidity) / self.reserve_a;
            let liquidity_b = (amount_b * self.total_liquidity) / self.reserve_b;
            liquidity_a.min(liquidity_b)
        };

        if liquidity_minted == 0 {
            return Err("Liquidity amount too small".to_string());
        }

        self.reserve_a += amount_a;
        self.reserve_b += amount_b;
        self.total_liquidity += liquidity_minted;

        let provider_liquidity = self.liquidity_providers.get(provider).unwrap_or(&0);
        self.liquidity_providers
            .insert(provider.to_string(), provider_liquidity + liquidity_minted);

        Ok(liquidity_minted)
    }

    /// Remove liquidity from the pool
    pub fn remove_liquidity(
        &mut self,
        provider: &str,
        liquidity: u128,
    ) -> Result<(u128, u128), String> {
        let provider_liquidity = *self.liquidity_providers.get(provider).unwrap_or(&0);
        if provider_liquidity < liquidity {
            return Err("Insufficient liquidity".to_string());
        }

        let amount_a = (liquidity * self.reserve_a) / self.total_liquidity;
        let amount_b = (liquidity * self.reserve_b) / self.total_liquidity;

        if amount_a == 0 || amount_b == 0 {
            return Err("Insufficient liquidity burned".to_string());
        }

        self.reserve_a -= amount_a;
        self.reserve_b -= amount_b;
        self.total_liquidity -= liquidity;
        self.liquidity_providers
            .insert(provider.to_string(), provider_liquidity - liquidity);

        Ok((amount_a, amount_b))
    }

    /// Swap token A for token B
    pub fn swap_a_for_b(&mut self, amount_a_in: u128) -> Result<u128, String> {
        if amount_a_in == 0 {
            return Err("Amount must be greater than zero".to_string());
        }

        // Calculate amount out with fee
        let amount_a_with_fee = amount_a_in * (10000 - self.fee_rate as u128);
        let numerator = amount_a_with_fee * self.reserve_b;
        let denominator = (self.reserve_a * 10000) + amount_a_with_fee;
        let amount_b_out = numerator / denominator;

        if amount_b_out == 0 {
            return Err("Insufficient output amount".to_string());
        }

        if amount_b_out >= self.reserve_b {
            return Err("Insufficient liquidity".to_string());
        }

        self.reserve_a += amount_a_in;
        self.reserve_b -= amount_b_out;

        Ok(amount_b_out)
    }

    /// Swap token B for token A
    pub fn swap_b_for_a(&mut self, amount_b_in: u128) -> Result<u128, String> {
        if amount_b_in == 0 {
            return Err("Amount must be greater than zero".to_string());
        }

        let amount_b_with_fee = amount_b_in * (10000 - self.fee_rate as u128);
        let numerator = amount_b_with_fee * self.reserve_a;
        let denominator = (self.reserve_b * 10000) + amount_b_with_fee;
        let amount_a_out = numerator / denominator;

        if amount_a_out == 0 {
            return Err("Insufficient output amount".to_string());
        }

        if amount_a_out >= self.reserve_a {
            return Err("Insufficient liquidity".to_string());
        }

        self.reserve_b += amount_b_in;
        self.reserve_a -= amount_a_out;

        Ok(amount_a_out)
    }

    /// Get quote for swapping A to B
    pub fn get_amount_out_a_to_b(&self, amount_a_in: u128) -> u128 {
        if amount_a_in == 0 || self.reserve_a == 0 || self.reserve_b == 0 {
            return 0;
        }
        let amount_a_with_fee = amount_a_in * (10000 - self.fee_rate as u128);
        let numerator = amount_a_with_fee * self.reserve_b;
        let denominator = (self.reserve_a * 10000) + amount_a_with_fee;
        numerator / denominator
    }

    /// Get quote for swapping B to A
    pub fn get_amount_out_b_to_a(&self, amount_b_in: u128) -> u128 {
        if amount_b_in == 0 || self.reserve_a == 0 || self.reserve_b == 0 {
            return 0;
        }
        let amount_b_with_fee = amount_b_in * (10000 - self.fee_rate as u128);
        let numerator = amount_b_with_fee * self.reserve_a;
        let denominator = (self.reserve_b * 10000) + amount_b_with_fee;
        numerator / denominator
    }

    /// Get current price (reserve_b / reserve_a)
    pub fn get_price(&self) -> f64 {
        if self.reserve_a == 0 {
            return 0.0;
        }
        (self.reserve_b as f64) / (self.reserve_a as f64)
    }
}

trait IntegerSqrt {
    fn sqrt(self) -> Self;
}

impl IntegerSqrt for u128 {
    fn sqrt(self) -> Self {
        if self < 2 {
            return self;
        }
        let mut x = self;
        let mut y = (x + 1) / 2;
        while y < x {
            x = y;
            y = (x + self / x) / 2;
        }
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_initial_liquidity() {
        let mut pool = LiquidityPool::new("TOKEN_A".to_string(), "TOKEN_B".to_string(), 30);
        
        let liquidity = pool.add_liquidity("alice", 1000, 2000).unwrap();
        assert_eq!(pool.reserve_a, 1000);
        assert_eq!(pool.reserve_b, 2000);
        assert_eq!(pool.total_liquidity, liquidity);
        assert_eq!(pool.liquidity_providers.get("alice"), Some(&liquidity));
    }

    #[test]
    fn test_add_subsequent_liquidity() {
        let mut pool = LiquidityPool::new("TOKEN_A".to_string(), "TOKEN_B".to_string(), 30);
        
        pool.add_liquidity("alice", 1000, 2000).unwrap();
        let liquidity2 = pool.add_liquidity("bob", 500, 1000).unwrap();
        
        assert!(liquidity2 > 0);
        assert_eq!(pool.reserve_a, 1500);
        assert_eq!(pool.reserve_b, 3000);
    }

    #[test]
    fn test_remove_liquidity() {
        let mut pool = LiquidityPool::new("TOKEN_A".to_string(), "TOKEN_B".to_string(), 30);
        
        let liquidity = pool.add_liquidity("alice", 1000, 2000).unwrap();
        let (amount_a, amount_b) = pool.remove_liquidity("alice", liquidity / 2).unwrap();
        
        assert!(amount_a > 0 && amount_b > 0);
        assert_eq!(pool.reserve_a, 1000 - amount_a);
        assert_eq!(pool.reserve_b, 2000 - amount_b);
    }

    #[test]
    fn test_swap_a_for_b() {
        let mut pool = LiquidityPool::new("TOKEN_A".to_string(), "TOKEN_B".to_string(), 30);
        
        pool.add_liquidity("alice", 1000, 2000).unwrap();
        let amount_b_out = pool.swap_a_for_b(100).unwrap();
        
        assert!(amount_b_out > 0);
        assert!(amount_b_out < 200); // Should get less than 200 due to slippage and fees
        assert_eq!(pool.reserve_a, 1100);
        assert_eq!(pool.reserve_b, 2000 - amount_b_out);
    }

    #[test]
    fn test_swap_b_for_a() {
        let mut pool = LiquidityPool::new("TOKEN_A".to_string(), "TOKEN_B".to_string(), 30);
        
        pool.add_liquidity("alice", 1000, 2000).unwrap();
        let amount_a_out = pool.swap_b_for_a(200).unwrap();
        
        assert!(amount_a_out > 0);
        assert!(amount_a_out < 100);
        assert_eq!(pool.reserve_b, 2200);
        assert_eq!(pool.reserve_a, 1000 - amount_a_out);
    }

    #[test]
    fn test_get_quote() {
        let mut pool = LiquidityPool::new("TOKEN_A".to_string(), "TOKEN_B".to_string(), 30);
        
        pool.add_liquidity("alice", 1000, 2000).unwrap();
        let quote = pool.get_amount_out_a_to_b(100);
        
        assert!(quote > 0);
        assert!(quote < 200);
    }

    #[test]
    fn test_price() {
        let mut pool = LiquidityPool::new("TOKEN_A".to_string(), "TOKEN_B".to_string(), 30);
        
        pool.add_liquidity("alice", 1000, 2000).unwrap();
        let price = pool.get_price();
        
        assert_eq!(price, 2.0); // 2000 / 1000 = 2.0
    }
}

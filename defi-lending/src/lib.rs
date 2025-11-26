/// DeFi Lending Protocol
/// 
/// Over-collateralized lending and borrowing platform with:
/// - Collateral deposits
/// - Borrowing with health factor monitoring
/// - Utilization-based interest rates
/// - Liquidation engine
/// - Oracle price feeds integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const SECONDS_PER_YEAR: u64 = 31536000;
const LIQUIDATION_THRESHOLD: u64 = 8000; // 80% LTV (basis points)
const LIQUIDATION_BONUS: u64 = 500;      // 5% bonus for liquidators
const MIN_HEALTH_FACTOR: u64 = 10000;     // 1.0 (basis points: 10000 = 1)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConfig {
    pub asset: String,              // Asset address
    pub collateral_factor: u64,     // Max LTV in basis points (7500 = 75%)
    pub reserve_factor: u64,        // Protocol reserve % (1000 = 10%)
    pub base_rate: u64,             // Base interest rate
    pub slope1: u64,                // Interest rate slope below kink
    pub slope2: u64,                // Interest rate slope above kink
    pub optimal_utilization: u64,  // Kink point (8000 = 80%)
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPosition {
    pub collateral: HashMap<String, u128>,  // asset -> amount
    pub borrows: HashMap<String, BorrowInfo>,  // asset -> borrow_info
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowInfo {
    pub principal: u128,           // Original borrowed amount
    pub interest_index: u128,       // Interest index at borrow time
    pub last_update: u64,          // Last update timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub total_deposits: u128,
    pub total_borrows: u128,
    pub total_reserves: u128,
    pub interest_index: u128,       // Cumulative interest index
    pub last_update: u64,           // Last interest accrual timestamp
}

pub struct LendingProtocol {
    /// Market configurations
    pub markets: HashMap<String, MarketConfig>,
    
    /// Market state
    pub market_state: HashMap<String, Market>,
    
    /// User positions
    pub users: HashMap<String, UserPosition>,
    
    /// Oracle prices (asset -> price in USD, 18 decimals)
    pub prices: HashMap<String, u128>,
}

impl LendingProtocol {
    pub fn new() -> Self {
        Self {
            markets: HashMap::new(),
            market_state: HashMap::new(),
            users: HashMap::new(),
            prices: HashMap::new(),
        }
    }
    
    /// Add new market
    pub fn add_market(&mut self, asset: String, config: MarketConfig) {
        self.markets.insert(asset.clone(), config);
        self.market_state.insert(asset, Market {
            total_deposits: 0,
            total_borrows: 0,
            total_reserves: 0,
            interest_index: 10u128.pow(18), // Start at 1.0
            last_update: 0,
        });
    }
    
    /// Update oracle price
    pub fn update_price(&mut self, asset: String, price: u128) {
        self.prices.insert(asset, price);
    }
    
    /// Deposit collateral
    pub fn deposit(
        &mut self,
        user: &str,
        asset: &str,
        amount: u128,
        timestamp: u64,
    ) -> Result<(), String> {
        // Check if market exists
        if !self.markets.contains_key(asset) {
            return Err("Market does not exist".to_string());
        }
        
        // Accrue interest before state change
        self.accrue_interest(asset, timestamp)?;
        
        // Update market state
        let market = self.market_state.get_mut(asset).unwrap();
        market.total_deposits += amount;
        
        // Update user position
        let position = self.users.entry(user.to_string()).or_insert(UserPosition {
            collateral: HashMap::new(),
            borrows: HashMap::new(),
        });
        
        *position.collateral.entry(asset.to_string()).or_insert(0) += amount;
        
        Ok(())
    }
    
    /// Withdraw collateral
    pub fn withdraw(
        &mut self,
        user: &str,
        asset: &str,
        amount: u128,
        timestamp: u64,
    ) -> Result<(), String> {
        // Accrue interest
        self.accrue_interest(asset, timestamp)?;
        
        // Check user has enough collateral
        let position = self.users.get_mut(user)
            .ok_or("User has no position")?;
        
        let collateral = position.collateral.get_mut(asset)
            .ok_or("No collateral in this asset")?;
        
        if *collateral < amount {
            return Err("Insufficient collateral".to_string());
        }
        
        *collateral -= amount;
        
        // Check health factor after withdrawal
        let is_healthy = self.is_healthy(user)?;
        if !is_healthy {
            // Revert by getting mutable reference again
            let position = self.users.get_mut(user).unwrap();
            *position.collateral.get_mut(asset).unwrap() += amount;
            return Err("Withdrawal would make position unhealthy".to_string());
        }
        
        // Update market state
        let market = self.market_state.get_mut(asset).unwrap();
        market.total_deposits -= amount;
        
        Ok(())
    }
    
    /// Borrow assets
    pub fn borrow(
        &mut self,
        user: &str,
        asset: &str,
        amount: u128,
        timestamp: u64,
    ) -> Result<(), String> {
        // Accrue interest
        self.accrue_interest(asset, timestamp)?;
        
        // Get market
        let market = self.market_state.get(asset)
            .ok_or("Market does not exist")?;
        
        // Check liquidity
        if market.total_deposits < market.total_borrows + amount {
            return Err("Insufficient liquidity".to_string());
        }
        
        // Update user borrow
        let position = self.users.entry(user.to_string()).or_insert(UserPosition {
            collateral: HashMap::new(),
            borrows: HashMap::new(),
        });
        
        let borrow_info = BorrowInfo {
            principal: amount,
            interest_index: market.interest_index,
            last_update: timestamp,
        };
        
        if let Some(existing) = position.borrows.get_mut(asset) {
            // Add to existing borrow
            existing.principal += amount;
        } else {
            position.borrows.insert(asset.to_string(), borrow_info);
        }
        
        // Check health factor
        let is_healthy = self.is_healthy(user)?;
        if !is_healthy {
            // Revert borrow
            let position = self.users.get_mut(user).unwrap();
            if let Some(borrow) = position.borrows.get_mut(asset) {
                borrow.principal -= amount;
            }
            return Err("Borrow would make position unhealthy".to_string());
        }
        
        // Update market state
        let market = self.market_state.get_mut(asset).unwrap();
        market.total_borrows += amount;
        
        Ok(())
    }
    
    /// Repay borrowed assets
    pub fn repay(
        &mut self,
        user: &str,
        asset: &str,
        amount: u128,
        timestamp: u64,
    ) -> Result<u128, String> {
        // Accrue interest
        self.accrue_interest(asset, timestamp)?;
        
        // Get user position
        let position = self.users.get_mut(user)
            .ok_or("User has no position")?;
        
        let borrow = position.borrows.get_mut(asset)
            .ok_or("No borrow in this asset")?;
        
        // Calculate current debt with interest
        let market = self.market_state.get(asset).unwrap();
        let market_index = market.interest_index;
        let borrow_index = borrow.interest_index;
        let principal = borrow.principal;
        
        let debt = if principal == 0 {
            0
        } else if market_index == borrow_index {
            principal
        } else {
            let scaled_principal = principal / 1_000_000_000;
            let scaled_borrow_index = borrow_index / 1_000_000_000;
            if scaled_borrow_index == 0 {
                principal
            } else {
                scaled_principal * market_index / scaled_borrow_index
            }
        };
        
        // Determine repay amount
        let actual_repay = amount.min(debt);
        
        // Update borrow
        borrow.principal = borrow.principal.saturating_sub(actual_repay);
        borrow.last_update = timestamp;
        
        // Update market state
        let market = self.market_state.get_mut(asset).unwrap();
        market.total_borrows = market.total_borrows.saturating_sub(actual_repay);
        
        Ok(actual_repay)
    }
    
    /// Liquidate underwater position
    pub fn liquidate(
        &mut self,
        liquidator: &str,
        borrower: &str,
        collateral_asset: &str,
        borrow_asset: &str,
        repay_amount: u128,
        timestamp: u64,
    ) -> Result<u128, String> {
        // Accrue interest on both markets
        self.accrue_interest(collateral_asset, timestamp)?;
        self.accrue_interest(borrow_asset, timestamp)?;
        
        // Check if position is liquidatable
        if self.is_healthy(borrower)? {
            return Err("Position is healthy, cannot liquidate".to_string());
        }
        
        // Calculate debt and seize amount first
        let (debt, seize_amount, actual_repay) = {
            let borrower_pos = self.users.get(borrower)
                .ok_or("Borrower has no position")?;
            
            let borrow = borrower_pos.borrows.get(borrow_asset)
                .ok_or("Borrower has no borrow in this asset")?;
            
            let market = self.market_state.get(borrow_asset).unwrap();
            let debt = self.calculate_borrow_balance(borrow, market);
            let actual_repay = repay_amount.min(debt);
            
            let collateral_price = self.prices.get(collateral_asset)
                .ok_or("Collateral price not available")?;
            let borrow_price = self.prices.get(borrow_asset)
                .ok_or("Borrow price not available")?;
            
            let collateral_value = actual_repay * borrow_price / collateral_price;
            let bonus = collateral_value * LIQUIDATION_BONUS as u128 / 10000;
            let seize_amount = collateral_value + bonus;
            
            (debt, seize_amount, actual_repay)
        };
        
        // Now mutate borrower position
        {
            let borrower_pos = self.users.get_mut(borrower)
                .ok_or("Borrower has no position")?;
            
            let borrower_collateral = borrower_pos.collateral.get_mut(collateral_asset)
                .ok_or("Borrower has no collateral in this asset")?;
            
            if *borrower_collateral < seize_amount {
                return Err("Insufficient collateral to seize".to_string());
            }
            
            *borrower_collateral -= seize_amount;
            
            let borrow = borrower_pos.borrows.get_mut(borrow_asset).unwrap();
            borrow.principal = borrow.principal.saturating_sub(actual_repay);
        }
        
        // Transfer collateral to liquidator
        let liquidator_pos = self.users.entry(liquidator.to_string()).or_insert(UserPosition {
            collateral: HashMap::new(),
            borrows: HashMap::new(),
        });
        *liquidator_pos.collateral.entry(collateral_asset.to_string()).or_insert(0) += seize_amount;
        
        // Update market state
        let borrow_market = self.market_state.get_mut(borrow_asset).unwrap();
        borrow_market.total_borrows = borrow_market.total_borrows.saturating_sub(actual_repay);
        
        Ok(seize_amount)
    }
    
    /// Calculate current borrow balance with interest
    fn calculate_borrow_balance(&self, borrow: &BorrowInfo, market: &Market) -> u128 {
        if borrow.principal == 0 {
            return 0;
        }
        // Use checked math to avoid overflow
        // If index hasn't changed much, just return principal
        if market.interest_index == borrow.interest_index {
            return borrow.principal;
        }
        // Scale down to avoid overflow: (principal / 1e9) * index / (borrow_index / 1e9)
        let scaled_principal = borrow.principal / 1_000_000_000;
        let scaled_borrow_index = borrow.interest_index / 1_000_000_000;
        if scaled_borrow_index == 0 {
            return borrow.principal;
        }
        scaled_principal * market.interest_index / scaled_borrow_index
    }
    
    /// Accrue interest for a market
    fn accrue_interest(&mut self, asset: &str, timestamp: u64) -> Result<(), String> {
        let market = self.market_state.get_mut(asset)
            .ok_or("Market does not exist")?;
        
        if market.last_update >= timestamp {
            return Ok(()); // Already updated
        }
        
        let time_elapsed = timestamp - market.last_update;
        if time_elapsed == 0 || market.total_borrows == 0 {
            market.last_update = timestamp;
            return Ok(());
        }
        
        // Calculate utilization rate
        let utilization = if market.total_deposits > 0 {
            (market.total_borrows * 10000) / market.total_deposits
        } else {
            0
        };
        
        // Calculate borrow rate
        let config = self.markets.get(asset).unwrap();
        let optimal_util = config.optimal_utilization as u128;
        let borrow_rate = if utilization <= optimal_util {
            config.base_rate as u128 + (utilization * config.slope1 as u128 / optimal_util)
        } else {
            let excess = utilization - optimal_util;
            let excess_util = (excess * 10000) / (10000 - optimal_util);
            config.base_rate as u128 + config.slope1 as u128 + (excess_util * config.slope2 as u128 / 10000)
        };
        
        // Calculate interest accrued
        let interest_rate_per_second = borrow_rate / SECONDS_PER_YEAR as u128;
        let interest = market.total_borrows / 10000 * interest_rate_per_second * time_elapsed as u128 / 10000;
        
        // Update market
        market.total_borrows += interest;
        if market.total_borrows > interest {
            market.interest_index = market.interest_index + (market.interest_index * interest / (market.total_borrows - interest));
        }
        market.last_update = timestamp;
        
        // Calculate reserves
        let config = self.markets.get(asset).unwrap();
        let reserve_amount = interest * config.reserve_factor as u128 / 10000;
        market.total_reserves += reserve_amount;
        
        Ok(())
    }
    
    /// Calculate borrow interest rate based on utilization
    fn calculate_borrow_rate(&self, asset: &str, utilization: u128) -> Result<u128, String> {
        let config = self.markets.get(asset)
            .ok_or("Market does not exist")?;
        
        let optimal_util = config.optimal_utilization as u128;
        
        if utilization <= optimal_util {
            // Below kink: base + (utilization / optimal) * slope1
            let rate = config.base_rate as u128 + (utilization * config.slope1 as u128 / optimal_util);
            Ok(rate)
        } else {
            // Above kink: base + slope1 + ((utilization - optimal) / (10000 - optimal)) * slope2
            let excess = utilization - optimal_util;
            let excess_util = (excess * 10000) / (10000 - optimal_util);
            let rate = config.base_rate as u128 + config.slope1 as u128 + (excess_util * config.slope2 as u128 / 10000);
            Ok(rate)
        }
    }
    
    /// Check if user position is healthy
    pub fn is_healthy(&self, user: &str) -> Result<bool, String> {
        let health_factor = self.calculate_health_factor(user)?;
        Ok(health_factor >= MIN_HEALTH_FACTOR as u128)
    }
    
    /// Calculate health factor (collateral value / borrow value)
    pub fn calculate_health_factor(&self, user: &str) -> Result<u128, String> {
        let position = self.users.get(user)
            .ok_or("User has no position")?;
        
        // Calculate total collateral value (adjusted by collateral factor)
        let mut collateral_value = 0u128;
        for (asset, amount) in &position.collateral {
            let price = self.prices.get(asset)
                .ok_or("Price not available")?;
            let config = self.markets.get(asset)
                .ok_or("Market not found")?;
            
            // value = (amount / 1e18) * (price / 1e18) * collateral_factor / 10000
            let value = (amount / 10u128.pow(9)) * (price / 10u128.pow(9)) * config.collateral_factor as u128 / 10000;
            collateral_value += value;
        }
        
        // Calculate total borrow value
        let mut borrow_value = 0u128;
        for (asset, borrow) in &position.borrows {
            let price = self.prices.get(asset)
                .ok_or("Price not available")?;
            let market = self.market_state.get(asset)
                .ok_or("Market not found")?;
            
            let debt = self.calculate_borrow_balance(borrow, market);
            let value = (debt / 10u128.pow(9)) * (price / 10u128.pow(9));
            borrow_value += value;
        }
        
        if borrow_value == 0 {
            return Ok(u128::MAX); // No borrows = infinite health
        }
        
        // Health factor = collateral_value / borrow_value (in basis points)
        Ok(collateral_value * 10000 / borrow_value)
    }
    
    /// Get user position
    pub fn get_position(&self, user: &str) -> Option<&UserPosition> {
        self.users.get(user)
    }
    
    /// Get market state
    pub fn get_market(&self, asset: &str) -> Option<&Market> {
        self.market_state.get(asset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_protocol() -> LendingProtocol {
        let mut protocol = LendingProtocol::new();
        
        // Add ACT market
        protocol.add_market("ACT".to_string(), MarketConfig {
            asset: "ACT".to_string(),
            collateral_factor: 7500, // 75% LTV
            reserve_factor: 1000,    // 10%
            base_rate: 200,          // 2%
            slope1: 1000,            // 10%
            slope2: 5000,            // 50%
            optimal_utilization: 8000, // 80%
            enabled: true,
        });
        
        // Set price: 1 ACT = $10
        protocol.update_price("ACT".to_string(), 10 * 10u128.pow(18));
        
        protocol
    }

    #[test]
    fn test_deposit() {
        let mut protocol = create_test_protocol();
        
        let result = protocol.deposit("user1", "ACT", 1000 * 10u128.pow(18), 100);
        assert!(result.is_ok());
        
        let market = protocol.get_market("ACT").unwrap();
        assert_eq!(market.total_deposits, 1000 * 10u128.pow(18));
        
        let position = protocol.get_position("user1").unwrap();
        assert_eq!(*position.collateral.get("ACT").unwrap(), 1000 * 10u128.pow(18));
    }

    #[test]
    fn test_borrow() {
        let mut protocol = create_test_protocol();
        
        // Deposit collateral
        protocol.deposit("user1", "ACT", 1000 * 10u128.pow(18), 100).unwrap();
        
        // Borrow (up to 75% of collateral value)
        let result = protocol.borrow("user1", "ACT", 500 * 10u128.pow(18), 100);
        assert!(result.is_ok());
        
        let market = protocol.get_market("ACT").unwrap();
        assert_eq!(market.total_borrows, 500 * 10u128.pow(18));
    }

    #[test]
    fn test_borrow_too_much() {
        let mut protocol = create_test_protocol();
        
        // Deposit collateral
        protocol.deposit("user1", "ACT", 1000 * 10u128.pow(18), 100).unwrap();
        
        // Try to borrow more than allowed
        let result = protocol.borrow("user1", "ACT", 800 * 10u128.pow(18), 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_repay() {
        let mut protocol = create_test_protocol();
        
        protocol.deposit("user1", "ACT", 1000 * 10u128.pow(18), 100).unwrap();
        protocol.borrow("user1", "ACT", 500 * 10u128.pow(18), 100).unwrap();
        
        let result = protocol.repay("user1", "ACT", 200 * 10u128.pow(18), 200);
        assert!(result.is_ok());
        
        let repaid = result.unwrap();
        assert_eq!(repaid, 200 * 10u128.pow(18));
    }

    #[test]
    fn test_health_factor() {
        let mut protocol = create_test_protocol();
        
        protocol.deposit("user1", "ACT", 1000 * 10u128.pow(18), 100).unwrap();
        protocol.borrow("user1", "ACT", 500 * 10u128.pow(18), 100).unwrap();
        
        let health = protocol.calculate_health_factor("user1").unwrap();
        // Collateral: 1000 ACT * $10 * 0.75 = $7500
        // Borrow: 500 ACT * $10 = $5000
        // Health factor: 7500 / 5000 = 1.5 = 15000 basis points
        assert!(health >= 15000);
    }

    #[test]
    fn test_withdraw_healthy() {
        let mut protocol = create_test_protocol();
        
        protocol.deposit("user1", "ACT", 1000 * 10u128.pow(18), 100).unwrap();
        protocol.borrow("user1", "ACT", 200 * 10u128.pow(18), 100).unwrap();
        
        // Can withdraw some collateral while staying healthy
        let result = protocol.withdraw("user1", "ACT", 500 * 10u128.pow(18), 200);
        assert!(result.is_ok());
    }

    #[test]
    fn test_withdraw_unhealthy() {
        let mut protocol = create_test_protocol();
        
        protocol.deposit("user1", "ACT", 1000 * 10u128.pow(18), 100).unwrap();
        protocol.borrow("user1", "ACT", 700 * 10u128.pow(18), 100).unwrap();
        
        // Cannot withdraw - would make position unhealthy
        let result = protocol.withdraw("user1", "ACT", 500 * 10u128.pow(18), 200);
        assert!(result.is_err());
    }
}

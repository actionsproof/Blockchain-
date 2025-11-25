use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Minimum stake required to become a validator (100,000 ACT)
pub const MIN_VALIDATOR_STAKE: u64 = 100_000_000_000_000; // 100k with 9 decimals

/// Maximum number of active validators
pub const MAX_ACTIVE_VALIDATORS: usize = 100;

/// Unstaking lock period in blocks (~14 days at 2s blocks = 604,800 blocks)
pub const UNSTAKE_LOCK_PERIOD: u64 = 604_800;

/// Block reward in smallest units (50 ACT)
pub const BLOCK_REWARD: u64 = 50_000_000_000; // 50 ACT with 9 decimals

/// Validator's share of transaction fees (80%)
pub const VALIDATOR_FEE_SHARE: f64 = 0.8;

/// Treasury share of transaction fees (20%)
pub const TREASURY_FEE_SHARE: f64 = 0.2;

/// Maximum stake concentration per validator (20% of total)
pub const MAX_STAKE_CONCENTRATION: f64 = 0.2;

/// Minimum commission rate (5%)
pub const MIN_COMMISSION_RATE: u8 = 5;

/// Maximum commission rate (50%)
pub const MAX_COMMISSION_RATE: u8 = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlashReason {
    DoubleSigning,
    Downtime,
    InvalidBlock,
    GovernanceViolation,
}

impl SlashReason {
    /// Returns the percentage of stake to slash (0-100)
    pub fn slash_percentage(&self) -> u8 {
        match self {
            SlashReason::DoubleSigning => 30,
            SlashReason::Downtime => 5,
            SlashReason::InvalidBlock => 10,
            SlashReason::GovernanceViolation => 20,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashEvent {
    pub reason: SlashReason,
    pub amount: u64,
    pub block_height: u64,
    pub reporter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub address: String,
    pub stake: u64,
    pub delegated_stake: u64,
    pub commission_rate: u8,
    pub active: bool,
    pub joined_at: u64,
    pub last_block: u64,
    pub total_blocks: u64,
    pub slash_events: Vec<SlashEvent>,
    pub unclaimed_rewards: u64,
}

impl Validator {
    pub fn new(address: String, stake: u64, commission_rate: u8, block_height: u64) -> Self {
        Self {
            address,
            stake,
            delegated_stake: 0,
            commission_rate,
            active: true,
            joined_at: block_height,
            last_block: 0,
            total_blocks: 0,
            slash_events: Vec::new(),
            unclaimed_rewards: 0,
        }
    }

    pub fn total_stake(&self) -> u64 {
        self.stake.saturating_add(self.delegated_stake)
    }

    pub fn can_validate(&self) -> bool {
        self.active && self.stake >= MIN_VALIDATOR_STAKE
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegator {
    pub address: String,
    pub validator: String,
    pub amount: u64,
    pub delegated_at: u64,
    pub unclaimed_rewards: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnstakeRequest {
    pub address: String,
    pub amount: u64,
    pub requested_at: u64,
    pub available_at: u64,
    pub is_validator: bool,
}

pub struct StakingManager {
    validators: HashMap<String, Validator>,
    delegators: HashMap<String, Vec<Delegator>>,
    unstake_requests: Vec<UnstakeRequest>,
    current_height: u64,
    total_staked: u64,
}

impl StakingManager {
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            delegators: HashMap::new(),
            unstake_requests: Vec::new(),
            current_height: 0,
            total_staked: 0,
        }
    }

    /// Update current block height
    pub fn set_block_height(&mut self, height: u64) {
        self.current_height = height;
    }

    /// Stake tokens to become a validator
    pub fn stake(
        &mut self,
        address: String,
        amount: u64,
        commission_rate: u8,
    ) -> Result<String, String> {
        // Validate stake amount
        if amount < MIN_VALIDATOR_STAKE {
            return Err(format!(
                "Minimum stake is {} tokens",
                MIN_VALIDATOR_STAKE / 1_000_000_000
            ));
        }

        // Validate commission rate
        if commission_rate < MIN_COMMISSION_RATE || commission_rate > MAX_COMMISSION_RATE {
            return Err(format!(
                "Commission rate must be between {}% and {}%",
                MIN_COMMISSION_RATE, MAX_COMMISSION_RATE
            ));
        }

        // Check if already a validator
        if let Some(validator) = self.validators.get_mut(&address) {
            // Add to existing stake
            validator.stake = validator.stake.saturating_add(amount);
            validator.active = true;
        } else {
            // Create new validator
            let validator = Validator::new(address.clone(), amount, commission_rate, self.current_height);
            self.validators.insert(address.clone(), validator);
        }

        self.total_staked = self.total_staked.saturating_add(amount);

        // Check stake concentration only if there's significant total stake
        if self.total_staked > MIN_VALIDATOR_STAKE * 5 {
            if let Some(validator) = self.validators.get(&address) {
                let concentration = validator.total_stake() as f64 / self.total_staked as f64;
                if concentration > MAX_STAKE_CONCENTRATION {
                    return Err(format!(
                        "Stake concentration exceeds {}% limit",
                        (MAX_STAKE_CONCENTRATION * 100.0) as u8
                    ));
                }
            }
        }

        Ok(address)
    }

    /// Delegate tokens to a validator
    pub fn delegate(
        &mut self,
        delegator_address: String,
        validator_address: String,
        amount: u64,
    ) -> Result<(), String> {
        // Check validator exists and is active
        let validator = self
            .validators
            .get_mut(&validator_address)
            .ok_or("Validator not found")?;

        if !validator.active {
            return Err("Validator is not active".to_string());
        }

        // Update validator's delegated stake
        validator.delegated_stake = validator.delegated_stake.saturating_add(amount);

        self.total_staked = self.total_staked.saturating_add(amount);

        // Check stake concentration only if there's significant total stake
        if self.total_staked > MIN_VALIDATOR_STAKE * 5 {
            let concentration = validator.total_stake() as f64 / self.total_staked as f64;
            if concentration > MAX_STAKE_CONCENTRATION {
                validator.delegated_stake = validator.delegated_stake.saturating_sub(amount);
                self.total_staked = self.total_staked.saturating_sub(amount);
                return Err(format!(
                    "Delegation would exceed {}% stake concentration limit",
                    (MAX_STAKE_CONCENTRATION * 100.0) as u8
                ));
            }
        }

        // Create or update delegator entry
        let delegator_list = self.delegators.entry(delegator_address.clone()).or_insert_with(Vec::new);

        if let Some(existing) = delegator_list.iter_mut().find(|d| d.validator == validator_address) {
            existing.amount = existing.amount.saturating_add(amount);
        } else {
            delegator_list.push(Delegator {
                address: delegator_address,
                validator: validator_address,
                amount,
                delegated_at: self.current_height,
                unclaimed_rewards: 0,
            });
        }

        self.total_staked = self.total_staked.saturating_add(amount);

        Ok(())
    }

    /// Unstake tokens (validator exit or partial unstake)
    pub fn unstake(&mut self, address: String, amount: u64) -> Result<u64, String> {
        let validator = self
            .validators
            .get_mut(&address)
            .ok_or("Not a validator")?;

        if amount > validator.stake {
            return Err("Insufficient staked amount".to_string());
        }

        let remaining = validator.stake.saturating_sub(amount);

        // Check if partial unstake leaves sufficient stake
        if remaining > 0 && remaining < MIN_VALIDATOR_STAKE {
            return Err(format!(
                "Remaining stake must be at least {} tokens or zero",
                MIN_VALIDATOR_STAKE / 1_000_000_000
            ));
        }

        // Update validator stake
        validator.stake = remaining;

        // If complete exit, mark inactive
        if remaining == 0 {
            validator.active = false;
        }

        // Create unstake request with lock period
        let request_id = self.unstake_requests.len() as u64;
        self.unstake_requests.push(UnstakeRequest {
            address: address.clone(),
            amount,
            requested_at: self.current_height,
            available_at: self.current_height + UNSTAKE_LOCK_PERIOD,
            is_validator: true,
        });

        self.total_staked = self.total_staked.saturating_sub(amount);

        Ok(request_id)
    }

    /// Undelegate tokens from a validator
    pub fn undelegate(
        &mut self,
        delegator_address: String,
        validator_address: String,
        amount: u64,
    ) -> Result<u64, String> {
        // Find delegator entry
        let delegator_list = self
            .delegators
            .get_mut(&delegator_address)
            .ok_or("No delegations found")?;

        let delegator = delegator_list
            .iter_mut()
            .find(|d| d.validator == validator_address)
            .ok_or("Not delegated to this validator")?;

        if amount > delegator.amount {
            return Err("Insufficient delegated amount".to_string());
        }

        // Update delegator amount
        delegator.amount = delegator.amount.saturating_sub(amount);

        // Update validator's delegated stake
        if let Some(validator) = self.validators.get_mut(&validator_address) {
            validator.delegated_stake = validator.delegated_stake.saturating_sub(amount);
        }

        // Remove delegator entry if amount is zero
        if delegator.amount == 0 {
            delegator_list.retain(|d| d.validator != validator_address);
        }

        // Create unstake request with lock period
        let request_id = self.unstake_requests.len() as u64;
        self.unstake_requests.push(UnstakeRequest {
            address: delegator_address,
            amount,
            requested_at: self.current_height,
            available_at: self.current_height + UNSTAKE_LOCK_PERIOD,
            is_validator: false,
        });

        self.total_staked = self.total_staked.saturating_sub(amount);

        Ok(request_id)
    }

    /// Claim unstaked tokens after lock period
    pub fn claim_unstaked(&mut self, address: String) -> Result<u64, String> {
        let mut total_claimed = 0u64;

        // Find all completed unstake requests for this address
        self.unstake_requests.retain(|req| {
            if req.address == address && req.available_at <= self.current_height {
                total_claimed = total_claimed.saturating_add(req.amount);
                false // Remove this request
            } else {
                true // Keep this request
            }
        });

        if total_claimed == 0 {
            return Err("No unstaked tokens available to claim".to_string());
        }

        Ok(total_claimed)
    }

    /// Distribute block rewards to validator and delegators
    pub fn distribute_block_reward(&mut self, validator_address: &str, tx_fees: u64) {
        let validator = match self.validators.get_mut(validator_address) {
            Some(v) => v,
            None => return,
        };

        // Calculate total reward
        let validator_fee_portion = (tx_fees as f64 * VALIDATOR_FEE_SHARE) as u64;
        let total_reward = BLOCK_REWARD.saturating_add(validator_fee_portion);

        // Calculate commission
        let commission = (total_reward as f64 * (validator.commission_rate as f64 / 100.0)) as u64;
        let delegator_pool = total_reward.saturating_sub(commission);

        // Distribute to validator
        validator.unclaimed_rewards = validator.unclaimed_rewards.saturating_add(commission);
        validator.last_block = self.current_height;
        validator.total_blocks = validator.total_blocks.saturating_add(1);

        // Distribute to delegators
        if validator.delegated_stake > 0 {
            let validator_addr = validator_address.to_string();
            // Iterate through all delegator lists to find those delegating to this validator
            for delegator_list in self.delegators.values_mut() {
                for delegator in delegator_list.iter_mut() {
                    if delegator.validator == validator_addr {
                        let share = (delegator_pool as f64 * (delegator.amount as f64 / self.validators.get(&validator_addr).unwrap().delegated_stake as f64)) as u64;
                        delegator.unclaimed_rewards = delegator.unclaimed_rewards.saturating_add(share);
                    }
                }
            }
        }
    }

    /// Claim accumulated rewards
    pub fn claim_rewards(&mut self, address: String) -> Result<u64, String> {
        let mut total_rewards = 0u64;

        // Check if validator
        if let Some(validator) = self.validators.get_mut(&address) {
            total_rewards = total_rewards.saturating_add(validator.unclaimed_rewards);
            validator.unclaimed_rewards = 0;
        }

        // Check if delegator
        if let Some(delegator_list) = self.delegators.get_mut(&address) {
            for delegator in delegator_list.iter_mut() {
                total_rewards = total_rewards.saturating_add(delegator.unclaimed_rewards);
                delegator.unclaimed_rewards = 0;
            }
        }

        if total_rewards == 0 {
            return Err("No rewards available to claim".to_string());
        }

        Ok(total_rewards)
    }

    /// Slash a validator for misbehavior
    pub fn slash(
        &mut self,
        validator_address: String,
        reason: SlashReason,
        reporter: Option<String>,
    ) -> Result<u64, String> {
        let validator = self
            .validators
            .get_mut(&validator_address)
            .ok_or("Validator not found")?;

        let slash_percentage = reason.slash_percentage();
        let slash_amount = (validator.stake as f64 * (slash_percentage as f64 / 100.0)) as u64;

        // Deduct from validator stake
        validator.stake = validator.stake.saturating_sub(slash_amount);

        // Record slash event
        validator.slash_events.push(SlashEvent {
            reason: reason.clone(),
            amount: slash_amount,
            block_height: self.current_height,
            reporter,
        });

        // Check if validator should be deactivated
        if validator.stake < MIN_VALIDATOR_STAKE {
            validator.active = false;
        }

        self.total_staked = self.total_staked.saturating_sub(slash_amount);

        Ok(slash_amount)
    }

    /// Get active validators sorted by total stake
    pub fn get_active_validators(&self) -> Vec<Validator> {
        let mut validators: Vec<Validator> = self
            .validators
            .values()
            .filter(|v| v.can_validate())
            .cloned()
            .collect();

        validators.sort_by(|a, b| b.total_stake().cmp(&a.total_stake()));

        validators.truncate(MAX_ACTIVE_VALIDATORS);

        validators
    }

    /// Get all validators (active and inactive)
    pub fn get_all_validators(&self) -> Vec<Validator> {
        let mut validators: Vec<Validator> = self.validators.values().cloned().collect();
        validators.sort_by(|a, b| b.total_stake().cmp(&a.total_stake()));
        validators
    }

    /// Get validator by address
    pub fn get_validator(&self, address: &str) -> Option<Validator> {
        self.validators.get(address).cloned()
    }

    /// Get delegations for an address
    pub fn get_delegations(&self, address: &str) -> Vec<Delegator> {
        self.delegators.get(address).cloned().unwrap_or_default()
    }

    /// Get pending unstake requests for an address
    pub fn get_unstake_requests(&self, address: &str) -> Vec<UnstakeRequest> {
        self.unstake_requests
            .iter()
            .filter(|req| req.address == address)
            .cloned()
            .collect()
    }

    /// Get total unclaimed rewards for an address
    pub fn get_unclaimed_rewards(&self, address: &str) -> u64 {
        let mut total = 0u64;

        // Check validator rewards
        if let Some(validator) = self.validators.get(address) {
            total = total.saturating_add(validator.unclaimed_rewards);
        }

        // Check delegator rewards
        if let Some(delegator_list) = self.delegators.get(address) {
            for delegator in delegator_list {
                total = total.saturating_add(delegator.unclaimed_rewards);
            }
        }

        total
    }

    /// Get total staked tokens across network
    pub fn get_total_staked(&self) -> u64 {
        self.total_staked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stake_validator() {
        let mut staking = StakingManager::new();
        staking.set_block_height(1000);

        let result = staking.stake(
            "ACT-validator1".to_string(),
            MIN_VALIDATOR_STAKE,
            10,
        );
        assert!(result.is_ok());

        let validator = staking.get_validator("ACT-validator1").unwrap();
        assert_eq!(validator.stake, MIN_VALIDATOR_STAKE);
        assert_eq!(validator.commission_rate, 10);
        assert!(validator.active);
    }

    #[test]
    fn test_insufficient_stake() {
        let mut staking = StakingManager::new();
        let result = staking.stake(
            "ACT-validator1".to_string(),
            MIN_VALIDATOR_STAKE - 1,
            10,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_delegate() {
        let mut staking = StakingManager::new();
        staking.set_block_height(1000);

        // Create validator
        staking.stake("ACT-validator1".to_string(), MIN_VALIDATOR_STAKE, 10).unwrap();

        // Delegate
        let result = staking.delegate(
            "ACT-delegator1".to_string(),
            "ACT-validator1".to_string(),
            50_000_000_000_000,
        );
        assert!(result.is_ok());

        let validator = staking.get_validator("ACT-validator1").unwrap();
        assert_eq!(validator.delegated_stake, 50_000_000_000_000);

        let delegations = staking.get_delegations("ACT-delegator1");
        assert_eq!(delegations.len(), 1);
        assert_eq!(delegations[0].amount, 50_000_000_000_000);
    }

    #[test]
    fn test_unstake() {
        let mut staking = StakingManager::new();
        staking.set_block_height(1000);

        staking.stake("ACT-validator1".to_string(), MIN_VALIDATOR_STAKE, 10).unwrap();

        let request_id = staking.unstake("ACT-validator1".to_string(), MIN_VALIDATOR_STAKE).unwrap();
        assert_eq!(request_id, 0);

        let validator = staking.get_validator("ACT-validator1").unwrap();
        assert_eq!(validator.stake, 0);
        assert!(!validator.active);

        let requests = staking.get_unstake_requests("ACT-validator1");
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].amount, MIN_VALIDATOR_STAKE);
    }

    #[test]
    fn test_reward_distribution() {
        let mut staking = StakingManager::new();
        staking.set_block_height(1000);

        // Create validator with 10% commission
        staking.stake("ACT-validator1".to_string(), MIN_VALIDATOR_STAKE, 10).unwrap();

        // Add delegator
        staking.delegate(
            "ACT-delegator1".to_string(),
            "ACT-validator1".to_string(),
            MIN_VALIDATOR_STAKE,
        ).unwrap();

        // Distribute reward
        staking.distribute_block_reward("ACT-validator1", 1_000_000_000);

        let validator = staking.get_validator("ACT-validator1").unwrap();
        assert!(validator.unclaimed_rewards > 0);

        let delegations = staking.get_delegations("ACT-delegator1");
        assert!(delegations[0].unclaimed_rewards > 0);
    }

    #[test]
    fn test_slash() {
        let mut staking = StakingManager::new();
        staking.set_block_height(1000);

        staking.stake("ACT-validator1".to_string(), MIN_VALIDATOR_STAKE, 10).unwrap();

        let slashed = staking.slash(
            "ACT-validator1".to_string(),
            SlashReason::DoubleSigning,
            Some("ACT-reporter1".to_string()),
        ).unwrap();

        assert_eq!(slashed, (MIN_VALIDATOR_STAKE as f64 * 0.3) as u64);

        let validator = staking.get_validator("ACT-validator1").unwrap();
        assert_eq!(validator.slash_events.len(), 1);
        assert!(!validator.active); // Should be deactivated due to insufficient stake
    }
}

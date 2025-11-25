# ACT Chain Staking Mechanism Design

## Overview
Proof-of-Authority consensus with economic staking layer for validator accountability and network security.

## Core Concepts

### Validator Staking
- **Minimum Stake**: 100,000 ACT tokens required to become validator
- **Maximum Validators**: 100 active validators
- **Delegation**: Token holders can delegate to validators (no minimum)
- **Lock Period**: 14 days unstaking period for security

### Staking Economics
- **Block Rewards**: 50 ACT per block (distributed to validators)
- **Transaction Fees**: Split 80% validators, 20% treasury
- **Delegation Rewards**: Validators share rewards with delegators (commission-based)
- **Annual Yield**: Target ~12% APY for staked tokens

### Slashing Conditions
1. **Double Signing**: -30% stake (signing conflicting blocks)
2. **Downtime**: -5% stake (offline >24 hours)
3. **Invalid Block**: -10% stake (producing invalid transactions)
4. **Governance Violation**: -20% stake (not following protocol upgrades)

Slashed funds are burned or sent to treasury based on governance.

## Data Structures

### Validator
```rust
pub struct Validator {
    pub address: String,              // Validator account address
    pub stake: u64,                   // Self-staked tokens
    pub delegated_stake: u64,         // Total delegated by others
    pub commission_rate: u8,          // 0-100% (e.g., 10 = 10%)
    pub active: bool,                 // Currently validating
    pub joined_at: u64,               // Block height when joined
    pub last_block: u64,              // Last block produced
    pub total_blocks: u64,            // Lifetime blocks produced
    pub slash_events: Vec<SlashEvent>, // History of penalties
}
```

### Delegator
```rust
pub struct Delegator {
    pub address: String,              // Delegator account
    pub validator: String,            // Target validator address
    pub amount: u64,                  // Delegated tokens
    pub delegated_at: u64,            // Block height
    pub unclaimed_rewards: u64,       // Pending rewards
}
```

### UnstakeRequest
```rust
pub struct UnstakeRequest {
    pub address: String,              // Requester address
    pub amount: u64,                  // Tokens to unstake
    pub requested_at: u64,            // Block height
    pub available_at: u64,            // Block height + lock period
    pub is_validator: bool,           // True if validator unstaking
}
```

### SlashEvent
```rust
pub struct SlashEvent {
    pub reason: SlashReason,          // Why slashed
    pub amount: u64,                  // Tokens slashed
    pub block_height: u64,            // When occurred
    pub reporter: Option<String>,     // Who reported (if applicable)
}

pub enum SlashReason {
    DoubleSigning,
    Downtime,
    InvalidBlock,
    GovernanceViolation,
}
```

## Staking Operations

### 1. Stake (Become Validator)
```rust
stake(amount: u64, commission_rate: u8) -> Result<ValidatorId>
```
- **Requires**: amount >= 100,000 ACT
- **Effects**: 
  - Locks tokens from account balance
  - Creates Validator entry with active=true
  - Adds to active validator set (if < 100 validators)
  - Otherwise joins waiting queue (sorted by stake)

### 2. Delegate
```rust
delegate(validator_address: String, amount: u64) -> Result<()>
```
- **Requires**: validator exists and is active
- **Effects**:
  - Locks tokens from delegator balance
  - Increases validator.delegated_stake
  - Creates/updates Delegator entry
  - Shares in validator's block rewards

### 3. Unstake (Validator Exit)
```rust
unstake(amount: u64) -> Result<UnstakeRequestId>
```
- **Requires**: 
  - Validator has sufficient stake remaining (or exiting completely)
  - If partial unstake: remaining_stake >= 100,000 ACT
- **Effects**:
  - Creates UnstakeRequest with 14-day lock
  - If complete exit: sets validator.active=false
  - Removes from active validator set
  - Tokens available after lock period

### 4. Undelegate
```rust
undelegate(validator_address: String, amount: u64) -> Result<UnstakeRequestId>
```
- **Requires**: delegator has sufficient delegation
- **Effects**:
  - Creates UnstakeRequest with 14-day lock
  - Reduces validator.delegated_stake
  - Updates/removes Delegator entry
  - Tokens available after lock period

### 5. Claim Unstaked
```rust
claim_unstaked() -> Result<u64>
```
- **Effects**:
  - Finds all UnstakeRequests where available_at <= current_height
  - Returns locked tokens to account balance
  - Deletes processed requests

### 6. Claim Rewards
```rust
claim_rewards() -> Result<u64>
```
- **Effects**:
  - Calculates unclaimed rewards since last claim
  - Transfers rewards to account balance
  - Resets unclaimed_rewards counter
  - Updates last_claim_height

## Reward Distribution

### Block Production Rewards
When validator produces block:
```
block_reward = 50 ACT
tx_fees = sum(transaction fees in block)
total_reward = block_reward + (tx_fees * 0.8)

validator_commission = total_reward * (commission_rate / 100)
delegator_pool = total_reward - validator_commission

for each delegator:
    delegator_share = delegator_pool * (delegator.amount / validator.delegated_stake)
    delegator.unclaimed_rewards += delegator_share

validator.unclaimed_rewards += validator_commission
```

### Treasury Allocation
```
treasury_share = tx_fees * 0.2
// Treasury funds governance proposals, development, grants
```

## Validator Selection

### Active Set Management
- **Sorted by**: total_stake = stake + delegated_stake
- **Top 100**: Active validators participate in consensus
- **Rotation**: Real-time updates when stake changes
- **Fair Selection**: Round-robin block production weighted by stake

### Entry/Exit Rules
1. New validator with stake > 100th validator → replaces lowest
2. Validator drops below 100,000 stake → forced exit
3. Slashed below minimum → immediate removal
4. Voluntary exit → 14-day lock before withdrawal

## Slashing Implementation

### Detection
- **Double Signing**: Node detects conflicting block signatures
- **Downtime**: Missed block production slots tracked
- **Invalid Block**: State transition validation failures
- **Governance**: Manual enforcement via governance proposals

### Execution
```rust
slash(validator_address: String, reason: SlashReason) -> Result<()>
```
1. Calculate slash amount based on reason
2. Deduct from validator.stake
3. Proportionally reduce delegators' effective stake
4. Record SlashEvent
5. Emit SlashEvent for monitoring
6. If stake < minimum: force exit

### Delegator Protection
- Delegators share proportional slash impact
- Can undelegate after slash event (subject to lock period)
- Rewards frozen during investigation period

## RPC Endpoints

### stake_deposit
```json
{
  "method": "stake_deposit",
  "params": {
    "amount": 100000000000000,
    "commission_rate": 10
  }
}
```

### stake_delegate
```json
{
  "method": "stake_delegate",
  "params": {
    "validator": "ACT-validator123...",
    "amount": 50000000000000
  }
}
```

### stake_unstake
```json
{
  "method": "stake_unstake",
  "params": {
    "amount": 50000000000000
  }
}
```

### stake_claim
```json
{
  "method": "stake_claim",
  "params": {}
}
```

### stake_getValidator
```json
{
  "method": "stake_getValidator",
  "params": {
    "address": "ACT-validator123..."
  }
}
```

### stake_getValidators
```json
{
  "method": "stake_getValidators",
  "params": {
    "active_only": true
  }
}
```

### stake_getDelegations
```json
{
  "method": "stake_getDelegations",
  "params": {
    "address": "ACT-delegator456..."
  }
}
```

### stake_getRewards
```json
{
  "method": "stake_getRewards",
  "params": {
    "address": "ACT-address..."
  }
}
```

## Security Considerations

1. **Stake Concentration**: Maximum 20% network stake per validator
2. **Commission Limits**: Min 5%, max 50% to prevent exploitation
3. **Lock Period**: 14 days prevents flash attacks
4. **Slashing Insurance**: Validators can purchase insurance via smart contracts
5. **Reputation System**: Track validator performance metrics

## Implementation Phases

### Phase 1: Core Staking
- Validator registration
- Stake deposit/withdrawal
- Basic reward distribution
- Lock period enforcement

### Phase 2: Delegation
- Delegation system
- Delegator rewards
- Commission handling
- Undelegation flow

### Phase 3: Slashing
- Double signing detection
- Downtime monitoring
- Slash execution
- Appeal mechanism (via governance)

### Phase 4: Advanced Features
- Stake concentration limits
- Validator reputation scores
- Auto-compounding rewards
- Liquid staking tokens

## Testing Strategy

1. **Unit Tests**: Each staking operation isolated
2. **Integration Tests**: Multi-validator scenarios
3. **Stress Tests**: 100 validators with max delegators
4. **Economic Simulation**: Reward distribution accuracy
5. **Security Audit**: Slashing edge cases, re-entrancy

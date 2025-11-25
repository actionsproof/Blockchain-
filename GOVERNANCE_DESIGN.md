# ACT Chain Governance System Design

## Overview
On-chain governance enabling token holders to propose and vote on protocol changes, parameter updates, treasury spending, and network upgrades.

## Core Principles
- **Token-Weighted Voting**: 1 ACT = 1 vote
- **Proposal Threshold**: Minimum tokens required to create proposal
- **Quorum Requirements**: Minimum participation for valid vote
- **Timelock Execution**: Delay between approval and execution
- **Veto Power**: Emergency mechanism for critical issues

## Governance Scope

### 1. Protocol Parameters
- Block time, gas limits, rewards
- Minimum validator stake
- Unstaking lock periods
- Slashing percentages

### 2. Treasury Management
- Development grants
- Marketing campaigns
- Infrastructure funding
- Community rewards

### 3. Network Upgrades
- Hard fork proposals
- Feature activations
- Protocol improvements

### 4. Emergency Actions
- Validator removal
- Smart contract pause/unpause
- Security patches

## Proposal Lifecycle

```
Create → Voting → Queued → Execution
  ↓        ↓         ↓         ↓
  7d      14d       2d      Instant

Rejected/Expired (at any stage)
```

### Phase 1: Proposal Creation (7 days review)
- **Who**: Any account with ≥ 10,000 ACT
- **Cost**: 1,000 ACT deposit (refunded if ≥ 10% quorum reached)
- **Content**: Title, description, execution actions
- **Review**: Community discussion period

### Phase 2: Voting Period (14 days)
- **Who**: All ACT holders
- **Options**: Yes, No, Abstain
- **Snapshot**: Balances at proposal creation height
- **Changes**: Cannot change vote once cast

### Phase 3: Queued (2 days timelock)
- **Condition**: Proposal passed (quorum + majority)
- **Purpose**: Security buffer, allow preparation
- **Cancellation**: Only via emergency veto

### Phase 4: Execution
- **Method**: Automated on-chain execution
- **Failure**: Proposal marked as failed (non-reversible)
- **Success**: Changes applied immediately

## Voting Rules

### Quorum Requirements
- **Standard Proposals**: 20% of circulating supply
- **Critical Changes**: 40% of circulating supply
- **Emergency Actions**: 60% of circulating supply

### Approval Thresholds
- **Standard**: >50% Yes votes (excluding Abstain)
- **Critical**: >66% Yes votes (supermajority)
- **Emergency**: >75% Yes votes

### Vote Weight
```rust
vote_power = balance_at_snapshot + delegated_stake + validator_stake
```

## Proposal Types

### 1. ParameterChange
```rust
ParameterChange {
    parameter: String,      // "min_validator_stake", "block_reward", etc.
    old_value: String,      // Current value (for verification)
    new_value: String,      // Proposed new value
}
```

### 2. TreasurySpend
```rust
TreasurySpend {
    recipient: String,      // Destination address
    amount: u64,            // ACT tokens to transfer
    purpose: String,        // Grant description
}
```

### 3. ValidatorAction
```rust
ValidatorAction {
    action: ValidatorActionType,  // Remove, Slash, Pardon
    validator_address: String,
    reason: String,
}
```

### 4. UpgradeProposal
```rust
UpgradeProposal {
    version: String,        // Target version (e.g., "2.0.0")
    activation_height: u64, // Block height for activation
    description: String,    // Upgrade details
}
```

### 5. TextProposal
```rust
TextProposal {
    content: String,        // Signaling/discussion proposal
}
```

## Data Structures

### Proposal
```rust
pub struct Proposal {
    pub id: u64,                       // Unique proposal ID
    pub proposer: String,              // Creator address
    pub proposal_type: ProposalType,   // Type of proposal
    pub title: String,                 // Short title (max 100 chars)
    pub description: String,           // Full description (max 5000 chars)
    pub created_at: u64,               // Block height
    pub voting_starts_at: u64,         // After review period
    pub voting_ends_at: u64,           // After voting period
    pub execution_eta: u64,            // After timelock (if passed)
    pub status: ProposalStatus,
    pub deposit: u64,                  // Proposer's deposit
    pub yes_votes: u64,                // Total Yes vote power
    pub no_votes: u64,                 // Total No vote power
    pub abstain_votes: u64,            // Total Abstain vote power
    pub total_supply_snapshot: u64,    // For quorum calculation
    pub executed: bool,
    pub execution_result: Option<String>,
}
```

### ProposalStatus
```rust
pub enum ProposalStatus {
    Review,          // Awaiting voting period start
    Active,          // Currently voting
    Passed,          // Met quorum + approval, queued for execution
    Rejected,        // Failed to meet approval threshold
    Expired,         // Voting ended without quorum
    Executed,        // Successfully executed
    Failed,          // Execution failed
    Vetoed,          // Emergency veto applied
}
```

### Vote
```rust
pub struct Vote {
    pub proposal_id: u64,
    pub voter: String,
    pub vote_option: VoteOption,
    pub vote_power: u64,           // Weight at snapshot height
    pub voted_at: u64,             // Block height
}

pub enum VoteOption {
    Yes,
    No,
    Abstain,
}
```

### VoterInfo
```rust
pub struct VoterInfo {
    pub balance: u64,              // Direct token holdings
    pub delegated_from: u64,       // If validator, delegations
    pub staked: u64,               // Self-staked amount
}
```

## Governance Operations

### 1. Create Proposal
```rust
create_proposal(
    proposer: String,
    proposal_type: ProposalType,
    title: String,
    description: String,
) -> Result<ProposalId>
```
- **Requires**: proposer has ≥ 10,000 ACT
- **Effects**:
  - Locks 1,000 ACT deposit
  - Creates Proposal with status=Review
  - Sets voting_starts_at = current_height + REVIEW_PERIOD
  - Sets voting_ends_at = voting_starts_at + VOTING_PERIOD

### 2. Cast Vote
```rust
cast_vote(
    proposal_id: u64,
    voter: String,
    vote_option: VoteOption,
) -> Result<()>
```
- **Requires**:
  - Proposal status = Active
  - Voter has not voted on this proposal
  - current_height <= voting_ends_at
- **Effects**:
  - Calculate voter's power at proposal snapshot
  - Record vote
  - Update proposal's yes/no/abstain counters

### 3. Finalize Proposal
```rust
finalize_proposal(proposal_id: u64) -> Result<()>
```
- **Requires**: current_height > voting_ends_at
- **Effects**:
  - Calculate quorum: (yes + no + abstain) / total_supply_snapshot
  - Calculate approval: yes / (yes + no)
  - If quorum met AND approval threshold met:
    - status = Passed
    - execution_eta = current_height + TIMELOCK_PERIOD
    - Refund proposer's deposit
  - Else:
    - status = Rejected or Expired
    - Burn proposer's deposit (if < 10% quorum)

### 4. Execute Proposal
```rust
execute_proposal(proposal_id: u64) -> Result<()>
```
- **Requires**:
  - status = Passed
  - current_height >= execution_eta
- **Effects**:
  - Execute proposal actions (parameter change, treasury spend, etc.)
  - Mark status = Executed (or Failed if execution error)
  - Emit ExecutionResult event

### 5. Veto Proposal (Emergency)
```rust
veto_proposal(proposal_id: u64, veto_council_signature: Signature) -> Result<()>
```
- **Requires**: Multi-sig from 3/5 veto council members
- **Effects**:
  - status = Vetoed
  - Cancel execution
  - Refund proposer's deposit

## Governance Parameters

```rust
pub const PROPOSAL_DEPOSIT: u64 = 1_000_000_000_000;     // 1,000 ACT
pub const MIN_PROPOSAL_BALANCE: u64 = 10_000_000_000_000; // 10,000 ACT
pub const REVIEW_PERIOD: u64 = 50_400;                   // 7 days (~2s blocks)
pub const VOTING_PERIOD: u64 = 100_800;                  // 14 days
pub const TIMELOCK_PERIOD: u64 = 14_400;                 // 2 days
pub const STANDARD_QUORUM: f64 = 0.20;                   // 20%
pub const CRITICAL_QUORUM: f64 = 0.40;                   // 40%
pub const EMERGENCY_QUORUM: f64 = 0.60;                  // 60%
pub const STANDARD_THRESHOLD: f64 = 0.50;                // >50%
pub const CRITICAL_THRESHOLD: f64 = 0.66;                // >66%
pub const EMERGENCY_THRESHOLD: f64 = 0.75;               // >75%
```

## Vote Power Calculation

```rust
fn calculate_vote_power(voter: &str, snapshot_height: u64) -> u64 {
    let balance = state.get_balance_at(voter, snapshot_height);
    
    let validator_stake = if let Some(validator) = staking.get_validator(voter) {
        validator.stake + validator.delegated_stake
    } else {
        0
    };
    
    let delegator_stake = staking.get_delegations(voter)
        .iter()
        .map(|d| d.amount)
        .sum();
    
    balance + validator_stake + delegator_stake
}
```

## Proposal Execution Examples

### Parameter Change
```rust
if proposal_type == ProposalType::ParameterChange {
    match parameter.as_str() {
        "block_reward" => {
            staking.set_block_reward(new_value.parse()?);
        }
        "min_validator_stake" => {
            staking.set_min_stake(new_value.parse()?);
        }
        _ => return Err("Unknown parameter"),
    }
}
```

### Treasury Spend
```rust
if proposal_type == ProposalType::TreasurySpend {
    state.transfer(
        TREASURY_ADDRESS,
        &proposal.recipient,
        proposal.amount,
    )?;
}
```

### Validator Action
```rust
if proposal_type == ProposalType::ValidatorAction {
    match action {
        ValidatorActionType::Remove => {
            staking.force_validator_exit(&validator_address)?;
        }
        ValidatorActionType::Slash => {
            staking.slash(validator_address, SlashReason::GovernanceViolation, None)?;
        }
        _ => {}
    }
}
```

## RPC Endpoints

### gov_propose
```json
{
  "method": "gov_propose",
  "params": {
    "proposer": "ACT-proposer123...",
    "proposal_type": "TreasurySpend",
    "title": "Community Events Budget",
    "description": "Fund 10 global meetups for Q1 2025",
    "details": {
      "recipient": "ACT-events456...",
      "amount": 50000000000000,
      "purpose": "Q1 2025 community events"
    }
  }
}
```

### gov_vote
```json
{
  "method": "gov_vote",
  "params": {
    "proposal_id": 1,
    "voter": "ACT-voter789...",
    "vote_option": "Yes"
  }
}
```

### gov_getProposal
```json
{
  "method": "gov_getProposal",
  "params": {
    "proposal_id": 1
  }
}
```

### gov_listProposals
```json
{
  "method": "gov_listProposals",
  "params": {
    "status": "Active",
    "limit": 10,
    "offset": 0
  }
}
```

### gov_getVote
```json
{
  "method": "gov_getVote",
  "params": {
    "proposal_id": 1,
    "voter": "ACT-voter789..."
  }
}
```

### gov_getVotingPower
```json
{
  "method": "gov_getVotingPower",
  "params": {
    "address": "ACT-voter789...",
    "snapshot_height": 100000
  }
}
```

### gov_getTallyResult
```json
{
  "method": "gov_getTallyResult",
  "params": {
    "proposal_id": 1
  }
}
```

## Security Considerations

1. **Proposal Spam**: 1,000 ACT deposit + 10,000 ACT minimum balance prevents spam
2. **Vote Buying**: Snapshot prevents buying tokens mid-vote
3. **Flash Attacks**: Timelock provides security buffer
4. **Governance Attacks**: Quorum requirements prevent low-participation attacks
5. **Veto Mechanism**: Emergency override for critical security issues

## Integration with Staking

- Validators automatically participate (delegated stake counts as voting power)
- Delegators can vote independently (their own balance)
- Delegation does NOT transfer voting rights (only validator gets delegated power)

## Implementation Phases

### Phase 1: Basic Proposals
- TextProposal and ParameterChange
- Voting mechanism
- Finalization logic

### Phase 2: Treasury
- Treasury spend proposals
- Grant management
- Multi-sig treasury control

### Phase 3: Advanced Actions
- Validator actions
- Upgrade proposals
- Emergency veto system

### Phase 4: UI/UX
- Web-based governance dashboard
- Vote delegation
- Proposal templates

## Testing Strategy

1. **Unit Tests**: Each governance operation isolated
2. **Integration Tests**: Full proposal lifecycle
3. **Security Audits**: Vote manipulation attempts
4. **Economic Simulation**: Quorum and threshold tuning
5. **Governance Games**: Community test governance on testnet

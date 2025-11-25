use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Proposal deposit (1,000 ACT)
pub const PROPOSAL_DEPOSIT: u64 = 1_000_000_000_000;

/// Minimum balance to create proposal (10,000 ACT)
pub const MIN_PROPOSAL_BALANCE: u64 = 10_000_000_000_000;

/// Review period before voting starts (~7 days at 2s blocks)
pub const REVIEW_PERIOD: u64 = 302_400;

/// Voting period duration (~14 days at 2s blocks)
pub const VOTING_PERIOD: u64 = 604_800;

/// Timelock period after passing (~2 days at 2s blocks)
pub const TIMELOCK_PERIOD: u64 = 86_400;

/// Standard quorum (20%)
pub const STANDARD_QUORUM: f64 = 0.20;

/// Critical quorum (40%)
pub const CRITICAL_QUORUM: f64 = 0.40;

/// Emergency quorum (60%)
pub const EMERGENCY_QUORUM: f64 = 0.60;

/// Standard approval threshold (>50%)
pub const STANDARD_THRESHOLD: f64 = 0.50;

/// Critical approval threshold (>66%)
pub const CRITICAL_THRESHOLD: f64 = 0.66;

/// Emergency approval threshold (>75%)
pub const EMERGENCY_THRESHOLD: f64 = 0.75;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    ParameterChange {
        parameter: String,
        old_value: String,
        new_value: String,
    },
    TreasurySpend {
        recipient: String,
        amount: u64,
        purpose: String,
    },
    ValidatorAction {
        action: ValidatorActionType,
        validator_address: String,
        reason: String,
    },
    UpgradeProposal {
        version: String,
        activation_height: u64,
        description: String,
    },
    TextProposal {
        content: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidatorActionType {
    Remove,
    Slash,
    Pardon,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Review,
    Active,
    Passed,
    Rejected,
    Expired,
    Executed,
    Failed,
    Vetoed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub proposer: String,
    pub proposal_type: ProposalType,
    pub title: String,
    pub description: String,
    pub created_at: u64,
    pub voting_starts_at: u64,
    pub voting_ends_at: u64,
    pub execution_eta: u64,
    pub status: ProposalStatus,
    pub deposit: u64,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub abstain_votes: u64,
    pub total_supply_snapshot: u64,
    pub executed: bool,
    pub execution_result: Option<String>,
}

impl Proposal {
    pub fn new(
        id: u64,
        proposer: String,
        proposal_type: ProposalType,
        title: String,
        description: String,
        current_height: u64,
        total_supply: u64,
    ) -> Self {
        Self {
            id,
            proposer,
            proposal_type,
            title,
            description,
            created_at: current_height,
            voting_starts_at: current_height + REVIEW_PERIOD,
            voting_ends_at: current_height + REVIEW_PERIOD + VOTING_PERIOD,
            execution_eta: 0,
            status: ProposalStatus::Review,
            deposit: PROPOSAL_DEPOSIT,
            yes_votes: 0,
            no_votes: 0,
            abstain_votes: 0,
            total_supply_snapshot: total_supply,
            executed: false,
            execution_result: None,
        }
    }

    pub fn total_votes(&self) -> u64 {
        self.yes_votes + self.no_votes + self.abstain_votes
    }

    pub fn quorum_percentage(&self) -> f64 {
        if self.total_supply_snapshot == 0 {
            return 0.0;
        }
        self.total_votes() as f64 / self.total_supply_snapshot as f64
    }

    pub fn approval_percentage(&self) -> f64 {
        let decisive_votes = self.yes_votes + self.no_votes;
        if decisive_votes == 0 {
            return 0.0;
        }
        self.yes_votes as f64 / decisive_votes as f64
    }

    pub fn required_quorum(&self) -> f64 {
        match &self.proposal_type {
            ProposalType::ParameterChange { .. } => CRITICAL_QUORUM,
            ProposalType::TreasurySpend { .. } => STANDARD_QUORUM,
            ProposalType::ValidatorAction { .. } => CRITICAL_QUORUM,
            ProposalType::UpgradeProposal { .. } => EMERGENCY_QUORUM,
            ProposalType::TextProposal { .. } => STANDARD_QUORUM,
        }
    }

    pub fn required_threshold(&self) -> f64 {
        match &self.proposal_type {
            ProposalType::ParameterChange { .. } => CRITICAL_THRESHOLD,
            ProposalType::TreasurySpend { .. } => STANDARD_THRESHOLD,
            ProposalType::ValidatorAction { .. } => CRITICAL_THRESHOLD,
            ProposalType::UpgradeProposal { .. } => EMERGENCY_THRESHOLD,
            ProposalType::TextProposal { .. } => STANDARD_THRESHOLD,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteOption {
    Yes,
    No,
    Abstain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: u64,
    pub voter: String,
    pub vote_option: VoteOption,
    pub vote_power: u64,
    pub voted_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TallyResult {
    pub proposal_id: u64,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub abstain_votes: u64,
    pub total_votes: u64,
    pub quorum_percentage: f64,
    pub approval_percentage: f64,
    pub required_quorum: f64,
    pub required_threshold: f64,
    pub quorum_met: bool,
    pub threshold_met: bool,
    pub passed: bool,
}

pub struct GovernanceManager {
    proposals: HashMap<u64, Proposal>,
    votes: HashMap<u64, HashMap<String, Vote>>, // proposal_id -> (voter -> vote)
    next_proposal_id: u64,
    current_height: u64,
}

impl GovernanceManager {
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            votes: HashMap::new(),
            next_proposal_id: 1,
            current_height: 0,
        }
    }

    pub fn set_block_height(&mut self, height: u64) {
        self.current_height = height;
    }

    /// Create a new proposal
    pub fn create_proposal(
        &mut self,
        proposer: String,
        proposal_type: ProposalType,
        title: String,
        description: String,
        proposer_balance: u64,
        total_supply: u64,
    ) -> Result<u64, String> {
        // Validate proposer balance
        if proposer_balance < MIN_PROPOSAL_BALANCE {
            return Err(format!(
                "Insufficient balance. Need at least {} ACT to create proposal",
                MIN_PROPOSAL_BALANCE / 1_000_000_000
            ));
        }

        // Validate title and description
        if title.is_empty() || title.len() > 100 {
            return Err("Title must be 1-100 characters".to_string());
        }

        if description.is_empty() || description.len() > 5000 {
            return Err("Description must be 1-5000 characters".to_string());
        }

        let proposal_id = self.next_proposal_id;
        self.next_proposal_id += 1;

        let proposal = Proposal::new(
            proposal_id,
            proposer,
            proposal_type,
            title,
            description,
            self.current_height,
            total_supply,
        );

        self.proposals.insert(proposal_id, proposal);
        self.votes.insert(proposal_id, HashMap::new());

        Ok(proposal_id)
    }

    /// Cast a vote on a proposal
    pub fn cast_vote(
        &mut self,
        proposal_id: u64,
        voter: String,
        vote_option: VoteOption,
        vote_power: u64,
    ) -> Result<(), String> {
        let proposal = self
            .proposals
            .get_mut(&proposal_id)
            .ok_or("Proposal not found")?;

        // Check proposal status
        if proposal.status != ProposalStatus::Active {
            return Err(format!("Proposal is not active (status: {:?})", proposal.status));
        }

        // Check if voting period has started
        if self.current_height < proposal.voting_starts_at {
            return Err("Voting has not started yet".to_string());
        }

        // Check if voting period has ended
        if self.current_height > proposal.voting_ends_at {
            return Err("Voting period has ended".to_string());
        }

        // Check if already voted
        let proposal_votes = self.votes.get(&proposal_id).ok_or("Vote map not found")?;
        if proposal_votes.contains_key(&voter) {
            return Err("Already voted on this proposal".to_string());
        }

        // Record vote
        let vote = Vote {
            proposal_id,
            voter: voter.clone(),
            vote_option: vote_option.clone(),
            vote_power,
            voted_at: self.current_height,
        };

        // Update vote counts
        match vote_option {
            VoteOption::Yes => proposal.yes_votes += vote_power,
            VoteOption::No => proposal.no_votes += vote_power,
            VoteOption::Abstain => proposal.abstain_votes += vote_power,
        }

        self.votes
            .get_mut(&proposal_id)
            .unwrap()
            .insert(voter, vote);

        Ok(())
    }

    /// Update proposal status based on block height
    pub fn update_proposal_status(&mut self, proposal_id: u64) -> Result<(), String> {
        let proposal = self
            .proposals
            .get_mut(&proposal_id)
            .ok_or("Proposal not found")?;

        match proposal.status {
            ProposalStatus::Review => {
                if self.current_height >= proposal.voting_starts_at {
                    proposal.status = ProposalStatus::Active;
                }
            }
            ProposalStatus::Active => {
                if self.current_height > proposal.voting_ends_at {
                    self.finalize_proposal(proposal_id)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Finalize proposal after voting ends
    fn finalize_proposal(&mut self, proposal_id: u64) -> Result<(), String> {
        let proposal = self
            .proposals
            .get_mut(&proposal_id)
            .ok_or("Proposal not found")?;

        let quorum = proposal.quorum_percentage();
        let approval = proposal.approval_percentage();
        let required_quorum = proposal.required_quorum();
        let required_threshold = proposal.required_threshold();

        let quorum_met = quorum >= required_quorum;
        let threshold_met = approval > required_threshold;

        if quorum_met && threshold_met {
            proposal.status = ProposalStatus::Passed;
            proposal.execution_eta = self.current_height + TIMELOCK_PERIOD;
        } else if !quorum_met {
            proposal.status = ProposalStatus::Expired;
        } else {
            proposal.status = ProposalStatus::Rejected;
        }

        Ok(())
    }

    /// Execute a passed proposal
    pub fn execute_proposal(&mut self, proposal_id: u64) -> Result<String, String> {
        let proposal = self
            .proposals
            .get_mut(&proposal_id)
            .ok_or("Proposal not found")?;

        // Check status
        if proposal.status != ProposalStatus::Passed {
            return Err(format!("Proposal is not in passed status (status: {:?})", proposal.status));
        }

        // Check timelock
        if self.current_height < proposal.execution_eta {
            return Err(format!(
                "Timelock not expired. Execution available at block {}",
                proposal.execution_eta
            ));
        }

        // Check if already executed
        if proposal.executed {
            return Err("Proposal already executed".to_string());
        }

        // Mark as executed
        proposal.executed = true;
        proposal.status = ProposalStatus::Executed;

        // Return execution details (actual execution happens in node/state manager)
        let result = match &proposal.proposal_type {
            ProposalType::ParameterChange { parameter, new_value, .. } => {
                format!("Parameter '{}' changed to '{}'", parameter, new_value)
            }
            ProposalType::TreasurySpend { recipient, amount, .. } => {
                format!("Treasury spend: {} ACT to {}", amount / 1_000_000_000, recipient)
            }
            ProposalType::ValidatorAction { action, validator_address, .. } => {
                format!("Validator action: {:?} for {}", action, validator_address)
            }
            ProposalType::UpgradeProposal { version, .. } => {
                format!("Upgrade to version {}", version)
            }
            ProposalType::TextProposal { .. } => {
                "Text proposal passed (signaling only)".to_string()
            }
        };

        proposal.execution_result = Some(result.clone());

        Ok(result)
    }

    /// Veto a proposal (emergency)
    pub fn veto_proposal(&mut self, proposal_id: u64) -> Result<(), String> {
        let proposal = self
            .proposals
            .get_mut(&proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.executed {
            return Err("Cannot veto executed proposal".to_string());
        }

        proposal.status = ProposalStatus::Vetoed;

        Ok(())
    }

    /// Get proposal by ID
    pub fn get_proposal(&self, proposal_id: u64) -> Option<Proposal> {
        self.proposals.get(&proposal_id).cloned()
    }

    /// List all proposals
    pub fn list_proposals(&self, status_filter: Option<ProposalStatus>) -> Vec<Proposal> {
        let mut proposals: Vec<Proposal> = self
            .proposals
            .values()
            .filter(|p| {
                if let Some(ref status) = status_filter {
                    &p.status == status
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        proposals.sort_by(|a, b| b.id.cmp(&a.id));

        proposals
    }

    /// Get vote for a specific proposal and voter
    pub fn get_vote(&self, proposal_id: u64, voter: &str) -> Option<Vote> {
        self.votes
            .get(&proposal_id)
            .and_then(|votes| votes.get(voter).cloned())
    }

    /// Get all votes for a proposal
    pub fn get_proposal_votes(&self, proposal_id: u64) -> Vec<Vote> {
        self.votes
            .get(&proposal_id)
            .map(|votes| votes.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Get tally result for a proposal
    pub fn get_tally_result(&self, proposal_id: u64) -> Result<TallyResult, String> {
        let proposal = self
            .proposals
            .get(&proposal_id)
            .ok_or("Proposal not found")?;

        let quorum_percentage = proposal.quorum_percentage();
        let approval_percentage = proposal.approval_percentage();
        let required_quorum = proposal.required_quorum();
        let required_threshold = proposal.required_threshold();

        let quorum_met = quorum_percentage >= required_quorum;
        let threshold_met = approval_percentage > required_threshold;
        let passed = quorum_met && threshold_met;

        Ok(TallyResult {
            proposal_id,
            yes_votes: proposal.yes_votes,
            no_votes: proposal.no_votes,
            abstain_votes: proposal.abstain_votes,
            total_votes: proposal.total_votes(),
            quorum_percentage,
            approval_percentage,
            required_quorum,
            required_threshold,
            quorum_met,
            threshold_met,
            passed,
        })
    }

    /// Get total number of proposals
    pub fn get_proposal_count(&self) -> u64 {
        self.proposals.len() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_proposal() {
        let mut gov = GovernanceManager::new();
        gov.set_block_height(1000);

        let result = gov.create_proposal(
            "ACT-proposer1".to_string(),
            ProposalType::TextProposal {
                content: "Test proposal".to_string(),
            },
            "Test Title".to_string(),
            "Test description for the proposal".to_string(),
            MIN_PROPOSAL_BALANCE,
            1_000_000_000_000_000,
        );

        assert!(result.is_ok());
        let proposal_id = result.unwrap();
        assert_eq!(proposal_id, 1);

        let proposal = gov.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Review);
        assert_eq!(proposal.voting_starts_at, 1000 + REVIEW_PERIOD);
    }

    #[test]
    fn test_insufficient_balance() {
        let mut gov = GovernanceManager::new();

        let result = gov.create_proposal(
            "ACT-proposer1".to_string(),
            ProposalType::TextProposal {
                content: "Test".to_string(),
            },
            "Test".to_string(),
            "Test description".to_string(),
            MIN_PROPOSAL_BALANCE - 1,
            1_000_000_000_000_000,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_vote_casting() {
        let mut gov = GovernanceManager::new();
        gov.set_block_height(1000);

        let proposal_id = gov
            .create_proposal(
                "ACT-proposer1".to_string(),
                ProposalType::TextProposal {
                    content: "Test".to_string(),
                },
                "Test".to_string(),
                "Test description".to_string(),
                MIN_PROPOSAL_BALANCE,
                1_000_000_000_000_000,
            )
            .unwrap();

        // Move to active voting period
        gov.set_block_height(1000 + REVIEW_PERIOD + 1);
        gov.update_proposal_status(proposal_id).unwrap();

        let result = gov.cast_vote(
            proposal_id,
            "ACT-voter1".to_string(),
            VoteOption::Yes,
            100_000_000_000_000,
        );

        assert!(result.is_ok());

        let proposal = gov.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.yes_votes, 100_000_000_000_000);
    }

    #[test]
    fn test_double_vote_prevention() {
        let mut gov = GovernanceManager::new();
        gov.set_block_height(1000);

        let proposal_id = gov
            .create_proposal(
                "ACT-proposer1".to_string(),
                ProposalType::TextProposal {
                    content: "Test".to_string(),
                },
                "Test".to_string(),
                "Test description".to_string(),
                MIN_PROPOSAL_BALANCE,
                1_000_000_000_000_000,
            )
            .unwrap();

        gov.set_block_height(1000 + REVIEW_PERIOD + 1);
        gov.update_proposal_status(proposal_id).unwrap();

        gov.cast_vote(
            proposal_id,
            "ACT-voter1".to_string(),
            VoteOption::Yes,
            100_000_000_000_000,
        )
        .unwrap();

        let result = gov.cast_vote(
            proposal_id,
            "ACT-voter1".to_string(),
            VoteOption::No,
            100_000_000_000_000,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_proposal_finalization() {
        let mut gov = GovernanceManager::new();
        gov.set_block_height(1000);

        let total_supply = 1_000_000_000_000_000;
        let proposal_id = gov
            .create_proposal(
                "ACT-proposer1".to_string(),
                ProposalType::TextProposal {
                    content: "Test".to_string(),
                },
                "Test".to_string(),
                "Test description".to_string(),
                MIN_PROPOSAL_BALANCE,
                total_supply,
            )
            .unwrap();

        // Move to voting period
        gov.set_block_height(1000 + REVIEW_PERIOD + 1);
        gov.update_proposal_status(proposal_id).unwrap();

        // Cast enough votes to meet quorum and threshold
        let vote_amount = (total_supply as f64 * 0.25) as u64; // 25% quorum
        gov.cast_vote(
            proposal_id,
            "ACT-voter1".to_string(),
            VoteOption::Yes,
            vote_amount,
        )
        .unwrap();

        // Move past voting period
        gov.set_block_height(1000 + REVIEW_PERIOD + VOTING_PERIOD + 1);
        gov.update_proposal_status(proposal_id).unwrap();

        let proposal = gov.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Passed);
    }

    #[test]
    fn test_proposal_execution() {
        let mut gov = GovernanceManager::new();
        gov.set_block_height(1000);

        let total_supply = 1_000_000_000_000_000;
        let proposal_id = gov
            .create_proposal(
                "ACT-proposer1".to_string(),
                ProposalType::ParameterChange {
                    parameter: "block_reward".to_string(),
                    old_value: "50".to_string(),
                    new_value: "100".to_string(),
                },
                "Increase Block Reward".to_string(),
                "Proposal to increase block reward to 100 ACT".to_string(),
                MIN_PROPOSAL_BALANCE,
                total_supply,
            )
            .unwrap();

        // Move through lifecycle
        gov.set_block_height(1000 + REVIEW_PERIOD + 1);
        gov.update_proposal_status(proposal_id).unwrap();

        let vote_amount = (total_supply as f64 * 0.70) as u64; // 70% for critical threshold
        gov.cast_vote(
            proposal_id,
            "ACT-voter1".to_string(),
            VoteOption::Yes,
            vote_amount,
        )
        .unwrap();

        gov.set_block_height(1000 + REVIEW_PERIOD + VOTING_PERIOD + 1);
        gov.update_proposal_status(proposal_id).unwrap();

        // Move past timelock
        gov.set_block_height(1000 + REVIEW_PERIOD + VOTING_PERIOD + TIMELOCK_PERIOD + 1);

        let result = gov.execute_proposal(proposal_id);
        assert!(result.is_ok());

        let proposal = gov.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Executed);
        assert!(proposal.executed);
    }
}

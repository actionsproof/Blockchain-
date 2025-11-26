use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OracleError {
    #[error("Feed not found: {0}")]
    FeedNotFound(String),
    #[error("Provider not registered: {0}")]
    ProviderNotRegistered(String),
    #[error("Insufficient stake")]
    InsufficientStake,
    #[error("Invalid price data")]
    InvalidPriceData,
    #[error("Dispute period active")]
    DisputePeriodActive,
    #[error("Not enough data sources")]
    NotEnoughDataSources,
    #[error("Price deviation too high")]
    PriceDeviationTooHigh,
    #[error("Provider not authorized")]
    ProviderNotAuthorized,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataFeedStatus {
    Active,
    Paused,
    Disputed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub price: u128,        // Price in smallest unit (e.g., cents for USD)
    pub decimals: u8,       // Number of decimals
    pub timestamp: u64,
    pub provider: String,
    pub confidence: u64,    // Confidence score 0-10000 (basis points)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFeed {
    pub feed_id: String,
    pub description: String,
    pub status: DataFeedStatus,
    pub latest_price: Option<PriceData>,
    pub price_history: Vec<PriceData>,
    pub providers: Vec<String>,
    pub min_providers: usize,
    pub max_price_deviation: u64, // Maximum allowed deviation in basis points
    pub update_frequency: u64,    // Minimum seconds between updates
    pub last_update: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleProvider {
    pub address: String,
    pub stake: u128,
    pub reputation_score: u64,    // 0-10000 basis points
    pub successful_updates: u64,
    pub failed_updates: u64,
    pub total_disputes: u64,
    pub slashed_amount: u128,
    pub authorized_feeds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dispute {
    pub dispute_id: u64,
    pub feed_id: String,
    pub price_data: PriceData,
    pub challenger: String,
    pub reason: String,
    pub timestamp: u64,
    pub resolved: bool,
    pub result: Option<DisputeResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DisputeResult {
    ValidPrice,      // Dispute rejected, price was valid
    InvalidPrice,    // Dispute accepted, price was invalid
    Inconclusive,    // Not enough evidence
}

pub struct OracleNetwork {
    pub feeds: HashMap<String, DataFeed>,
    pub providers: HashMap<String, OracleProvider>,
    pub disputes: HashMap<u64, Dispute>,
    pub next_dispute_id: u64,
    pub min_provider_stake: u128,
    pub dispute_period: u64,
    pub slash_percentage: u64, // In basis points
}

impl OracleNetwork {
    pub fn new(min_provider_stake: u128, dispute_period: u64, slash_percentage: u64) -> Self {
        Self {
            feeds: HashMap::new(),
            providers: HashMap::new(),
            disputes: HashMap::new(),
            next_dispute_id: 1,
            min_provider_stake,
            dispute_period,
            slash_percentage,
        }
    }

    pub fn register_provider(
        &mut self,
        address: String,
        stake: u128,
    ) -> Result<(), OracleError> {
        if stake < self.min_provider_stake {
            return Err(OracleError::InsufficientStake);
        }

        let provider = OracleProvider {
            address: address.clone(),
            stake,
            reputation_score: 5000, // Start at 50%
            successful_updates: 0,
            failed_updates: 0,
            total_disputes: 0,
            slashed_amount: 0,
            authorized_feeds: Vec::new(),
        };

        self.providers.insert(address, provider);
        Ok(())
    }

    pub fn create_feed(
        &mut self,
        feed_id: String,
        description: String,
        min_providers: usize,
        max_price_deviation: u64,
        update_frequency: u64,
    ) -> Result<(), OracleError> {
        let feed = DataFeed {
            feed_id: feed_id.clone(),
            description,
            status: DataFeedStatus::Active,
            latest_price: None,
            price_history: Vec::new(),
            providers: Vec::new(),
            min_providers,
            max_price_deviation,
            update_frequency,
            last_update: 0,
        };

        self.feeds.insert(feed_id, feed);
        Ok(())
    }

    pub fn authorize_provider(
        &mut self,
        provider: &str,
        feed_id: &str,
    ) -> Result<(), OracleError> {
        let provider_data = self
            .providers
            .get_mut(provider)
            .ok_or_else(|| OracleError::ProviderNotRegistered(provider.to_string()))?;

        if !provider_data.authorized_feeds.contains(&feed_id.to_string()) {
            provider_data.authorized_feeds.push(feed_id.to_string());
        }

        let feed = self
            .feeds
            .get_mut(feed_id)
            .ok_or_else(|| OracleError::FeedNotFound(feed_id.to_string()))?;

        if !feed.providers.contains(&provider.to_string()) {
            feed.providers.push(provider.to_string());
        }

        Ok(())
    }

    pub fn submit_price(
        &mut self,
        feed_id: &str,
        price_data: PriceData,
        current_time: u64,
    ) -> Result<(), OracleError> {
        // Verify provider is authorized
        let provider = self
            .providers
            .get(&price_data.provider)
            .ok_or_else(|| OracleError::ProviderNotRegistered(price_data.provider.clone()))?;

        if !provider.authorized_feeds.contains(&feed_id.to_string()) {
            return Err(OracleError::ProviderNotAuthorized);
        }

        let feed = self
            .feeds
            .get_mut(feed_id)
            .ok_or_else(|| OracleError::FeedNotFound(feed_id.to_string()))?;

        // Check if update frequency is respected
        if current_time - feed.last_update < feed.update_frequency {
            return Err(OracleError::InvalidPriceData);
        }

        // Check price deviation if there's a previous price
        let max_deviation = feed.max_price_deviation;
        if let Some(ref latest) = feed.latest_price {
            let deviation = Self::calculate_deviation_static(price_data.price, latest.price);
            if deviation > max_deviation {
                return Err(OracleError::PriceDeviationTooHigh);
            }
        }

        // Add to history
        feed.price_history.push(price_data.clone());

        // Keep only last 100 prices
        if feed.price_history.len() > 100 {
            feed.price_history.remove(0);
        }

        // Update latest price
        feed.latest_price = Some(price_data);
        feed.last_update = current_time;

        // Update provider stats
        let first_provider = feed.providers.get(0).cloned();
        if let Some(provider_addr) = first_provider {
            if let Some(provider) = self.providers.get_mut(&provider_addr) {
                provider.successful_updates += 1;
                Self::update_reputation_static(provider);
            }
        }

        Ok(())
    }

    pub fn get_aggregated_price(&self, feed_id: &str) -> Result<PriceData, OracleError> {
        let feed = self
            .feeds
            .get(feed_id)
            .ok_or_else(|| OracleError::FeedNotFound(feed_id.to_string()))?;

        // Get recent prices from all authorized providers
        let mut prices: Vec<u128> = Vec::new();
        let mut total_confidence = 0u64;

        for provider_addr in &feed.providers {
            if let Some(provider) = self.providers.get(provider_addr) {
                // Use provider's reputation as weight
                if let Some(ref price_data) = feed.latest_price {
                    if price_data.provider == *provider_addr {
                        prices.push(price_data.price);
                        total_confidence += provider.reputation_score;
                    }
                }
            }
        }

        if prices.is_empty() {
            return Err(OracleError::NotEnoughDataSources);
        }

        // Calculate median price (more resistant to outliers than mean)
        prices.sort_unstable();
        let median_price = if prices.len() % 2 == 0 {
            (prices[prices.len() / 2 - 1] + prices[prices.len() / 2]) / 2
        } else {
            prices[prices.len() / 2]
        };

        let avg_confidence = if !prices.is_empty() {
            total_confidence / prices.len() as u64
        } else {
            0
        };

        Ok(PriceData {
            price: median_price,
            decimals: feed.latest_price.as_ref().map(|p| p.decimals).unwrap_or(8),
            timestamp: feed.last_update,
            provider: "aggregated".to_string(),
            confidence: avg_confidence,
        })
    }

    pub fn dispute_price(
        &mut self,
        feed_id: &str,
        challenger: String,
        reason: String,
        current_time: u64,
    ) -> Result<u64, OracleError> {
        let feed = self
            .feeds
            .get(feed_id)
            .ok_or_else(|| OracleError::FeedNotFound(feed_id.to_string()))?;

        let price_data = feed
            .latest_price
            .clone()
            .ok_or(OracleError::InvalidPriceData)?;

        let dispute_id = self.next_dispute_id;
        self.next_dispute_id += 1;

        let dispute = Dispute {
            dispute_id,
            feed_id: feed_id.to_string(),
            price_data,
            challenger,
            reason,
            timestamp: current_time,
            resolved: false,
            result: None,
        };

        self.disputes.insert(dispute_id, dispute);

        // Mark feed as disputed
        if let Some(feed) = self.feeds.get_mut(feed_id) {
            feed.status = DataFeedStatus::Disputed;
        }

        Ok(dispute_id)
    }

    pub fn resolve_dispute(
        &mut self,
        dispute_id: u64,
        result: DisputeResult,
    ) -> Result<(), OracleError> {
        let dispute = self.disputes.get_mut(&dispute_id).unwrap();

        dispute.resolved = true;
        dispute.result = Some(result.clone());

        // Handle slashing if price was invalid
        if result == DisputeResult::InvalidPrice {
            if let Some(provider) = self.providers.get_mut(&dispute.price_data.provider) {
                let slash_amount = (provider.stake * self.slash_percentage as u128) / 10000;
                provider.slashed_amount += slash_amount;
                provider.stake -= slash_amount;
                provider.failed_updates += 1;
                provider.total_disputes += 1;
                Self::update_reputation_static(provider);
            }
        }

        // Update feed status
        if let Some(feed) = self.feeds.get_mut(&dispute.feed_id) {
            feed.status = DataFeedStatus::Active;
        }

        Ok(())
    }

    fn calculate_deviation_static(new_price: u128, old_price: u128) -> u64 {
        if old_price == 0 {
            return 0;
        }

        let diff = if new_price > old_price {
            new_price - old_price
        } else {
            old_price - new_price
        };

        // Return deviation in basis points
        ((diff * 10000) / old_price) as u64
    }

    fn update_reputation_static(provider: &mut OracleProvider) {
        let total_updates = provider.successful_updates + provider.failed_updates;
        if total_updates == 0 {
            return;
        }

        // Calculate success rate
        let success_rate = (provider.successful_updates * 10000) / total_updates;

        // Factor in disputes (reduce reputation for disputes)
        let dispute_penalty = provider.total_disputes * 100; // -1% per dispute

        // New reputation = success_rate - dispute_penalty
        provider.reputation_score = if success_rate > dispute_penalty {
            success_rate - dispute_penalty
        } else {
            0
        };

        // Cap at 10000 (100%)
        if provider.reputation_score > 10000 {
            provider.reputation_score = 10000;
        }
    }

    pub fn get_feed(&self, feed_id: &str) -> Option<&DataFeed> {
        self.feeds.get(feed_id)
    }

    pub fn get_provider(&self, address: &str) -> Option<&OracleProvider> {
        self.providers.get(address)
    }

    pub fn get_dispute(&self, dispute_id: u64) -> Option<&Dispute> {
        self.disputes.get(&dispute_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_registration() {
        let mut oracle = OracleNetwork::new(1000, 3600, 500); // 1000 min stake, 1h dispute, 5% slash

        assert!(oracle
            .register_provider("provider1".to_string(), 2000)
            .is_ok());
        assert!(oracle
            .register_provider("provider2".to_string(), 500)
            .is_err()); // Insufficient stake

        let provider = oracle.get_provider("provider1").unwrap();
        assert_eq!(provider.stake, 2000);
        assert_eq!(provider.reputation_score, 5000); // 50% initial
    }

    #[test]
    fn test_feed_creation() {
        let mut oracle = OracleNetwork::new(1000, 3600, 500);

        assert!(oracle
            .create_feed("ETH/USD".to_string(), "Ethereum price".to_string(), 3, 500, 60)
            .is_ok());

        let feed = oracle.get_feed("ETH/USD").unwrap();
        assert_eq!(feed.feed_id, "ETH/USD");
        assert_eq!(feed.min_providers, 3);
        assert_eq!(feed.max_price_deviation, 500); // 5%
    }

    #[test]
    fn test_price_submission() {
        let mut oracle = OracleNetwork::new(1000, 3600, 500);

        oracle
            .register_provider("provider1".to_string(), 2000)
            .unwrap();
        oracle
            .create_feed("BTC/USD".to_string(), "Bitcoin price".to_string(), 1, 1000, 60)
            .unwrap();
        oracle.authorize_provider("provider1", "BTC/USD").unwrap();

        let price_data = PriceData {
            price: 50000_00000000, // $50,000 with 8 decimals
            decimals: 8,
            timestamp: 1000,
            provider: "provider1".to_string(),
            confidence: 9000,
        };

        assert!(oracle.submit_price("BTC/USD", price_data, 1000).is_ok());

        let feed = oracle.get_feed("BTC/USD").unwrap();
        assert!(feed.latest_price.is_some());
        assert_eq!(feed.latest_price.as_ref().unwrap().price, 50000_00000000);
    }

    #[test]
    fn test_price_deviation_check() {
        let mut oracle = OracleNetwork::new(1000, 3600, 500);

        oracle
            .register_provider("provider1".to_string(), 2000)
            .unwrap();
        oracle
            .create_feed("ETH/USD".to_string(), "Ethereum price".to_string(), 1, 500, 60)
            .unwrap(); // 5% max deviation
        oracle.authorize_provider("provider1", "ETH/USD").unwrap();

        // First price
        let price1 = PriceData {
            price: 2000_00000000,
            decimals: 8,
            timestamp: 1000,
            provider: "provider1".to_string(),
            confidence: 9000,
        };
        oracle.submit_price("ETH/USD", price1, 1000).unwrap();

        // Second price within 5% - should succeed
        let price2 = PriceData {
            price: 2090_00000000, // 4.5% increase
            decimals: 8,
            timestamp: 1100,
            provider: "provider1".to_string(),
            confidence: 9000,
        };
        assert!(oracle.submit_price("ETH/USD", price2, 1100).is_ok());

        // Third price exceeds 5% - should fail
        let price3 = PriceData {
            price: 2300_00000000, // 10% increase from last
            decimals: 8,
            timestamp: 1200,
            provider: "provider1".to_string(),
            confidence: 9000,
        };
        assert!(oracle.submit_price("ETH/USD", price3, 1200).is_err());
    }

    #[test]
    fn test_dispute_mechanism() {
        let mut oracle = OracleNetwork::new(1000, 3600, 500);

        oracle
            .register_provider("provider1".to_string(), 2000)
            .unwrap();
        oracle
            .create_feed("BTC/USD".to_string(), "Bitcoin price".to_string(), 1, 1000, 60)
            .unwrap();
        oracle.authorize_provider("provider1", "BTC/USD").unwrap();

        let price_data = PriceData {
            price: 50000_00000000,
            decimals: 8,
            timestamp: 1000,
            provider: "provider1".to_string(),
            confidence: 9000,
        };
        oracle.submit_price("BTC/USD", price_data, 1000).unwrap();

        // Dispute the price
        let dispute_id = oracle
            .dispute_price(
                "BTC/USD",
                "challenger".to_string(),
                "Price too high".to_string(),
                1500,
            )
            .unwrap();

        let feed = oracle.get_feed("BTC/USD").unwrap();
        assert_eq!(feed.status, DataFeedStatus::Disputed);

        // Resolve dispute as invalid price
        oracle
            .resolve_dispute(dispute_id, DisputeResult::InvalidPrice)
            .unwrap();

        let provider = oracle.get_provider("provider1").unwrap();
        assert_eq!(provider.slashed_amount, 100); // 5% of 2000
        assert_eq!(provider.stake, 1900);
        assert_eq!(provider.failed_updates, 1);
    }

    #[test]
    fn test_reputation_updates() {
        let mut oracle = OracleNetwork::new(1000, 3600, 500);
        oracle
            .register_provider("provider1".to_string(), 2000)
            .unwrap();

        let provider = oracle.providers.get_mut("provider1").unwrap();
        provider.successful_updates = 95;
        provider.failed_updates = 5;
        provider.total_disputes = 0;

        OracleNetwork::update_reputation_static(provider);

        // Should have 95% success rate = 9500 reputation
        assert_eq!(provider.reputation_score, 9500);
    }

    #[test]
    fn test_aggregated_price() {
        let mut oracle = OracleNetwork::new(1000, 3600, 500);

        oracle
            .register_provider("provider1".to_string(), 2000)
            .unwrap();
        oracle
            .create_feed("ETH/USD".to_string(), "Ethereum price".to_string(), 1, 1000, 60)
            .unwrap();
        oracle.authorize_provider("provider1", "ETH/USD").unwrap();

        let price_data = PriceData {
            price: 2000_00000000,
            decimals: 8,
            timestamp: 1000,
            provider: "provider1".to_string(),
            confidence: 9000,
        };
        oracle.submit_price("ETH/USD", price_data, 1000).unwrap();

        let aggregated = oracle.get_aggregated_price("ETH/USD").unwrap();
        assert_eq!(aggregated.price, 2000_00000000);
        assert_eq!(aggregated.provider, "aggregated");
    }
}

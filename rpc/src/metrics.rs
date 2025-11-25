use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec, IntCounter, IntCounterVec,
    IntGauge, IntGaugeVec, Opts, Registry,
};
use std::sync::Arc;

lazy_static::lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    // Block metrics
    pub static ref BLOCKS_PRODUCED: IntCounter = IntCounter::new(
        "act_blocks_produced_total",
        "Total number of blocks produced"
    ).expect("metric can be created");

    pub static ref BLOCKS_PRODUCED_BY_VALIDATOR: IntCounterVec = IntCounterVec::new(
        Opts::new("act_blocks_produced_by_validator", "Blocks produced by validator"),
        &["validator"]
    ).expect("metric can be created");

    pub static ref CURRENT_BLOCK_NUMBER: IntGauge = IntGauge::new(
        "act_current_block_number",
        "Current block number"
    ).expect("metric can be created");

    pub static ref BLOCK_PRODUCTION_TIME: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new("act_block_production_time_seconds", "Block production time in seconds")
    ).expect("metric can be created");

    // Transaction metrics
    pub static ref TRANSACTIONS_TOTAL: IntCounter = IntCounter::new(
        "act_transactions_total",
        "Total number of transactions"
    ).expect("metric can be created");

    pub static ref TRANSACTIONS_PENDING: IntGauge = IntGauge::new(
        "act_transactions_pending",
        "Number of pending transactions in mempool"
    ).expect("metric can be created");

    pub static ref TRANSACTIONS_FAILED: IntCounter = IntCounter::new(
        "act_transactions_failed_total",
        "Total number of failed transactions"
    ).expect("metric can be created");

    pub static ref TRANSACTION_PROCESSING_TIME: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new("act_transaction_processing_time_seconds", "Transaction processing time")
    ).expect("metric can be created");

    // Network metrics
    pub static ref PEER_COUNT: IntGauge = IntGauge::new(
        "act_peer_count",
        "Number of connected peers"
    ).expect("metric can be created");

    pub static ref MESSAGES_SENT: IntCounterVec = IntCounterVec::new(
        Opts::new("act_messages_sent_total", "Messages sent by type"),
        &["message_type"]
    ).expect("metric can be created");

    pub static ref MESSAGES_RECEIVED: IntCounterVec = IntCounterVec::new(
        Opts::new("act_messages_received_total", "Messages received by type"),
        &["message_type"]
    ).expect("metric can be created");

    // Staking metrics
    pub static ref TOTAL_STAKED: Gauge = Gauge::new(
        "act_total_staked",
        "Total amount staked (in ACT)"
    ).expect("metric can be created");

    pub static ref VALIDATOR_COUNT: IntGauge = IntGauge::new(
        "act_validator_count",
        "Number of active validators"
    ).expect("metric can be created");

    pub static ref VALIDATOR_STAKE: GaugeVec = GaugeVec::new(
        Opts::new("act_validator_stake", "Stake amount per validator"),
        &["validator"]
    ).expect("metric can be created");

    pub static ref DELEGATIONS_TOTAL: IntCounter = IntCounter::new(
        "act_delegations_total",
        "Total number of delegations"
    ).expect("metric can be created");

    pub static ref REWARDS_DISTRIBUTED: Counter = Counter::new(
        "act_rewards_distributed_total",
        "Total rewards distributed (in ACT)"
    ).expect("metric can be created");

    // Governance metrics
    pub static ref PROPOSALS_TOTAL: IntCounter = IntCounter::new(
        "act_proposals_total",
        "Total number of governance proposals"
    ).expect("metric can be created");

    pub static ref PROPOSALS_ACTIVE: IntGauge = IntGauge::new(
        "act_proposals_active",
        "Number of active proposals"
    ).expect("metric can be created");

    pub static ref VOTES_TOTAL: IntCounter = IntCounter::new(
        "act_votes_total",
        "Total number of votes cast"
    ).expect("metric can be created");

    pub static ref VOTING_PARTICIPATION: Gauge = Gauge::new(
        "act_voting_participation_rate",
        "Voting participation rate (0-1)"
    ).expect("metric can be created");

    // RPC metrics
    pub static ref RPC_REQUESTS: IntCounterVec = IntCounterVec::new(
        Opts::new("act_rpc_requests_total", "Total RPC requests by method"),
        &["method"]
    ).expect("metric can be created");

    pub static ref RPC_REQUEST_DURATION: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new("act_rpc_request_duration_seconds", "RPC request duration"),
        &["method"]
    ).expect("metric can be created");

    pub static ref RPC_ERRORS: IntCounterVec = IntCounterVec::new(
        Opts::new("act_rpc_errors_total", "Total RPC errors"),
        &["method", "error_type"]
    ).expect("metric can be created");

    // State metrics
    pub static ref STATE_SIZE: IntGauge = IntGauge::new(
        "act_state_size_bytes",
        "Size of blockchain state in bytes"
    ).expect("metric can be created");

    pub static ref ACCOUNT_COUNT: IntGauge = IntGauge::new(
        "act_account_count",
        "Number of accounts with non-zero balance"
    ).expect("metric can be created");

    pub static ref CONTRACT_COUNT: IntGauge = IntGauge::new(
        "act_contract_count",
        "Number of deployed contracts"
    ).expect("metric can be created");

    // Node health metrics
    pub static ref NODE_UPTIME: IntGauge = IntGauge::new(
        "act_node_uptime_seconds",
        "Node uptime in seconds"
    ).expect("metric can be created");

    pub static ref NODE_HEALTH: IntGauge = IntGauge::new(
        "act_node_health",
        "Node health status (1=healthy, 0=unhealthy)"
    ).expect("metric can be created");

    pub static ref LAST_BLOCK_TIME: IntGauge = IntGauge::new(
        "act_last_block_time_seconds",
        "Unix timestamp of last block"
    ).expect("metric can be created");

    pub static ref SYNC_STATUS: IntGauge = IntGauge::new(
        "act_sync_status",
        "Sync status (1=synced, 0=syncing)"
    ).expect("metric can be created");
}

/// Initialize all metrics
pub fn init_metrics() {
    REGISTRY.register(Box::new(BLOCKS_PRODUCED.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(BLOCKS_PRODUCED_BY_VALIDATOR.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(CURRENT_BLOCK_NUMBER.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(BLOCK_PRODUCTION_TIME.clone())).expect("collector can be registered");

    REGISTRY.register(Box::new(TRANSACTIONS_TOTAL.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(TRANSACTIONS_PENDING.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(TRANSACTIONS_FAILED.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(TRANSACTION_PROCESSING_TIME.clone())).expect("collector can be registered");

    REGISTRY.register(Box::new(PEER_COUNT.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(MESSAGES_SENT.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(MESSAGES_RECEIVED.clone())).expect("collector can be registered");

    REGISTRY.register(Box::new(TOTAL_STAKED.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(VALIDATOR_COUNT.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(VALIDATOR_STAKE.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(DELEGATIONS_TOTAL.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(REWARDS_DISTRIBUTED.clone())).expect("collector can be registered");

    REGISTRY.register(Box::new(PROPOSALS_TOTAL.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(PROPOSALS_ACTIVE.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(VOTES_TOTAL.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(VOTING_PARTICIPATION.clone())).expect("collector can be registered");

    REGISTRY.register(Box::new(RPC_REQUESTS.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(RPC_REQUEST_DURATION.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(RPC_ERRORS.clone())).expect("collector can be registered");

    REGISTRY.register(Box::new(STATE_SIZE.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(ACCOUNT_COUNT.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(CONTRACT_COUNT.clone())).expect("collector can be registered");

    REGISTRY.register(Box::new(NODE_UPTIME.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(NODE_HEALTH.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(LAST_BLOCK_TIME.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(SYNC_STATUS.clone())).expect("collector can be registered");
}

/// Export metrics in Prometheus format
pub fn export_metrics() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

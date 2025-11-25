use libp2p::{
    gossipsub, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId,
    futures::StreamExt,
};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use tokio::{io, select};

use consensus::{start_consensus, ConsensusEngine};
use governance::GovernanceManager;
use mempool::Mempool;
use rpc::{start_rpc_server, RpcState};
use state::{GenesisAccount, StateManager};
use staking::StakingManager;
use storage::BlockchainStorage;
use types::{Action, Transaction, TransactionType};

#[derive(NetworkBehaviour)]
struct NodeBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 ACT Blockchain Node starting...");

    // Initialize storage
    let storage = Arc::new(BlockchainStorage::new("./actchain_data")?);
    println!("💾 Storage initialized");

    // Initialize state manager with genesis accounts
    let state_manager = Arc::new(StateManager::new(storage.clone()));
    
    // Create genesis accounts with initial ACT allocation
    let genesis_accounts = vec![
        GenesisAccount::new("ACT-validator1".to_string(), 1_000_000.0), // 1M ACT
        GenesisAccount::new("ACT-validator2".to_string(), 1_000_000.0),
        GenesisAccount::new("ACT-validator3".to_string(), 1_000_000.0),
        GenesisAccount::new("ACT-treasury".to_string(), 10_000_000.0),  // 10M ACT
    ];
    
    state_manager.initialize_genesis(genesis_accounts)?;
    println!("🌱 Genesis state initialized");

    // Initialize mempool
    let mempool = Arc::new(Mempool::new(10_000)); // Max 10k pending txs
    println!("🔄 Mempool initialized");

    // Initialize staking manager
    let staking_manager = Arc::new(tokio::sync::Mutex::new(StakingManager::new()));
    println!("💎 Staking manager initialized");

    // Initialize governance manager
    let governance_manager = Arc::new(tokio::sync::Mutex::new(GovernanceManager::new()));
    println!("🏛️  Governance manager initialized");

    // Initialize consensus engine
    let consensus_engine = Arc::new(ConsensusEngine::new());
    println!("🎯 Consensus engine initialized");

    // Start consensus in background
    let consensus_handle = consensus_engine.clone();
    tokio::spawn(async move {
        start_consensus(consensus_handle).await;
    });

    // Start RPC server in background
    let rpc_state = RpcState::new(
        state_manager.clone(),
        mempool.clone(),
        staking_manager.clone(),
        governance_manager.clone(),
    );
    tokio::spawn(async move {
        if let Err(e) = start_rpc_server(rpc_state, 8545).await {
            eprintln!("❌ RPC server error: {}", e);
        }
    });

    // Create a random PeerId
    let local_key = libp2p::identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("📍 Local peer id: {local_peer_id}");

    // Set up gossipsub
    let message_id_fn = |message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        gossipsub::MessageId::from(s.finish().to_string())
    };

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .message_id_fn(message_id_fn)
        .build()
        .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?;

    let mut gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(local_key.clone()),
        gossipsub_config,
    )?;

    // Subscribe to topics
    let blocks_topic = gossipsub::IdentTopic::new("act-blocks");
    let tx_topic = gossipsub::IdentTopic::new("act-transactions");
    gossipsub.subscribe(&blocks_topic)?;
    gossipsub.subscribe(&tx_topic)?;
    println!("📡 Subscribed to act-blocks and act-transactions topics");

    // Set up mDNS for local peer discovery
    let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;

    // Create the swarm
    let behaviour = NodeBehaviour { gossipsub, mdns };
    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| behaviour)?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    // Listen on all interfaces
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("🌐 P2P network initialized. Listening for connections...");

    // Transaction handler - process incoming transactions
    let mempool_for_handler = mempool.clone();
    let state_for_handler = state_manager.clone();
    let (tx_sender, mut tx_receiver) = tokio::sync::mpsc::channel::<Transaction>(1000);
    
    tokio::spawn(async move {
        while let Some(tx) = tx_receiver.recv().await {
            match mempool_for_handler.add_transaction(tx.clone(), &state_for_handler) {
                Ok(hash) => {
                    println!("📥 Transaction added to mempool: {}...", &hash[..16]);
                    let stats = mempool_for_handler.get_stats();
                    println!("   Mempool: {} txs from {} senders, avg gas: {}",
                        stats.total_transactions,
                        stats.unique_senders,
                        stats.avg_gas_price
                    );
                }
                Err(e) => {
                    eprintln!("❌ Invalid transaction: {}", e);
                }
            }
        }
    });

    // Block proposer - create blocks with transactions from mempool
    let engine_for_blocks = consensus_engine.clone();
    let mempool_for_blocks = mempool.clone();
    let state_for_blocks = state_manager.clone();
    let staking_for_blocks = staking_manager.clone();
    let governance_for_blocks = governance_manager.clone();
    
    tokio::spawn(async move {
        let mut block_num = 0;
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            block_num += 1;
            
            // Update block height in staking and governance
            {
                let mut staking = staking_for_blocks.lock().await;
                staking.set_block_height(block_num);
            }
            {
                let mut governance = governance_for_blocks.lock().await;
                governance.set_block_height(block_num);
            }
            
            // Get transactions from mempool
            let txs = mempool_for_blocks.get_transactions_for_block(100, &state_for_blocks);
            
            let mut total_tx_fees = 0u64;
            
            if !txs.is_empty() {
                println!("\n🔨 Creating block {} with {} transactions", block_num, txs.len());
                
                // Process transactions and update state
                for tx in &txs {
                    let tx_hash = tx.hash();
                    println!("  ⚡ Including tx {}... from {}", &tx_hash[..16], tx.from.to_string());
                    
                    // Calculate transaction fee
                    let tx_fee = (tx.gas_limit as u64) * (tx.gas_price as u64);
                    total_tx_fees += tx_fee;
                    
                    // Execute transaction
                    match &tx.tx_type {
                        TransactionType::Transfer { to, amount } => {
                            if let Err(e) = state_for_blocks.transfer(&tx.from.to_string(), to, *amount) {
                                eprintln!("     ⚠️  Transfer failed: {}", e);
                                continue;
                            }
                        }
                        _ => {}
                    }
                    
                    // Remove from mempool
                    mempool_for_blocks.remove_transaction(&tx_hash);
                }
                
                println!("📊 Block {} processed {} transactions, fees: {} ACT", 
                    block_num, txs.len(), total_tx_fees / 1_000_000_000);
            }
            
            // Distribute block rewards to validator (simplified - using first validator)
            let validator_address = "ACT-validator1";
            {
                let mut staking = staking_for_blocks.lock().await;
                staking.distribute_block_reward(validator_address, total_tx_fees);
                println!("💰 Block rewards distributed to {}", validator_address);
            }
            
            // Update active governance proposals
            {
                let mut governance = governance_for_blocks.lock().await;
                let active_proposals: Vec<u64> = governance
                    .list_proposals(None)
                    .iter()
                    .filter(|p| matches!(p.status, governance::ProposalStatus::Review | governance::ProposalStatus::Active))
                    .map(|p| p.id)
                    .collect();
                
                for proposal_id in active_proposals {
                    if let Err(e) = governance.update_proposal_status(proposal_id) {
                        eprintln!("⚠️  Failed to update proposal {}: {}", proposal_id, e);
                    }
                }
            }
            
            // Still propose block (even if empty) for consensus
            let action = Action {
                actor: format!("block_proposer_{}", block_num),
                payload: format!("block_{}_data", block_num).into_bytes(),
                nonce: block_num,
            };
            
            match engine_for_blocks.propose_block(action).await {
                Ok(header) => {
                    println!("📦 Block {} finalized at height {}", block_num, header.height);
                }
                Err(e) => {
                    eprintln!("❌ Failed to propose block: {}", e);
                }
            }
        }
    });

    // Event loop
    loop {
        select! {
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("🎧 Listening on {address}");
                }
                SwarmEvent::Behaviour(NodeBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("🔍 mDNS discovered a new peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                }
                SwarmEvent::Behaviour(NodeBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("❌ mDNS peer expired: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                }
                SwarmEvent::Behaviour(NodeBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                })) => {
                    // Check which topic the message is from
                    if message.topic.to_string().contains("transactions") {
                        // Deserialize transaction
                        if let Ok(tx) = serde_json::from_slice::<Transaction>(&message.data) {
                            println!("📨 Received transaction from peer: {}", peer_id);
                            let _ = tx_sender.send(tx).await;
                        }
                    } else if message.topic.to_string().contains("blocks") {
                        println!("📨 Received block from peer: {}", peer_id);
                    }
                }
                _ => {}
            }
        }
    }
}

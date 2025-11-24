use libp2p::{
    core::upgrade,
    gossipsub, identify, kad,
    mdns, noise,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp, yamux, PeerId, Transport,
};
use std::error::Error;
use std::time::Duration;
use tokio::io::{self, AsyncBufReadExt};

#[derive(NetworkBehaviour)]
pub struct BlockchainBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub identify: identify::Behaviour,
}

pub async fn start_p2p_network(port: u16) -> Result<(), Box<dyn Error>> {
    // Create a random PeerId for this node
    let local_key = libp2p::identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Set up an encrypted TCP transport with noise and yamux multiplexing
    let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::Config::new(&local_key).unwrap())
        .multiplex(yamux::Config::default())
        .boxed();

    // Configure Gossipsub for block and transaction propagation
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(1))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .build()
        .expect("Valid config");
    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(local_key.clone()),
        gossipsub_config,
    )
    .expect("Correct configuration");

    // Set up mDNS for local peer discovery
    let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;

    // Set up Kademlia DHT for distributed peer discovery
    let mut kad_config = kad::Config::default();
    kad_config.set_query_timeout(Duration::from_secs(60));
    let kademlia = kad::Behaviour::new(
        local_peer_id,
        kad::store::MemoryStore::new(local_peer_id),
    );

    // Set up Identify protocol for peer info exchange
    let identify = identify::Behaviour::new(identify::Config::new(
        "/proof-of-action/1.0.0".to_string(),
        local_key.public(),
    ));

    // Create the network behaviour
    let behaviour = BlockchainBehaviour {
        gossipsub,
        mdns,
        kademlia,
        identify,
    };

    // Build the swarm
    let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id).build();

    // Listen on the specified port
    let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", port);
    swarm.listen_on(listen_addr.parse()?)?;
    println!("Listening on port {}", port);

    // Subscribe to blockchain topics
    let block_topic = gossipsub::IdentTopic::new("blocks");
    let action_topic = gossipsub::IdentTopic::new("actions");
    swarm.behaviour_mut().gossipsub.subscribe(&block_topic)?;
    swarm.behaviour_mut().gossipsub.subscribe(&action_topic)?;
    println!("Subscribed to topics: blocks, actions");

    // Event loop
    loop {
        tokio::select! {
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(BlockchainBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, multiaddr) in list {
                            println!("Discovered peer: {} at {}", peer_id, multiaddr);
                            swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                            swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
                        }
                    }
                    SwarmEvent::Behaviour(BlockchainBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                        for (peer_id, _) in list {
                            println!("Peer expired: {}", peer_id);
                            swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        }
                    }
                    SwarmEvent::Behaviour(BlockchainBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        propagation_source,
                        message_id,
                        message,
                    })) => {
                        println!(
                            "Got message from {:?} (id: {:?}): {:?}",
                            propagation_source,
                            message_id,
                            String::from_utf8_lossy(&message.data)
                        );
                    }
                    SwarmEvent::Behaviour(BlockchainBehaviourEvent::Identify(identify::Event::Received {
                        peer_id,
                        info,
                    })) => {
                        println!("Identified peer: {} with protocol version: {}", peer_id, info.protocol_version);
                    }
                    _ => {}
                }
            }
        }
    }
}

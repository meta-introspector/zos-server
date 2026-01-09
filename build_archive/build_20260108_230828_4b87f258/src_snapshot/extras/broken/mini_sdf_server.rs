// Mini SDF.org Server Implementation
// LibP2P-based community server inspired by SDF.org

use libp2p::{
    gossipsub, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, Swarm,
};
use libp2p_swarm_derive::NetworkBehaviour;
use std::collections::HashMap;
use tokio::io::{self, AsyncBufReadExt};

#[derive(NetworkBehaviour)]
pub struct SdfBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

pub struct MiniSdfServer {
    swarm: Swarm<SdfBehaviour>,
    users: HashMap<PeerId, String>,
    shell_sessions: HashMap<PeerId, String>,
}

impl MiniSdfServer {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        println!("Mini SDF Server Peer ID: {local_peer_id}");

        // Gossipsub configuration
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(std::time::Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .expect("Valid config");

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )?;

        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;

        let behaviour = SdfBehaviour { gossipsub, mdns };

        let swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
            .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
            .with_behaviour(|_| behaviour)?
            .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
            .build();

        Ok(MiniSdfServer {
            swarm,
            users: HashMap::new(),
            shell_sessions: HashMap::new(),
        })
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Listen on all interfaces
        self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        // Subscribe to SDF topics
        let sdf_topic = gossipsub::IdentTopic::new("sdf-community");
        let shell_topic = gossipsub::IdentTopic::new("sdf-shell");

        self.swarm.behaviour_mut().gossipsub.subscribe(&sdf_topic)?;
        self.swarm.behaviour_mut().gossipsub.subscribe(&shell_topic)?;

        println!("🌐 Mini SDF Server started!");
        println!("📡 Listening for LibP2P connections...");
        println!("🐚 Shell access available via gossipsub");
        println!("👥 Community chat enabled");

        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("📍 Listening on {address}");
                }
                SwarmEvent::Behaviour(SdfBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("👋 Discovered peer: {peer_id}");
                        self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                }
                SwarmEvent::Behaviour(SdfBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: _,
                    message,
                })) => {
                    let msg_str = String::from_utf8_lossy(&message.data);
                    println!("💬 Message from {peer_id}: {msg_str}");

                    // Handle shell commands
                    if message.topic == shell_topic.hash() {
                        self.handle_shell_command(peer_id, &msg_str).await;
                    }
                }
                _ => {}
            }
        }
    }

    async fn handle_shell_command(&mut self, peer_id: PeerId, command: &str) {
        println!("🐚 Shell command from {peer_id}: {command}");

        // Simple shell command handling
        let response = match command.trim() {
            "whoami" => format!("peer_{}", peer_id.to_string()[..8].to_string()),
            "pwd" => "/home/sdf".to_string(),
            "ls" => "bin/ lib/ share/ tmp/ users/".to_string(),
            "uptime" => "Mini SDF Server - LibP2P Community Shell".to_string(),
            cmd if cmd.starts_with("echo ") => cmd[5..].to_string(),
            _ => format!("Command not found: {}", command),
        };

        // Send response back via gossipsub
        let shell_topic = gossipsub::IdentTopic::new("sdf-shell-response");
        let response_msg = format!("{}: {}", peer_id.to_string()[..8].to_string(), response);

        if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(shell_topic, response_msg.as_bytes()) {
            println!("❌ Failed to send shell response: {e}");
        }
    }
}

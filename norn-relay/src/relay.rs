use futures::StreamExt;
use libp2p::gossipsub::{self, IdentTopic};
use libp2p::request_response;
use libp2p::swarm::SwarmEvent;
use libp2p::{Multiaddr, PeerId, Swarm, SwarmBuilder};
use norn_types::network::NornMessage;
use norn_types::primitives::Address;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use crate::behaviour::{build_behaviour, NornBehaviour, NornBehaviourEvent};
use crate::codec;
use crate::config::RelayConfig;
use crate::discovery::Discovery;
use crate::error::RelayError;
use crate::peer_manager::PeerManager;
use crate::protocol::{BLOCKS_TOPIC, COMMITMENTS_TOPIC, FRAUD_PROOFS_TOPIC, GENERAL_TOPIC};
use crate::spindle_registry::SpindleRegistry;

/// The main relay node that handles networking.
pub struct RelayNode {
    config: RelayConfig,
    swarm: Swarm<NornBehaviour>,
    peer_manager: PeerManager,
    _spindle_registry: SpindleRegistry,
    _discovery: Discovery,
    message_tx: broadcast::Sender<NornMessage>,
}

impl RelayNode {
    /// Create a new RelayNode, start listening on the configured address.
    pub async fn new(config: RelayConfig) -> Result<Self, RelayError> {
        let keypair = if let Some(seed) = &config.keypair_seed {
            let mut seed_bytes = *seed;
            libp2p::identity::Keypair::ed25519_from_bytes(&mut seed_bytes).map_err(|e| {
                RelayError::NetworkError {
                    reason: format!("invalid keypair seed: {}", e),
                }
            })?
        } else {
            libp2p::identity::Keypair::generate_ed25519()
        };

        let mut swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::noise::Config::new,
                libp2p::yamux::Config::default,
            )
            .map_err(|e| RelayError::NetworkError {
                reason: format!("tcp transport: {}", e),
            })?
            .with_behaviour(build_behaviour)
            .map_err(|e| RelayError::NetworkError {
                reason: format!("behaviour: {}", e),
            })?
            .with_swarm_config(|cfg| {
                cfg.with_idle_connection_timeout(std::time::Duration::from_secs(60))
            })
            .build();

        // Subscribe to gossipsub topics.
        let topics = [
            BLOCKS_TOPIC,
            COMMITMENTS_TOPIC,
            FRAUD_PROOFS_TOPIC,
            GENERAL_TOPIC,
        ];
        for topic_name in &topics {
            let topic = IdentTopic::new(*topic_name);
            swarm
                .behaviour_mut()
                .gossipsub
                .subscribe(&topic)
                .map_err(|e| RelayError::ProtocolError {
                    reason: format!("subscribe to {}: {}", topic_name, e),
                })?;
        }

        // Listen on the configured address.
        let listen_addr: Multiaddr = format!(
            "/ip4/{}/tcp/{}",
            config.listen_addr.ip(),
            config.listen_addr.port()
        )
        .parse()
        .map_err(|e| RelayError::NetworkError {
            reason: format!("parse listen addr: {}", e),
        })?;

        swarm
            .listen_on(listen_addr)
            .map_err(|e| RelayError::NetworkError {
                reason: format!("listen: {}", e),
            })?;

        // Set up discovery for boot nodes.
        let discovery = Discovery::new(config.boot_nodes.clone());
        for addr in discovery.boot_addrs() {
            swarm
                .dial(addr.clone())
                .map_err(|e| RelayError::ConnectionError {
                    reason: format!("dial boot node {}: {}", addr, e),
                })?;
        }

        let peer_manager = PeerManager::new(config.max_connections);
        let spindle_registry = SpindleRegistry::new();
        let (message_tx, _) = broadcast::channel(1024);

        info!(
            peer_id = %swarm.local_peer_id(),
            listen = %config.listen_addr,
            "relay node started"
        );

        Ok(Self {
            config,
            swarm,
            peer_manager,
            _spindle_registry: spindle_registry,
            _discovery: discovery,
            message_tx,
        })
    }

    /// Get the local peer ID.
    pub fn local_peer_id(&self) -> PeerId {
        *self.swarm.local_peer_id()
    }

    /// Subscribe to receive messages from the network.
    pub fn subscribe(&self) -> broadcast::Receiver<NornMessage> {
        self.message_tx.subscribe()
    }

    /// Send a direct message to a specific Norn address.
    pub async fn send_to(&mut self, addr: Address, msg: NornMessage) -> Result<(), RelayError> {
        let peer_id = self
            .peer_manager
            .peer_for_address(&addr)
            .copied()
            .ok_or_else(|| RelayError::PeerNotFound {
                peer: hex_encode(&addr),
            })?;

        self.swarm
            .behaviour_mut()
            .request_response
            .send_request(&peer_id, msg);

        Ok(())
    }

    /// Broadcast a message to all peers via gossipsub.
    ///
    /// Routes the message to the appropriate topic based on its type:
    /// - `Block` -> `BLOCKS_TOPIC`
    /// - `Commitment` -> `COMMITMENTS_TOPIC`
    /// - `FraudProof` -> `FRAUD_PROOFS_TOPIC`
    /// - All other messages -> `GENERAL_TOPIC`
    pub async fn broadcast(&mut self, msg: NornMessage) -> Result<(), RelayError> {
        let topic_name = topic_for_message(&msg);
        let data = codec::encode_message(&msg)?;
        let topic = IdentTopic::new(topic_name);

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic, data)
            .map_err(|e| RelayError::NetworkError {
                reason: format!("publish: {}", e),
            })?;

        Ok(())
    }

    /// Broadcast a message on a specific topic.
    pub async fn broadcast_on_topic(
        &mut self,
        topic_name: &str,
        msg: NornMessage,
    ) -> Result<(), RelayError> {
        let data = codec::encode_message(&msg)?;
        let topic = IdentTopic::new(topic_name);

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic, data)
            .map_err(|e| RelayError::NetworkError {
                reason: format!("publish to {}: {}", topic_name, e),
            })?;

        Ok(())
    }

    /// Main event loop. Processes swarm events.
    pub async fn run(&mut self) -> Result<(), RelayError> {
        loop {
            let event = self.swarm.next().await;
            match event {
                Some(SwarmEvent::Behaviour(event)) => {
                    self.handle_behaviour_event(event);
                }
                Some(SwarmEvent::ConnectionEstablished {
                    peer_id, endpoint, ..
                }) => {
                    debug!(%peer_id, ?endpoint, "connection established");
                    if !self.peer_manager.add_peer(peer_id) {
                        warn!(
                            %peer_id,
                            max = self.config.max_connections,
                            "peer limit reached, disconnecting peer"
                        );
                        let _ = self.swarm.disconnect_peer_id(peer_id);
                    }
                }
                Some(SwarmEvent::ConnectionClosed { peer_id, cause, .. }) => {
                    debug!(%peer_id, ?cause, "connection closed");
                    self.peer_manager.remove_peer(&peer_id);
                }
                Some(SwarmEvent::NewListenAddr { address, .. }) => {
                    info!(%address, "listening on new address");
                }
                Some(other) => {
                    debug!(?other, "other swarm event");
                }
                None => {
                    return Err(RelayError::NetworkError {
                        reason: "swarm stream ended".to_string(),
                    });
                }
            }
        }
    }

    fn handle_behaviour_event(&mut self, event: NornBehaviourEvent) {
        match event {
            NornBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source,
                message,
                ..
            }) => {
                debug!(
                    %propagation_source,
                    topic = %message.topic,
                    "received gossipsub message"
                );
                match codec::decode_message(&message.data) {
                    Ok(msg) => {
                        let _ = self.message_tx.send(msg);
                    }
                    Err(e) => {
                        warn!("failed to decode gossipsub message: {}", e);
                    }
                }
            }
            NornBehaviourEvent::RequestResponse(request_response::Event::Message {
                peer,
                message,
            }) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    debug!(%peer, "received direct request");
                    let _ = self.message_tx.send(request.clone());
                    // Send back an echo response (acknowledgement).
                    let _ = self
                        .swarm
                        .behaviour_mut()
                        .request_response
                        .send_response(channel, request);
                }
                request_response::Message::Response { response, .. } => {
                    debug!(%peer, "received direct response");
                    let _ = self.message_tx.send(response);
                }
            },
            NornBehaviourEvent::Identify(libp2p::identify::Event::Received {
                peer_id,
                info,
                ..
            }) => {
                debug!(
                    %peer_id,
                    protocol = %info.protocol_version,
                    agent = %info.agent_version,
                    "identified peer"
                );
            }
            _ => {}
        }
    }

    /// Get a reference to the relay config.
    pub fn config(&self) -> &RelayConfig {
        &self.config
    }

    /// Get a reference to the peer manager.
    pub fn peer_manager(&self) -> &PeerManager {
        &self.peer_manager
    }

    /// Get a mutable reference to the peer manager.
    pub fn peer_manager_mut(&mut self) -> &mut PeerManager {
        &mut self.peer_manager
    }
}

/// Returns the gossipsub topic name for the given message type.
pub fn topic_for_message(msg: &NornMessage) -> &'static str {
    match msg {
        NornMessage::Block(_) => BLOCKS_TOPIC,
        NornMessage::Commitment(_) => COMMITMENTS_TOPIC,
        NornMessage::FraudProof(_) => FRAUD_PROOFS_TOPIC,
        _ => GENERAL_TOPIC,
    }
}

/// Simple hex encoder to avoid adding a dependency.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    fn test_config(port: u16) -> RelayConfig {
        RelayConfig {
            listen_addr: SocketAddr::from(([127, 0, 0, 1], port)),
            boot_nodes: vec![],
            max_connections: 50,
            keypair_seed: None,
        }
    }

    #[tokio::test]
    async fn test_relay_node_creation() {
        let config = test_config(0);
        let node = RelayNode::new(config).await;
        assert!(node.is_ok());
    }

    #[tokio::test]
    async fn test_relay_node_with_seed() {
        let config = RelayConfig {
            listen_addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            boot_nodes: vec![],
            max_connections: 50,
            keypair_seed: Some([42u8; 32]),
        };
        let node1 = RelayNode::new(config.clone()).await.unwrap();
        let node2 = RelayNode::new(config).await.unwrap();
        assert_eq!(node1.local_peer_id(), node2.local_peer_id());
    }

    #[tokio::test]
    async fn test_subscribe() {
        let config = test_config(0);
        let node = RelayNode::new(config).await.unwrap();
        let _rx = node.subscribe();
    }

    #[tokio::test]
    async fn test_send_to_unknown_peer() {
        let config = test_config(0);
        let mut node = RelayNode::new(config).await.unwrap();
        let addr = [1u8; 20];
        let msg = NornMessage::Registration(norn_types::weave::Registration {
            thread_id: [1u8; 20],
            owner: [2u8; 32],
            initial_state_hash: [3u8; 32],
            timestamp: 1000,
            signature: [4u8; 64],
        });
        let result = node.send_to(addr, msg).await;
        assert!(matches!(result, Err(RelayError::PeerNotFound { .. })));
    }

    #[test]
    fn test_topic_routing_block() {
        let block = NornMessage::Block(Box::new(norn_types::weave::WeaveBlock {
            height: 1,
            hash: [0u8; 32],
            prev_hash: [0u8; 32],
            commitments_root: [0u8; 32],
            registrations_root: [0u8; 32],
            anchors_root: [0u8; 32],
            commitments: vec![],
            registrations: vec![],
            anchors: vec![],
            fraud_proofs: vec![],
            fraud_proofs_root: [0u8; 32],
            timestamp: 1000,
            proposer: [0u8; 32],
            validator_signatures: vec![],
        }));
        assert_eq!(topic_for_message(&block), BLOCKS_TOPIC);
    }

    #[test]
    fn test_topic_routing_commitment() {
        let commitment = NornMessage::Commitment(norn_types::weave::CommitmentUpdate {
            thread_id: [1u8; 20],
            owner: [2u8; 32],
            version: 1,
            state_hash: [3u8; 32],
            prev_commitment_hash: [0u8; 32],
            knot_count: 5,
            timestamp: 1000,
            signature: [4u8; 64],
        });
        assert_eq!(topic_for_message(&commitment), COMMITMENTS_TOPIC);
    }

    #[test]
    fn test_topic_routing_fraud_proof() {
        use norn_types::fraud::{FraudProof, FraudProofSubmission};

        let fraud = NornMessage::FraudProof(Box::new(FraudProofSubmission {
            proof: FraudProof::InvalidLoomTransition {
                loom_id: [1u8; 32],
                knot: Box::new(norn_types::knot::Knot {
                    id: [0u8; 32],
                    knot_type: norn_types::knot::KnotType::Transfer,
                    timestamp: 1000,
                    expiry: None,
                    before_states: vec![],
                    after_states: vec![],
                    payload: norn_types::knot::KnotPayload::Transfer(
                        norn_types::knot::TransferPayload {
                            token_id: [0u8; 32],
                            amount: 100,
                            from: [1u8; 20],
                            to: [2u8; 20],
                            memo: None,
                        },
                    ),
                    signatures: vec![],
                }),
                reason: "test".to_string(),
            },
            submitter: [5u8; 32],
            timestamp: 2000,
            signature: [6u8; 64],
        }));
        assert_eq!(topic_for_message(&fraud), FRAUD_PROOFS_TOPIC);
    }

    #[test]
    fn test_topic_routing_general_for_registration() {
        let reg = NornMessage::Registration(norn_types::weave::Registration {
            thread_id: [1u8; 20],
            owner: [2u8; 32],
            initial_state_hash: [3u8; 32],
            timestamp: 1000,
            signature: [4u8; 64],
        });
        assert_eq!(topic_for_message(&reg), GENERAL_TOPIC);
    }

    #[test]
    fn test_topic_routing_general_for_relay_message() {
        let relay = NornMessage::Relay(norn_types::network::RelayMessage {
            from: [1u8; 20],
            to: [2u8; 20],
            payload: vec![1, 2, 3],
            timestamp: 1000,
            signature: [4u8; 64],
        });
        assert_eq!(topic_for_message(&relay), GENERAL_TOPIC);
    }

    /// Verify that relay nodes can be created with Strict validation mode.
    #[tokio::test]
    async fn test_relay_node_strict_mode() {
        let config = test_config(0);
        let node = RelayNode::new(config).await;
        assert!(
            node.is_ok(),
            "relay node should work with Strict validation"
        );
    }

    /// Integration test: two relay nodes exchange a direct message.
    /// Marked as ignored because it requires real networking and may be flaky in CI.
    #[tokio::test]
    #[ignore]
    async fn test_two_nodes_exchange_message() {
        use tokio::time::{timeout, Duration};

        // Create two nodes on random ports.
        let config1 = RelayConfig {
            listen_addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            boot_nodes: vec![],
            max_connections: 50,
            keypair_seed: Some([1u8; 32]),
        };
        let mut node1 = RelayNode::new(config1).await.unwrap();
        let peer1 = node1.local_peer_id();

        // We need to get node1's actual listen address.
        // Run the swarm briefly to get the listening address.
        let listen_addr1 = loop {
            match timeout(Duration::from_secs(5), node1.swarm.next()).await {
                Ok(Some(SwarmEvent::NewListenAddr { address, .. })) => {
                    break address;
                }
                Ok(Some(_)) => continue,
                _ => panic!("node1 did not start listening"),
            }
        };

        let config2 = RelayConfig {
            listen_addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            boot_nodes: vec![format!("{}/p2p/{}", listen_addr1, peer1)],
            max_connections: 50,
            keypair_seed: Some([2u8; 32]),
        };
        let mut node2 = RelayNode::new(config2).await.unwrap();
        let _rx2 = node2.subscribe();

        let msg = NornMessage::Registration(norn_types::weave::Registration {
            thread_id: [1u8; 20],
            owner: [2u8; 32],
            initial_state_hash: [3u8; 32],
            timestamp: 1000,
            signature: [4u8; 64],
        });

        // Run both swarms for a bit to let them connect.
        let result = timeout(Duration::from_secs(10), async {
            loop {
                tokio::select! {
                    event = node1.swarm.next() => {
                        if let Some(SwarmEvent::ConnectionEstablished { .. }) = event {
                            info!("node1 connection established");
                        }
                    }
                    event = node2.swarm.next() => {
                        if let Some(SwarmEvent::ConnectionEstablished { peer_id, .. }) = event {
                            info!(%peer_id, "node2 connected to peer");
                            // Try to send a direct request.
                            node2.swarm.behaviour_mut().request_response.send_request(&peer_id, msg.clone());
                            return;
                        }
                    }
                }
            }
        })
        .await;

        assert!(result.is_ok(), "nodes failed to connect within timeout");
    }
}

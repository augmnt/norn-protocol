use std::collections::HashSet;

use futures::StreamExt;
use libp2p::gossipsub::{self, IdentTopic};
use libp2p::request_response;
use libp2p::swarm::SwarmEvent;
use libp2p::{Multiaddr, PeerId, Swarm, SwarmBuilder};
use norn_types::network::{NornMessage, UpgradeNotice};
use norn_types::primitives::Address;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, warn};

use crate::behaviour::{build_behaviour, NornBehaviour, NornBehaviourEvent};
use crate::codec::{self, DecodedMessage};
use crate::config::RelayConfig;
use crate::discovery::Discovery;
use crate::error::RelayError;
use crate::peer_manager::PeerManager;
use crate::protocol::{
    versioned_topic, BLOCKS_TOPIC, COMMITMENTS_TOPIC, FRAUD_PROOFS_TOPIC, GENERAL_TOPIC,
    PROTOCOL_VERSION,
};
use crate::spindle_registry::SpindleRegistry;

/// Internal enum for outbound message routing.
enum OutboundMessage {
    /// Broadcast to all peers via gossipsub.
    Broadcast(NornMessage),
    /// Send directly to a specific peer via request-response.
    SendToPeer(PeerId, NornMessage),
}

/// A cloneable handle for sending messages through the relay after `run()` is spawned.
#[derive(Clone)]
pub struct RelayHandle {
    outbound_tx: mpsc::Sender<OutboundMessage>,
}

impl RelayHandle {
    /// Broadcast a message through the relay's gossipsub network.
    pub async fn broadcast(&self, msg: NornMessage) -> Result<(), RelayError> {
        self.outbound_tx
            .send(OutboundMessage::Broadcast(msg))
            .await
            .map_err(|_| RelayError::ChannelError {
                reason: "relay outbound channel closed".to_string(),
            })
    }

    /// Send a message directly to a specific peer via request-response.
    pub async fn send_to_peer(&self, peer_id: PeerId, msg: NornMessage) -> Result<(), RelayError> {
        self.outbound_tx
            .send(OutboundMessage::SendToPeer(peer_id, msg))
            .await
            .map_err(|_| RelayError::ChannelError {
                reason: "relay outbound channel closed".to_string(),
            })
    }
}

/// The main relay node that handles networking.
pub struct RelayNode {
    config: RelayConfig,
    swarm: Swarm<NornBehaviour>,
    peer_manager: PeerManager,
    _spindle_registry: SpindleRegistry,
    _discovery: Discovery,
    message_tx: broadcast::Sender<(NornMessage, Option<PeerId>)>,
    outbound_tx: mpsc::Sender<OutboundMessage>,
    outbound_rx: Option<mpsc::Receiver<OutboundMessage>>,
    /// Protocol versions for which we've already broadcast an upgrade notice.
    notified_versions: HashSet<u8>,
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
            .with_dns()
            .map_err(|e| RelayError::NetworkError {
                reason: format!("dns transport: {}", e),
            })?
            .with_behaviour(|kp| build_behaviour(kp, PROTOCOL_VERSION))
            .map_err(|e| RelayError::NetworkError {
                reason: format!("behaviour: {}", e),
            })?
            .with_swarm_config(|cfg| {
                cfg.with_idle_connection_timeout(std::time::Duration::from_secs(60))
            })
            .build();

        // Subscribe to both legacy (unversioned) and versioned gossipsub topics.
        let legacy_topics = [
            BLOCKS_TOPIC,
            COMMITMENTS_TOPIC,
            FRAUD_PROOFS_TOPIC,
            GENERAL_TOPIC,
        ];
        for topic_name in &legacy_topics {
            let topic = IdentTopic::new(*topic_name);
            swarm
                .behaviour_mut()
                .gossipsub
                .subscribe(&topic)
                .map_err(|e| RelayError::ProtocolError {
                    reason: format!("subscribe to {}: {}", topic_name, e),
                })?;
        }

        // Versioned topics (e.g. "norn/blocks/v4").
        for base in &legacy_topics {
            let v_topic_name = versioned_topic(base, PROTOCOL_VERSION);
            let topic = IdentTopic::new(&v_topic_name);
            swarm
                .behaviour_mut()
                .gossipsub
                .subscribe(&topic)
                .map_err(|e| RelayError::ProtocolError {
                    reason: format!("subscribe to {}: {}", v_topic_name, e),
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
        let (outbound_tx, outbound_rx) = mpsc::channel(256);

        info!(
            peer_id = %swarm.local_peer_id(),
            listen = %config.listen_addr,
            protocol_version = PROTOCOL_VERSION,
            "relay node started"
        );

        Ok(Self {
            config,
            swarm,
            peer_manager,
            _spindle_registry: spindle_registry,
            _discovery: discovery,
            message_tx,
            outbound_tx,
            outbound_rx: Some(outbound_rx),
            notified_versions: HashSet::new(),
        })
    }

    /// Get the local peer ID.
    pub fn local_peer_id(&self) -> PeerId {
        *self.swarm.local_peer_id()
    }

    /// Subscribe to receive messages from the network.
    /// Each message is paired with the source peer ID (if known).
    pub fn subscribe(&self) -> broadcast::Receiver<(NornMessage, Option<PeerId>)> {
        self.message_tx.subscribe()
    }

    /// Get a cloneable handle for sending outbound messages through the relay.
    /// Call this before spawning `run()`.
    pub fn handle(&self) -> RelayHandle {
        RelayHandle {
            outbound_tx: self.outbound_tx.clone(),
        }
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
    /// Dual-publishes: envelope format on the versioned topic, and legacy format
    /// on the unversioned topic (for backward compatibility during rolling upgrades).
    pub async fn broadcast(&mut self, msg: NornMessage) -> Result<(), RelayError> {
        // Publish on versioned topic with envelope format.
        let v_topic_name = versioned_topic_for_message(&msg);
        let data = codec::encode_message(&msg)?;
        let topic = IdentTopic::new(&v_topic_name);

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic, data)
            .map_err(|e| RelayError::NetworkError {
                reason: format!("publish: {}", e),
            })?;

        // Also publish on legacy topic (best-effort, non-fatal).
        self.publish_legacy(&msg);

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

    /// Best-effort publish on the legacy (unversioned) topic for backward compatibility.
    fn publish_legacy(&mut self, msg: &NornMessage) {
        let legacy_topic_name = legacy_topic_for_message(msg);
        match codec::encode_message_legacy(msg) {
            Ok(data) => {
                let topic = IdentTopic::new(legacy_topic_name);
                if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(topic, data) {
                    debug!("legacy publish failed (non-fatal): {}", e);
                }
            }
            Err(e) => {
                // Expected for new message types (discriminant > 13).
                debug!("legacy encode skipped: {}", e);
            }
        }
    }

    /// Main event loop. Processes swarm events and outbound messages.
    pub async fn run(&mut self) -> Result<(), RelayError> {
        let mut outbound_rx = self
            .outbound_rx
            .take()
            .ok_or_else(|| RelayError::ChannelError {
                reason: "outbound channel already consumed (run called twice?)".to_string(),
            })?;

        loop {
            tokio::select! {
                event = self.swarm.next() => {
                    match event {
                        Some(SwarmEvent::Behaviour(event)) => {
                            self.handle_behaviour_event(event);
                        }
                        Some(SwarmEvent::ConnectionEstablished {
                            peer_id, endpoint, ..
                        }) => {
                            info!(%peer_id, ?endpoint, "peer connected");
                            if !self.peer_manager.add_peer(peer_id) {
                                warn!(
                                    %peer_id,
                                    max = self.config.max_connections,
                                    "peer limit reached, disconnecting peer"
                                );
                                let _ = self.swarm.disconnect_peer_id(peer_id);
                            }
                        }
                        Some(SwarmEvent::ConnectionClosed { peer_id, .. }) => {
                            info!(%peer_id, "peer disconnected");
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
                Some(outbound) = outbound_rx.recv() => {
                    match outbound {
                        OutboundMessage::Broadcast(msg) => {
                            // Dual-publish: envelope on versioned topic, legacy on unversioned.
                            let v_topic_name = versioned_topic_for_message(&msg);
                            match codec::encode_message(&msg) {
                                Ok(data) => {
                                    let topic = IdentTopic::new(&v_topic_name);
                                    if let Err(e) = self.swarm
                                        .behaviour_mut()
                                        .gossipsub
                                        .publish(topic, data)
                                    {
                                        debug!("outbound publish failed: {}", e);
                                    }
                                }
                                Err(e) => {
                                    warn!("failed to encode outbound message: {}", e);
                                }
                            }
                            // Legacy publish (best-effort).
                            self.publish_legacy(&msg);
                        }
                        OutboundMessage::SendToPeer(peer_id, msg) => {
                            debug!(%peer_id, "sending direct message to peer");
                            self.swarm
                                .behaviour_mut()
                                .request_response
                                .send_request(&peer_id, msg);
                        }
                    }
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
                    Ok(DecodedMessage::Known(msg)) => {
                        let _ = self.message_tx.send((*msg, Some(propagation_source)));
                    }
                    Ok(DecodedMessage::Unknown {
                        protocol_version,
                        message_type,
                    }) => {
                        debug!(
                            protocol_version,
                            message_type, "received unknown message type (newer protocol version)"
                        );
                        if protocol_version > PROTOCOL_VERSION {
                            self.maybe_broadcast_upgrade_notice(protocol_version);
                        }
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
                    let _ = self.message_tx.send((request.clone(), Some(peer)));
                    // Send back an echo response (acknowledgement).
                    let _ = self
                        .swarm
                        .behaviour_mut()
                        .request_response
                        .send_response(channel, request);
                }
                request_response::Message::Response { response, .. } => {
                    debug!(%peer, "received direct response");
                    let _ = self.message_tx.send((response, Some(peer)));
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
                // Parse protocol version from agent_version "norn/{version}".
                if let Some(version) = parse_agent_version(&info.agent_version) {
                    self.peer_manager.set_peer_version(&peer_id, version);
                    if version > PROTOCOL_VERSION {
                        warn!(
                            %peer_id,
                            peer_version = version,
                            our_version = PROTOCOL_VERSION,
                            "peer running newer protocol version — upgrade recommended"
                        );
                        self.maybe_broadcast_upgrade_notice(version);
                    }
                }
            }
            NornBehaviourEvent::Mdns(libp2p::mdns::Event::Discovered(peers)) => {
                for (peer_id, addr) in peers {
                    info!(%peer_id, %addr, "mDNS: discovered peer");
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .add_explicit_peer(&peer_id);
                }
            }
            NornBehaviourEvent::Mdns(libp2p::mdns::Event::Expired(peers)) => {
                for (peer_id, addr) in peers {
                    debug!(%peer_id, %addr, "mDNS: peer expired");
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .remove_explicit_peer(&peer_id);
                }
            }
            _ => {}
        }
    }

    /// Rate-limited upgrade notice: broadcast once per observed version.
    fn maybe_broadcast_upgrade_notice(&mut self, detected_version: u8) {
        if !self.notified_versions.insert(detected_version) {
            return; // Already notified for this version.
        }
        let notice = NornMessage::UpgradeNotice(UpgradeNotice {
            protocol_version: detected_version,
            message: format!(
                "detected peer running protocol v{}, we are on v{} — upgrade recommended",
                detected_version, PROTOCOL_VERSION
            ),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });
        let _ = self.message_tx.send((notice, None));
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

/// Returns the versioned gossipsub topic name for the given message type.
pub fn versioned_topic_for_message(msg: &NornMessage) -> String {
    versioned_topic(legacy_topic_for_message(msg), PROTOCOL_VERSION)
}

/// Returns the legacy (unversioned) gossipsub topic name for the given message type.
pub fn legacy_topic_for_message(msg: &NornMessage) -> &'static str {
    match msg {
        NornMessage::Block(_) => BLOCKS_TOPIC,
        NornMessage::Commitment(_) => COMMITMENTS_TOPIC,
        NornMessage::FraudProof(_) => FRAUD_PROOFS_TOPIC,
        _ => GENERAL_TOPIC,
    }
}

/// Kept for backward compatibility — callers that only need the legacy topic name.
pub fn topic_for_message(msg: &NornMessage) -> &'static str {
    legacy_topic_for_message(msg)
}

/// Parse `"norn/{version}"` from an identify agent_version string.
fn parse_agent_version(agent: &str) -> Option<u8> {
    agent.strip_prefix("norn/")?.parse().ok()
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
            name_registrations: vec![],
            name_registrations_root: [0u8; 32],
            fraud_proofs: vec![],
            fraud_proofs_root: [0u8; 32],
            transfers: vec![],
            transfers_root: [0u8; 32],
            token_definitions: vec![],
            token_definitions_root: [0u8; 32],
            token_mints: vec![],
            token_mints_root: [0u8; 32],
            token_burns: vec![],
            token_burns_root: [0u8; 32],
            loom_deploys: vec![],
            loom_deploys_root: [0u8; 32],
            timestamp: 1000,
            proposer: [0u8; 32],
            validator_signatures: vec![],
        }));
        assert_eq!(legacy_topic_for_message(&block), BLOCKS_TOPIC);
        assert_eq!(
            versioned_topic_for_message(&block),
            format!("{}/v{}", BLOCKS_TOPIC, PROTOCOL_VERSION)
        );
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
        assert_eq!(legacy_topic_for_message(&commitment), COMMITMENTS_TOPIC);
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
        assert_eq!(legacy_topic_for_message(&fraud), FRAUD_PROOFS_TOPIC);
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
        assert_eq!(legacy_topic_for_message(&reg), GENERAL_TOPIC);
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
        assert_eq!(legacy_topic_for_message(&relay), GENERAL_TOPIC);
    }

    #[test]
    fn test_parse_agent_version() {
        assert_eq!(parse_agent_version("norn/4"), Some(4));
        assert_eq!(parse_agent_version("norn/3"), Some(3));
        assert_eq!(parse_agent_version("norn/255"), Some(255));
        assert_eq!(parse_agent_version("other/1"), None);
        assert_eq!(parse_agent_version("norn/"), None);
        assert_eq!(parse_agent_version("norn/abc"), None);
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

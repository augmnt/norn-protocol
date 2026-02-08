use libp2p::gossipsub;
use libp2p::identity::Keypair;
use libp2p::request_response;
use libp2p::swarm::NetworkBehaviour;
use libp2p::StreamProtocol;
use std::time::Duration;

use crate::codec::NornCodec;
use crate::protocol::DIRECT_PROTOCOL;

/// Combined network behaviour for the Norn relay.
#[derive(NetworkBehaviour)]
pub struct NornBehaviour {
    /// Gossipsub for pub/sub broadcast messages.
    pub gossipsub: gossipsub::Behaviour,
    /// Request-response for direct messaging.
    pub request_response: request_response::Behaviour<NornCodec>,
    /// Identify protocol for peer identification.
    pub identify: libp2p::identify::Behaviour,
    /// mDNS for automatic local network peer discovery.
    pub mdns: libp2p::mdns::tokio::Behaviour,
}

/// Build a NornBehaviour from a keypair.
///
/// Returns `Result<NornBehaviour, Box<dyn Error + Send + Sync>>` to conform
/// to the `TryIntoBehaviour` trait expected by `SwarmBuilder::with_behaviour`.
///
/// The `protocol_version` is advertised via the identify protocol's agent version
/// string as `"norn/{version}"`, allowing peers to detect version mismatches.
pub fn build_behaviour(
    keypair: &Keypair,
    protocol_version: u8,
) -> Result<NornBehaviour, Box<dyn std::error::Error + Send + Sync>> {
    // --- Gossipsub ---
    let message_id_fn = |message: &gossipsub::Message| {
        // Deduplicate based on content hash.
        let hash = blake3::hash(&message.data);
        gossipsub::MessageId::from(hash.as_bytes().to_vec())
    };

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(1))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .message_id_fn(message_id_fn)
        .build()
        .map_err(|e| format!("gossipsub config: {}", e))?;

    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(keypair.clone()),
        gossipsub_config,
    )
    .map_err(|e| format!("gossipsub behaviour: {}", e))?;

    // --- Request-Response ---
    let rr_protocol = StreamProtocol::try_from_owned(DIRECT_PROTOCOL.to_string())
        .map_err(|e| format!("invalid protocol: {:?}", e))?;

    let request_response = request_response::Behaviour::with_codec(
        NornCodec,
        [(rr_protocol, request_response::ProtocolSupport::Full)],
        request_response::Config::default(),
    );

    // --- Identify ---
    let identify_config =
        libp2p::identify::Config::new("/norn/1.0.0".to_string(), keypair.public())
            .with_agent_version(format!("norn/{}", protocol_version));
    let identify = libp2p::identify::Behaviour::new(identify_config);

    // --- mDNS ---
    let mdns = libp2p::mdns::tokio::Behaviour::new(
        libp2p::mdns::Config::default(),
        keypair.public().to_peer_id(),
    )?;

    Ok(NornBehaviour {
        gossipsub,
        request_response,
        identify,
        mdns,
    })
}

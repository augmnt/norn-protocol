use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::consensus::ConsensusMessage;
use crate::fraud::FraudProofSubmission;
use crate::knot::Knot;
use crate::primitives::*;
use crate::weave::{CommitmentUpdate, Registration, WeaveBlock};

/// Network identifier for distinguishing dev, testnet, and mainnet environments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkId {
    Dev,
    Testnet,
    Mainnet,
}

impl NetworkId {
    /// Returns the chain ID string for this network.
    pub fn chain_id(&self) -> &'static str {
        match self {
            NetworkId::Dev => "norn-dev",
            NetworkId::Testnet => "norn-testnet-1",
            NetworkId::Mainnet => "norn-mainnet",
        }
    }

    /// Whether the faucet is available on this network.
    pub fn faucet_enabled(&self) -> bool {
        matches!(self, NetworkId::Dev | NetworkId::Testnet)
    }

    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            NetworkId::Dev => "Development",
            NetworkId::Testnet => "Testnet",
            NetworkId::Mainnet => "Mainnet",
        }
    }

    /// Short lowercase identifier (for CLI/config).
    pub fn as_str(&self) -> &'static str {
        match self {
            NetworkId::Dev => "dev",
            NetworkId::Testnet => "testnet",
            NetworkId::Mainnet => "mainnet",
        }
    }

    /// Parse from a string identifier.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "dev" => Some(NetworkId::Dev),
            "testnet" => Some(NetworkId::Testnet),
            "mainnet" => Some(NetworkId::Mainnet),
            _ => None,
        }
    }

    /// Default faucet cooldown in seconds for this network.
    pub fn faucet_cooldown(&self) -> u64 {
        match self {
            NetworkId::Dev => 60,
            NetworkId::Testnet => 3600,
            NetworkId::Mainnet => 0, // faucet disabled
        }
    }
}

impl std::fmt::Display for NetworkId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}

/// A message relayed between spindles.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct RelayMessage {
    /// Sender's address.
    pub from: Address,
    /// Recipient's address.
    pub to: Address,
    /// The message payload.
    pub payload: Vec<u8>,
    /// Timestamp of the message.
    pub timestamp: Timestamp,
    /// Signature by the sender.
    #[serde(with = "crate::primitives::serde_sig")]
    pub signature: Signature,
}

/// A spindle's registration with a relay.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct SpindleRegistration {
    /// The spindle's public key.
    pub pubkey: PublicKey,
    /// The spindle's address.
    pub address: Address,
    /// Relay endpoint (host:port).
    pub relay_endpoint: String,
    /// Timestamp of registration.
    pub timestamp: Timestamp,
    /// Signature by the spindle.
    #[serde(with = "crate::primitives::serde_sig")]
    pub signature: Signature,
}

/// A spindle status update sent to a relay.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct SpindleUpdate {
    /// The spindle's address.
    pub address: Address,
    /// Whether the spindle is online.
    pub online: bool,
    /// Latest thread version.
    pub latest_version: Version,
    /// Timestamp of this update.
    pub timestamp: Timestamp,
    /// Signature by the spindle.
    #[serde(with = "crate::primitives::serde_sig")]
    pub signature: Signature,
}

/// An alert from a spindle about suspicious activity.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct SpindleAlert {
    /// The spindle raising the alert.
    pub from: Address,
    /// The subject of the alert.
    pub subject: Address,
    /// Description of the alert.
    pub reason: String,
    /// Timestamp of the alert.
    pub timestamp: Timestamp,
    /// Signature by the alerting spindle.
    #[serde(with = "crate::primitives::serde_sig")]
    pub signature: Signature,
}

/// Versioned envelope for P2P messages. Wraps borsh-encoded payloads so that
/// nodes can skip unknown `message_type` values instead of crashing.
///
/// Nodes that receive an unknown `message_type` log a debug warning and drop
/// the message, maintaining forward compatibility when new message types are
/// added.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct MessageEnvelope {
    /// Envelope version (currently 1).
    pub version: u8,
    /// Known message type discriminator. Corresponds to the `NornMessage` enum
    /// variant index.
    pub message_type: u8,
    /// Borsh-encoded inner message payload.
    pub payload: Vec<u8>,
}

impl MessageEnvelope {
    /// Wrap a `NornMessage` into a versioned envelope.
    pub fn wrap(msg: &NornMessage) -> Result<Self, std::io::Error> {
        let message_type = msg.discriminant();
        let payload = borsh::to_vec(msg)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Self {
            version: 1,
            message_type,
            payload,
        })
    }

    /// Unwrap the envelope back into a `NornMessage`.
    ///
    /// Returns `None` if the `message_type` is unknown (forward-compatible skip).
    pub fn unwrap_message(&self) -> Option<NornMessage> {
        NornMessage::try_from_slice(&self.payload).ok()
    }
}

/// Top-level Norn protocol message envelope.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum NornMessage {
    /// A knot proposal between two parties.
    KnotProposal(Box<Knot>),
    /// A knot response (co-signed knot).
    KnotResponse(Box<Knot>),
    /// A commitment update for the weave.
    Commitment(CommitmentUpdate),
    /// A thread registration.
    Registration(Registration),
    /// A relay message.
    Relay(RelayMessage),
    /// A spindle registration.
    SpindleReg(SpindleRegistration),
    /// A spindle status update.
    SpindleStatus(SpindleUpdate),
    /// A spindle alert.
    Alert(SpindleAlert),
    /// A fraud proof submission.
    FraudProof(Box<FraudProofSubmission>),
    /// A weave block.
    Block(Box<WeaveBlock>),
    /// A consensus protocol message.
    Consensus(ConsensusMessage),
    /// Request state from peers (used for initial sync).
    StateRequest {
        /// The requester's current block height.
        current_height: u64,
        /// Genesis hash for chain identity validation.
        genesis_hash: Hash,
    },
    /// Response with blocks for state sync.
    StateResponse {
        /// Blocks to apply.
        blocks: Vec<WeaveBlock>,
        /// The sender's tip height.
        tip_height: u64,
        /// Genesis hash for chain identity validation.
        genesis_hash: Hash,
    },
}

impl NornMessage {
    /// Returns a stable discriminant byte for this message variant.
    /// Used by `MessageEnvelope` for forward-compatible type tagging.
    pub fn discriminant(&self) -> u8 {
        match self {
            NornMessage::KnotProposal(_) => 0,
            NornMessage::KnotResponse(_) => 1,
            NornMessage::Commitment(_) => 2,
            NornMessage::Registration(_) => 3,
            NornMessage::Relay(_) => 4,
            NornMessage::SpindleReg(_) => 5,
            NornMessage::SpindleStatus(_) => 6,
            NornMessage::Alert(_) => 7,
            NornMessage::FraudProof(_) => 8,
            NornMessage::Block(_) => 9,
            NornMessage::Consensus(_) => 10,
            NornMessage::StateRequest { .. } => 11,
            NornMessage::StateResponse { .. } => 12,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weave::Registration;

    fn sample_message() -> NornMessage {
        NornMessage::Registration(Registration {
            thread_id: [1u8; 20],
            owner: [2u8; 32],
            initial_state_hash: [3u8; 32],
            timestamp: 1000,
            signature: [4u8; 64],
        })
    }

    #[test]
    fn test_envelope_roundtrip() {
        let msg = sample_message();
        let envelope = MessageEnvelope::wrap(&msg).expect("wrap failed");
        assert_eq!(envelope.version, 1);
        assert_eq!(envelope.message_type, 3); // Registration = discriminant 3
        let unwrapped = envelope.unwrap_message().expect("unwrap failed");
        assert_eq!(msg, unwrapped);
    }

    #[test]
    fn test_envelope_unknown_type_returns_none() {
        // Simulate an envelope with unknown message type and garbage payload.
        let envelope = MessageEnvelope {
            version: 1,
            message_type: 255, // unknown
            payload: vec![0xFF, 0xFF, 0xFF],
        };
        assert!(envelope.unwrap_message().is_none());
    }

    #[test]
    fn test_discriminant_values() {
        let msg = NornMessage::StateRequest {
            current_height: 0,
            genesis_hash: [0u8; 32],
        };
        assert_eq!(msg.discriminant(), 11);
    }
}

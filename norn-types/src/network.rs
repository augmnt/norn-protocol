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

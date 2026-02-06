use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::consensus::ConsensusMessage;
use crate::fraud::FraudProofSubmission;
use crate::knot::Knot;
use crate::primitives::*;
use crate::weave::{CommitmentUpdate, Registration, WeaveBlock};

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
}

/// Wire protocol version. Bump this whenever a breaking change is made to
/// NornMessage variants or any borsh-serialized P2P type.
pub const PROTOCOL_VERSION: u8 = 3;

/// Gossipsub topic for general network messages.
pub const GOSSIP_PROTOCOL: &str = "/norn/gossip/1.0.0";

/// Direct message protocol.
pub const DIRECT_PROTOCOL: &str = "/norn/direct/1.0.0";

/// Spindle registration protocol.
pub const SPINDLE_PROTOCOL: &str = "/norn/spindle/1.0.0";

/// Gossipsub topic name for block announcements.
pub const BLOCKS_TOPIC: &str = "norn/blocks";

/// Gossipsub topic name for commitments.
pub const COMMITMENTS_TOPIC: &str = "norn/commitments";

/// Gossipsub topic name for fraud proofs.
pub const FRAUD_PROOFS_TOPIC: &str = "norn/fraud-proofs";

/// Gossipsub topic name for general/default messages.
pub const GENERAL_TOPIC: &str = "norn/general";

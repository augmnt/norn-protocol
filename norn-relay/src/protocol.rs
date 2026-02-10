/// Wire protocol version. Bump this whenever a breaking change is made to
/// NornMessage variants or any borsh-serialized P2P type.
pub const PROTOCOL_VERSION: u8 = 8;

/// Envelope wire header version. The first byte after the 4-byte length prefix.
/// Since this is 1 and the old LEGACY_PROTOCOL_VERSION was 3, the codec can
/// unambiguously detect which format a peer is using by inspecting byte[4].
pub const ENVELOPE_VERSION: u8 = 1;

/// The protocol version used by pre-envelope nodes (v0.5.x).
/// Used for dual-decode: if byte[4] == 3, treat as legacy raw NornMessage.
pub const LEGACY_PROTOCOL_VERSION: u8 = 3;

/// Gossipsub topic for general network messages.
pub const GOSSIP_PROTOCOL: &str = "/norn/gossip/1.0.0";

/// Direct message protocol.
pub const DIRECT_PROTOCOL: &str = "/norn/direct/1.0.0";

/// Spindle registration protocol.
pub const SPINDLE_PROTOCOL: &str = "/norn/spindle/1.0.0";

/// Legacy (unversioned) gossipsub topic name for block announcements.
pub const BLOCKS_TOPIC: &str = "norn/blocks";

/// Legacy (unversioned) gossipsub topic name for commitments.
pub const COMMITMENTS_TOPIC: &str = "norn/commitments";

/// Legacy (unversioned) gossipsub topic name for fraud proofs.
pub const FRAUD_PROOFS_TOPIC: &str = "norn/fraud-proofs";

/// Legacy (unversioned) gossipsub topic name for general/default messages.
pub const GENERAL_TOPIC: &str = "norn/general";

/// Build a versioned topic string: `"{base}/v{version}"`.
pub fn versioned_topic(base: &str, version: u8) -> String {
    format!("{}/v{}", base, version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_versioned_topic() {
        assert_eq!(versioned_topic("norn/blocks", 4), "norn/blocks/v4");
        assert_eq!(versioned_topic("norn/general", 4), "norn/general/v4");
    }

    #[test]
    fn test_envelope_version_differs_from_legacy() {
        // This is the key invariant that makes dual-decode work.
        assert_ne!(ENVELOPE_VERSION, LEGACY_PROTOCOL_VERSION);
    }
}

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// 32-byte BLAKE3 hash.
pub type Hash = [u8; 32];

/// 32-byte Ed25519 public key.
pub type PublicKey = [u8; 32];

/// 64-byte Ed25519 signature.
pub type Signature = [u8; 64];

/// 20-byte address derived from BLAKE3(pubkey)[0..20].
pub type Address = [u8; 20];

/// Token identifier — 32-byte hash of the token definition.
pub type TokenId = [u8; 32];

/// Unique identifier for a thread — same as the creator's address.
pub type ThreadId = Address;

/// Unique identifier for a knot — BLAKE3 hash of all fields except signatures.
pub type KnotId = Hash;

/// Unique identifier for a loom.
pub type LoomId = [u8; 32];

/// Version number for knots within a thread (monotonically increasing).
pub type Version = u64;

/// Amount of tokens (native uses 12 decimals).
pub type Amount = u128;

/// Unix timestamp in seconds.
pub type Timestamp = u64;

/// The native token ID (all zeros).
pub const NATIVE_TOKEN_ID: TokenId = [0u8; 32];

/// Serde helper for [u8; 64] fields.
pub mod serde_sig {
    use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as a byte slice
        value.as_slice().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Vec<u8> = Vec::deserialize(deserializer)?;
        v.try_into()
            .map_err(|_| serde::de::Error::custom("expected 64 bytes for signature"))
    }
}

/// Serde helper for Vec<[u8; 64]> fields (signature arrays).
pub mod serde_sig_vec {
    use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &[[u8; 64]], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let byte_vecs: Vec<&[u8]> = value.iter().map(|s| s.as_slice()).collect();
        byte_vecs.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<[u8; 64]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vecs: Vec<Vec<u8>> = Vec::deserialize(deserializer)?;
        vecs.into_iter()
            .map(|v| {
                v.try_into()
                    .map_err(|_| serde::de::Error::custom("expected 64 bytes for signature"))
            })
            .collect()
    }
}

/// A signed amount that can represent debits (negative) and credits (positive).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
pub struct SignedAmount {
    /// True if the amount is negative.
    pub negative: bool,
    /// Absolute value of the amount.
    pub value: Amount,
}

impl SignedAmount {
    pub fn zero() -> Self {
        Self {
            negative: false,
            value: 0,
        }
    }

    pub fn positive(value: Amount) -> Self {
        Self {
            negative: false,
            value,
        }
    }

    pub fn negative(value: Amount) -> Self {
        Self {
            negative: true,
            value,
        }
    }
}

use thiserror::Error;

/// All error codes for the Norn protocol (Appendix D).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NornError {
    // ─── Knot Validation Errors ──────────────────────────────────────────────
    #[error("invalid signature: signer {signer_index}")]
    InvalidSignature { signer_index: usize },

    #[error("knot ID mismatch: expected {expected:?}, got {actual:?}")]
    KnotIdMismatch {
        expected: [u8; 32],
        actual: [u8; 32],
    },

    #[error(
        "version mismatch for participant {participant_index}: expected {expected}, got {actual}"
    )]
    VersionMismatch {
        participant_index: usize,
        expected: u64,
        actual: u64,
    },

    #[error("state hash mismatch for participant {participant_index}")]
    StateHashMismatch { participant_index: usize },

    #[error("insufficient balance: have {available}, need {required}")]
    InsufficientBalance { available: u128, required: u128 },

    #[error("invalid amount: amount must be positive")]
    InvalidAmount,

    #[error("timestamp too far in the future: {timestamp} > {max_allowed}")]
    TimestampTooFuture { timestamp: u64, max_allowed: u64 },

    #[error("timestamp before previous knot: {timestamp} < {previous}")]
    TimestampBeforePrevious { timestamp: u64, previous: u64 },

    #[error("knot expired at {expiry}, current time is {current}")]
    KnotExpired { expiry: u64, current: u64 },

    #[error("payload internally inconsistent: {reason}")]
    PayloadInconsistent { reason: String },

    // ─── Thread Errors ───────────────────────────────────────────────────────
    #[error("thread not found: {0:?}")]
    ThreadNotFound([u8; 20]),

    #[error("thread already exists: {0:?}")]
    ThreadAlreadyExists([u8; 20]),

    #[error("too many uncommitted knots: {count} >= {max}")]
    TooManyUncommittedKnots { count: usize, max: usize },

    #[error("invalid knot chain: gap at index {index}")]
    InvalidKnotChain { index: usize },

    // ─── Crypto Errors ───────────────────────────────────────────────────────
    #[error("invalid key material")]
    InvalidKeyMaterial,

    #[error("invalid mnemonic phrase")]
    InvalidMnemonic,

    #[error("derivation failed: {reason}")]
    DerivationFailed { reason: String },

    #[error("shamir secret sharing error: {reason}")]
    ShamirError { reason: String },

    #[error("encryption failed: {reason}")]
    EncryptionFailed { reason: String },

    #[error("decryption failed: {reason}")]
    DecryptionFailed { reason: String },

    // ─── Merkle Tree Errors ──────────────────────────────────────────────────
    #[error("merkle proof verification failed")]
    MerkleProofInvalid,

    #[error("key not found in merkle tree")]
    MerkleKeyNotFound,

    // ─── Weave Errors ────────────────────────────────────────────────────────
    #[error("invalid weave block: {reason}")]
    InvalidWeaveBlock { reason: String },

    #[error("stale commitment: age {age}s exceeds max {max_age}s")]
    StaleCommitment { age: u64, max_age: u64 },

    // ─── Loom Errors ─────────────────────────────────────────────────────────
    #[error("loom not found: {0:?}")]
    LoomNotFound([u8; 32]),

    #[error("loom participant limit exceeded: {count} > {max}")]
    LoomParticipantLimit { count: usize, max: usize },

    #[error("not a loom participant")]
    NotLoomParticipant,

    // ─── Name Registry Errors ─────────────────────────────────────────────────
    #[error("name already registered: {0}")]
    NameAlreadyRegistered(String),

    #[error("invalid name: {0}")]
    InvalidName(String),

    // ─── Serialization Errors ────────────────────────────────────────────────
    #[error("serialization error: {reason}")]
    SerializationError { reason: String },

    #[error("deserialization error: {reason}")]
    DeserializationError { reason: String },

    // ─── Network Errors ──────────────────────────────────────────────────────
    #[error("message too large: {size} > {max_size}")]
    MessageTooLarge { size: usize, max_size: usize },

    #[error("invalid message format: {reason}")]
    InvalidMessageFormat { reason: String },

    // ─── Arithmetic Errors ──────────────────────────────────────────────────
    #[error("balance overflow")]
    BalanceOverflow,

    #[error("version overflow")]
    VersionOverflow,

    #[error("insufficient participants: need at least {required}, got {actual}")]
    InsufficientParticipants { required: usize, actual: usize },
}

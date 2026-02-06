use crate::primitives::Amount;
use std::time::Duration;

// ─── Token Parameters ────────────────────────────────────────────────────────

/// Number of decimal places for the native NORN token.
pub const NORN_DECIMALS: u32 = 12;

/// One full NORN token in base units (10^12).
pub const ONE_NORN: Amount = 1_000_000_000_000;

/// Maximum supply of NORN tokens (in base units).
pub const MAX_SUPPLY: Amount = 1_000_000_000 * ONE_NORN; // 1 billion NORN

// ─── Knot Parameters ─────────────────────────────────────────────────────────

/// Maximum size of a memo field in bytes.
pub const MAX_MEMO_SIZE: usize = 256;

/// Maximum number of transfers in a multi-transfer knot.
pub const MAX_MULTI_TRANSFERS: usize = 64;

/// Maximum timestamp drift into the future (seconds).
pub const MAX_TIMESTAMP_DRIFT: u64 = 300; // 5 minutes

/// Default knot expiry duration (seconds).
pub const DEFAULT_KNOT_EXPIRY: u64 = 3600; // 1 hour

// ─── Weave Parameters ────────────────────────────────────────────────────────

/// Target time between weave blocks.
pub const BLOCK_TIME_TARGET: Duration = Duration::from_secs(3);

/// Maximum number of commitment updates per weave block.
pub const MAX_COMMITMENTS_PER_BLOCK: usize = 10_000;

/// Number of blocks before a commitment is considered finalized.
pub const COMMITMENT_FINALITY_DEPTH: u64 = 10;

/// Maximum age of a commitment before it's considered stale (seconds).
pub const MAX_COMMITMENT_AGE: u64 = 86_400; // 24 hours

// ─── Loom Parameters ─────────────────────────────────────────────────────────

/// Maximum number of participants in a loom.
pub const MAX_LOOM_PARTICIPANTS: usize = 1_000;

/// Minimum number of participants for a loom to be active.
pub const MIN_LOOM_PARTICIPANTS: usize = 2;

/// Maximum loom state size in bytes.
pub const MAX_LOOM_STATE_SIZE: usize = 1_048_576; // 1 MB

// ─── Network Parameters ──────────────────────────────────────────────────────

/// Maximum message size in bytes.
pub const MAX_MESSAGE_SIZE: usize = 2_097_152; // 2 MB

/// Default relay port.
pub const DEFAULT_RELAY_PORT: u16 = 9740;

/// Maximum number of relay connections per spindle.
pub const MAX_RELAY_CONNECTIONS: usize = 50;

// ─── Thread Parameters ───────────────────────────────────────────────────────

/// Maximum number of unconfirmed knots before a commitment is required.
pub const MAX_UNCOMMITTED_KNOTS: usize = 1_000;

/// Thread header size in bytes (fixed).
pub const THREAD_HEADER_SIZE: usize = 208;

// ─── Epoch Parameters ───────────────────────────────────────────────────────

/// Number of blocks per epoch (validator set rotation period).
pub const BLOCKS_PER_EPOCH: u64 = 1_000;

// ─── Fraud Proof Parameters ──────────────────────────────────────────────────

/// Time window for submitting a fraud proof after a commitment (seconds).
pub const FRAUD_PROOF_WINDOW: u64 = 86_400; // 24 hours

/// Minimum stake required to submit a fraud proof.
pub const FRAUD_PROOF_MIN_STAKE: Amount = ONE_NORN;

// ─── Derivation Path ─────────────────────────────────────────────────────────

/// Coin type for SLIP-44 registration (placeholder — not yet registered).
pub const NORN_COIN_TYPE: u32 = 0x4E4F524E; // "NORN" in hex

use serde::{Deserialize, Serialize};

/// Information about a thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadInfo {
    /// Thread ID as hex string.
    pub thread_id: String,
    /// Owner public key as hex string.
    pub owner: String,
    /// Current version number.
    pub version: u64,
    /// Current state hash as hex string.
    pub state_hash: String,
}

/// Information about a weave block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    /// Block height.
    pub height: u64,
    /// Block hash as hex string.
    pub hash: String,
    /// Previous block hash as hex string.
    pub prev_hash: String,
    /// Block timestamp.
    pub timestamp: u64,
    /// Proposer public key as hex string.
    pub proposer: String,
    /// Number of commitment updates in this block.
    pub commitment_count: usize,
    /// Number of registrations in this block.
    pub registration_count: usize,
    /// Number of loom anchors in this block.
    pub anchor_count: usize,
    /// Number of fraud proofs in this block.
    pub fraud_proof_count: usize,
}

/// Information about the current weave state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaveStateInfo {
    /// Current block height.
    pub height: u64,
    /// Latest block hash as hex string.
    pub latest_hash: String,
    /// Threads Merkle root as hex string.
    pub threads_root: String,
    /// Total number of registered threads.
    pub thread_count: u64,
    /// Current base fee.
    pub base_fee: String,
    /// Fee multiplier (scaled by 1000).
    pub fee_multiplier: u64,
}

/// Result of submitting a commitment or registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResult {
    /// Whether the submission was accepted.
    pub success: bool,
    /// Reason for failure, if any.
    pub reason: Option<String>,
}

/// Thread state info with balance details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadStateInfo {
    /// Thread ID as hex string.
    pub thread_id: String,
    /// Owner public key as hex string.
    pub owner: String,
    /// Current version number.
    pub version: u64,
    /// Current state hash as hex string.
    pub state_hash: String,
    /// Token balances.
    pub balances: Vec<BalanceEntry>,
}

/// A single balance entry for a token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceEntry {
    /// Token ID as hex string.
    pub token_id: String,
    /// Raw amount as string.
    pub amount: String,
    /// Human-readable formatted amount.
    pub human_readable: String,
}

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthInfo {
    /// Current block height.
    pub height: u64,
    /// Whether the node is a validator.
    pub is_validator: bool,
    /// Number of registered threads.
    pub thread_count: u64,
    /// Node uptime status.
    pub status: String,
}

/// Information about a validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    /// Public key as hex string.
    pub pubkey: String,
    /// Address as hex string.
    pub address: String,
    /// Staked amount as string.
    pub stake: String,
    /// Whether the validator is active.
    pub active: bool,
}

/// Information about the current validator set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSetInfo {
    /// List of validators.
    pub validators: Vec<ValidatorInfo>,
    /// Total staked amount.
    pub total_stake: String,
    /// Current epoch.
    pub epoch: u64,
}

/// Fee estimate response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEstimateInfo {
    /// Estimated fee for one commitment in nits.
    pub fee_per_commitment: String,
    /// Current base fee in nits.
    pub base_fee: String,
    /// Current fee multiplier (scaled by 1000).
    pub fee_multiplier: u64,
}

/// Merkle proof for a thread commitment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentProofInfo {
    /// Thread ID as hex string.
    pub thread_id: String,
    /// Merkle proof key as hex string.
    pub key: String,
    /// Merkle proof value as hex string.
    pub value: String,
    /// Sibling hashes as hex strings.
    pub siblings: Vec<String>,
}

/// A single entry in the transaction history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionHistoryEntry {
    /// Knot ID as hex string.
    pub knot_id: String,
    /// Sender address as hex string.
    pub from: String,
    /// Recipient address as hex string.
    pub to: String,
    /// Token ID as hex string.
    pub token_id: String,
    /// Raw amount as string.
    pub amount: String,
    /// Human-readable formatted amount.
    pub human_readable: String,
    /// Optional memo as UTF-8 string.
    pub memo: Option<String>,
    /// Timestamp of the transfer.
    pub timestamp: u64,
    /// Block height (if included in a block).
    pub block_height: Option<u64>,
    /// Direction relative to the queried address: "sent" or "received".
    pub direction: String,
}

/// Result of resolving a name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameResolution {
    /// The registered name.
    pub name: String,
    /// Owner address as hex string.
    pub owner: String,
    /// Timestamp when the name was registered.
    pub registered_at: u64,
    /// Fee paid for registration as string.
    pub fee_paid: String,
}

/// Information about a name owned by an address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameInfo {
    /// The registered name.
    pub name: String,
    /// Timestamp when the name was registered.
    pub registered_at: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_info_serialization() {
        let info = BlockInfo {
            height: 42,
            hash: "abc123".to_string(),
            prev_hash: "def456".to_string(),
            timestamp: 1700000000,
            proposer: "aabbcc".to_string(),
            commitment_count: 10,
            registration_count: 2,
            anchor_count: 1,
            fraud_proof_count: 0,
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: BlockInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.height, 42);
    }

    #[test]
    fn test_weave_state_info_serialization() {
        let info = WeaveStateInfo {
            height: 100,
            latest_hash: "deadbeef".to_string(),
            threads_root: "cafe".to_string(),
            thread_count: 50,
            base_fee: "100".to_string(),
            fee_multiplier: 1000,
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: WeaveStateInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.height, 100);
        assert_eq!(deserialized.thread_count, 50);
    }
}

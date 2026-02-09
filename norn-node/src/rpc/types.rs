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
    /// Number of name registrations in this block.
    #[serde(default)]
    pub name_registration_count: usize,
    /// Number of transfers in this block.
    #[serde(default)]
    pub transfer_count: usize,
    /// Number of token definitions in this block.
    #[serde(default)]
    pub token_definition_count: usize,
    /// Number of token mints in this block.
    #[serde(default)]
    pub token_mint_count: usize,
    /// Number of token burns in this block.
    #[serde(default)]
    pub token_burn_count: usize,
    /// Number of loom deployments in this block.
    #[serde(default)]
    pub loom_deploy_count: usize,
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
    /// Network identifier (e.g., "dev", "testnet", "mainnet").
    pub network: String,
    /// Chain ID (e.g., "norn-dev", "norn-testnet-1", "norn-mainnet").
    pub chain_id: String,
    /// Node software version.
    pub version: String,
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

/// Information about a token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    /// Token ID as hex string.
    pub token_id: String,
    /// Human-readable name (e.g., "Wrapped Bitcoin").
    pub name: String,
    /// Ticker symbol (e.g., "WBTC").
    pub symbol: String,
    /// Decimal places.
    pub decimals: u8,
    /// Maximum supply (0 = unlimited), as string.
    pub max_supply: String,
    /// Current circulating supply, as string.
    pub current_supply: String,
    /// Creator address as hex string.
    pub creator: String,
    /// Creation timestamp.
    pub created_at: u64,
}

/// Information about a deployed loom (smart contract).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoomInfo {
    /// Loom ID as hex string.
    pub loom_id: String,
    /// Human-readable name.
    pub name: String,
    /// Operator public key as hex string.
    pub operator: String,
    /// Whether the loom is active.
    pub active: bool,
    /// Deployment timestamp.
    pub deployed_at: u64,
    /// Whether bytecode has been uploaded.
    #[serde(default)]
    pub has_bytecode: bool,
    /// Number of active participants.
    #[serde(default)]
    pub participant_count: usize,
}

/// A key-value attribute in a structured event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeInfo {
    /// Attribute key.
    pub key: String,
    /// Attribute value.
    pub value: String,
}

/// A structured event emitted by a loom contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInfo {
    /// Event type (e.g., "Transfer", "Approval").
    #[serde(rename = "type")]
    pub ty: String,
    /// Key-value attributes.
    pub attributes: Vec<AttributeInfo>,
}

/// Result of executing a loom contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Whether execution succeeded.
    pub success: bool,
    /// Output data as hex string.
    pub output_hex: Option<String>,
    /// Gas consumed.
    pub gas_used: u64,
    /// Log messages from execution.
    pub logs: Vec<String>,
    /// Structured events from execution.
    #[serde(default)]
    pub events: Vec<EventInfo>,
    /// Reason for failure, if any.
    pub reason: Option<String>,
}

/// Result of querying a loom contract (read-only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Whether query succeeded.
    pub success: bool,
    /// Output data as hex string.
    pub output_hex: Option<String>,
    /// Gas consumed.
    pub gas_used: u64,
    /// Log messages from query.
    pub logs: Vec<String>,
    /// Structured events from query.
    #[serde(default)]
    pub events: Vec<EventInfo>,
    /// Reason for failure, if any.
    pub reason: Option<String>,
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
            name_registration_count: 3,
            transfer_count: 5,
            token_definition_count: 1,
            token_mint_count: 2,
            token_burn_count: 0,
            loom_deploy_count: 4,
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: BlockInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.height, 42);
        assert_eq!(deserialized.name_registration_count, 3);
        assert_eq!(deserialized.transfer_count, 5);
        assert_eq!(deserialized.token_definition_count, 1);
        assert_eq!(deserialized.token_mint_count, 2);
        assert_eq!(deserialized.token_burn_count, 0);
        assert_eq!(deserialized.loom_deploy_count, 4);
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

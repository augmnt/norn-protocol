use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::primitives::*;

/// Current genesis config version. Bump when making breaking changes to
/// GenesisConfig or GenesisParameters that would alter the genesis hash.
pub const GENESIS_CONFIG_VERSION: u32 = 1;

/// Configuration for the genesis block.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Genesis config version — included in genesis hash for explicit chain identity.
    #[serde(default = "default_genesis_version")]
    pub version: u32,
    /// Chain identifier.
    pub chain_id: String,
    /// Genesis timestamp.
    pub timestamp: Timestamp,
    /// Initial validators.
    pub validators: Vec<GenesisValidator>,
    /// Initial token allocations.
    pub allocations: Vec<GenesisAllocation>,
    /// Protocol parameters.
    pub parameters: GenesisParameters,
    /// Names to register at genesis.
    #[serde(default)]
    pub name_registrations: Vec<GenesisNameRegistration>,
}

fn default_genesis_version() -> u32 {
    GENESIS_CONFIG_VERSION
}

/// A validator in the genesis configuration.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct GenesisValidator {
    /// Validator's public key.
    pub pubkey: PublicKey,
    /// Validator's address.
    pub address: Address,
    /// Initial stake amount.
    pub stake: Amount,
}

/// An initial token allocation.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct GenesisAllocation {
    /// Recipient address.
    pub address: Address,
    /// Token ID.
    pub token_id: TokenId,
    /// Amount to allocate.
    pub amount: Amount,
}

/// A name registration included in the genesis config.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct GenesisNameRegistration {
    /// The name to register.
    pub name: String,
    /// Owner address for the name.
    pub owner: Address,
}

/// Protocol parameters set at genesis.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct GenesisParameters {
    /// Target block time in seconds.
    pub block_time_target: u64,
    /// Maximum commitments per block.
    pub max_commitments_per_block: u64,
    /// Commitment finality depth.
    pub commitment_finality_depth: u64,
    /// Fraud proof window in seconds.
    pub fraud_proof_window: u64,
    /// Minimum stake to be a validator.
    pub min_validator_stake: Amount,
    /// Initial base fee.
    pub initial_base_fee: Amount,
}

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::primitives::*;

/// Configuration for the genesis block.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct GenesisConfig {
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

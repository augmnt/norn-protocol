use norn_crypto::hash::blake3_hash;
use norn_types::genesis::{
    GenesisAllocation, GenesisConfig, GenesisNameRegistration, GenesisParameters, GenesisValidator,
};
use norn_types::primitives::{Address, NATIVE_TOKEN_ID};
use norn_types::weave::{FeeState, WeaveBlock, WeaveState};

use crate::error::NodeError;

/// Create a genesis block and initial weave state from a genesis config.
pub fn create_genesis_block(config: &GenesisConfig) -> Result<(WeaveBlock, WeaveState), NodeError> {
    // Build a genesis block at height 0 with empty content.
    let mut block = WeaveBlock {
        height: 0,
        hash: [0u8; 32],
        prev_hash: [0u8; 32],
        commitments_root: [0u8; 32],
        registrations_root: [0u8; 32],
        anchors_root: [0u8; 32],
        commitments: Vec::new(),
        registrations: Vec::new(),
        anchors: Vec::new(),
        name_registrations: Vec::new(),
        name_registrations_root: [0u8; 32],
        fraud_proofs: Vec::new(),
        fraud_proofs_root: [0u8; 32],
        transfers: Vec::new(),
        transfers_root: [0u8; 32],
        token_definitions: Vec::new(),
        token_definitions_root: [0u8; 32],
        token_mints: Vec::new(),
        token_mints_root: [0u8; 32],
        token_burns: Vec::new(),
        token_burns_root: [0u8; 32],
        loom_deploys: Vec::new(),
        loom_deploys_root: [0u8; 32],
        stake_operations: Vec::new(),
        stake_operations_root: [0u8; 32],
        state_root: [0u8; 32],
        timestamp: config.timestamp,
        proposer: [0u8; 32],
        validator_signatures: Vec::new(),
    };

    // Compute the genesis block hash from its fields.
    block.hash = compute_genesis_hash(&block, config);

    // Create initial weave state.
    let state = WeaveState {
        height: 0,
        latest_hash: block.hash,
        threads_root: [0u8; 32],
        thread_count: 0,
        fee_state: FeeState {
            base_fee: config.parameters.initial_base_fee,
            fee_multiplier: 1000, // 1.0x
            epoch_fees: 0,
        },
    };

    Ok((block, state))
}

/// Compute a deterministic hash for the genesis block, incorporating the chain_id
/// and genesis config version for explicit chain identity.
pub fn compute_genesis_hash(block: &WeaveBlock, config: &GenesisConfig) -> [u8; 32] {
    let mut data = Vec::new();
    data.extend_from_slice(&config.version.to_le_bytes());
    data.extend_from_slice(config.chain_id.as_bytes());
    data.extend_from_slice(&block.height.to_le_bytes());
    data.extend_from_slice(&block.prev_hash);
    data.extend_from_slice(&block.commitments_root);
    data.extend_from_slice(&block.registrations_root);
    data.extend_from_slice(&block.anchors_root);
    data.extend_from_slice(&block.fraud_proofs_root);
    data.extend_from_slice(&block.timestamp.to_le_bytes());
    data.extend_from_slice(&block.proposer);
    blake3_hash(&data)
}

/// Augmnt founder address — private key held securely in local encrypted wallet.
///
/// Every `--dev` node allocates 10M NORN to this address at genesis.
/// Only the wallet holder can sign transactions from this address.
const DEVNET_FOUNDER: Address = [
    0x55, 0x7d, 0xed, 0xe0, 0x78, 0x28, 0xfc, 0x8e, 0xa6, 0x64, 0x77, 0xa6, 0x05, 0x6d, 0xbd, 0x44,
    0x6a, 0x64, 0x00, 0x03,
];

/// Devnet seed node keypair seed (deterministic): `[0x01; 32]`
pub const DEVNET_SEED_KEYPAIR_SEED: &str =
    "0101010101010101010101010101010101010101010101010101010101010101";

/// Devnet validator node keypair seed (deterministic): `[0x02; 32]`
/// Passed via `--keypair-seed` on the validator server's service file.
#[allow(dead_code)] // Used externally by operators via --keypair-seed CLI flag
pub const DEVNET_VALIDATOR_KEYPAIR_SEED: &str =
    "0202020202020202020202020202020202020202020202020202020202020202";

/// Seed node public key (derived from `DEVNET_SEED_KEYPAIR_SEED`).
const DEVNET_SEED_PUBKEY: [u8; 32] = [
    0x8a, 0x88, 0xe3, 0xdd, 0x74, 0x09, 0xf1, 0x95, 0xfd, 0x52, 0xdb, 0x2d, 0x3c, 0xba, 0x5d, 0x72,
    0xca, 0x67, 0x09, 0xbf, 0x1d, 0x94, 0x12, 0x1b, 0xf3, 0x74, 0x88, 0x01, 0xb4, 0x0f, 0x6f, 0x5c,
];

/// Seed node address (derived from `DEVNET_SEED_PUBKEY`).
const DEVNET_SEED_ADDRESS: Address = [
    0x83, 0x56, 0x1a, 0xdb, 0x39, 0x8f, 0xd8, 0x7f, 0x8e, 0x7e, 0xd8, 0x33, 0x1b, 0xff, 0x2f, 0xcb,
    0x94, 0x57, 0x33, 0xcc,
];

/// Validator node public key (derived from `DEVNET_VALIDATOR_KEYPAIR_SEED`).
const DEVNET_VALIDATOR_PUBKEY: [u8; 32] = [
    0x81, 0x39, 0x77, 0x0e, 0xa8, 0x7d, 0x17, 0x5f, 0x56, 0xa3, 0x54, 0x66, 0xc3, 0x4c, 0x7e, 0xcc,
    0xcb, 0x8d, 0x8a, 0x91, 0xb4, 0xee, 0x37, 0xa2, 0x5d, 0xf6, 0x0f, 0x5b, 0x8f, 0xc9, 0xb3, 0x94,
];

/// Validator node address (derived from `DEVNET_VALIDATOR_PUBKEY`).
const DEVNET_VALIDATOR_ADDRESS: Address = [
    0x1e, 0xed, 0x29, 0xb1, 0x65, 0x4f, 0xbc, 0xa9, 0x46, 0x17, 0x00, 0x4d, 0x79, 0x69, 0xdf, 0xc4,
    0x65, 0x2b, 0x1f, 0x30,
];

/// Create a devnet genesis config with the augmnt founder pre-funded
/// and two deterministic validators (seed + validator node).
///
/// Returns `(genesis_config, founder_address)`.
pub fn devnet_genesis() -> (GenesisConfig, Address) {
    // Fixed timestamp so all --dev nodes produce the same genesis hash,
    // enabling state sync between peers on the devnet chain.
    let now: u64 = 1_771_286_400; // 2026-02-17T00:00:00Z

    let config = GenesisConfig {
        version: norn_types::genesis::GENESIS_CONFIG_VERSION,
        chain_id: "norn-dev".to_string(),
        timestamp: now,
        validators: vec![
            GenesisValidator {
                pubkey: DEVNET_SEED_PUBKEY,
                address: DEVNET_SEED_ADDRESS,
                stake: 1_000_000_000_000,
            },
            GenesisValidator {
                pubkey: DEVNET_VALIDATOR_PUBKEY,
                address: DEVNET_VALIDATOR_ADDRESS,
                stake: 1_000_000_000_000,
            },
        ],
        allocations: vec![GenesisAllocation {
            address: DEVNET_FOUNDER,
            token_id: NATIVE_TOKEN_ID,
            amount: 10_000_000_000_000_000_000, // 10M NORN (10^7 * 10^12 base units)
        }],
        parameters: GenesisParameters {
            block_time_target: 3,
            max_commitments_per_block: 10_000,
            commitment_finality_depth: 10,
            fraud_proof_window: 86_400,
            min_validator_stake: 1_000_000_000_000,
            initial_base_fee: 100,
            bonding_period: 100,
        },
        name_registrations: vec![GenesisNameRegistration {
            name: "augmnt".to_string(),
            owner: DEVNET_FOUNDER,
        }],
    };

    (config, DEVNET_FOUNDER)
}

/// Generate a genesis block from a config file and write it to output.
pub fn generate_genesis(config_path: &str, output_path: &str) -> Result<(), NodeError> {
    let config_str = std::fs::read_to_string(config_path).map_err(|e| NodeError::GenesisError {
        reason: format!("failed to read genesis config '{}': {}", config_path, e),
    })?;

    let config: GenesisConfig =
        serde_json::from_str(&config_str).map_err(|e| NodeError::GenesisError {
            reason: format!("failed to parse genesis config: {}", e),
        })?;

    let (block, state) = create_genesis_block(&config)?;

    let output = serde_json::json!({
        "config": config,
        "block": block,
        "state": state,
    });

    let json_str = serde_json::to_string_pretty(&output).map_err(|e| NodeError::GenesisError {
        reason: format!("failed to serialize genesis data: {}", e),
    })?;

    std::fs::write(output_path, json_str)?;

    Ok(())
}

/// Load genesis data from a previously saved genesis file.
pub fn load_genesis(path: &str) -> Result<(GenesisConfig, WeaveBlock, WeaveState), NodeError> {
    let contents = std::fs::read_to_string(path).map_err(|e| NodeError::GenesisError {
        reason: format!("failed to read genesis file '{}': {}", path, e),
    })?;

    let value: serde_json::Value =
        serde_json::from_str(&contents).map_err(|e| NodeError::GenesisError {
            reason: format!("failed to parse genesis file: {}", e),
        })?;

    let config: GenesisConfig =
        serde_json::from_value(value["config"].clone()).map_err(|e| NodeError::GenesisError {
            reason: format!("failed to parse genesis config: {}", e),
        })?;

    let block: WeaveBlock =
        serde_json::from_value(value["block"].clone()).map_err(|e| NodeError::GenesisError {
            reason: format!("failed to parse genesis block: {}", e),
        })?;

    let state: WeaveState =
        serde_json::from_value(value["state"].clone()).map_err(|e| NodeError::GenesisError {
            reason: format!("failed to parse genesis state: {}", e),
        })?;

    Ok((config, block, state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_types::genesis::{GenesisParameters, GenesisValidator};

    fn make_genesis_config() -> GenesisConfig {
        GenesisConfig {
            version: norn_types::genesis::GENESIS_CONFIG_VERSION,
            chain_id: "norn-testnet-0".to_string(),
            timestamp: 1700000000,
            validators: vec![GenesisValidator {
                pubkey: [1u8; 32],
                address: [1u8; 20],
                stake: 1_000_000_000_000,
            }],
            allocations: Vec::new(),
            parameters: GenesisParameters {
                block_time_target: 3,
                max_commitments_per_block: 10_000,
                commitment_finality_depth: 10,
                fraud_proof_window: 86_400,
                min_validator_stake: 1_000_000_000_000,
                initial_base_fee: 100,
                bonding_period: 100,
            },
            name_registrations: Vec::new(),
        }
    }

    #[test]
    fn test_create_genesis_block() {
        let config = make_genesis_config();
        let (block, state) = create_genesis_block(&config).unwrap();

        assert_eq!(block.height, 0);
        assert_eq!(block.prev_hash, [0u8; 32]);
        assert_ne!(block.hash, [0u8; 32]);
        assert_eq!(block.timestamp, config.timestamp);
        assert!(block.commitments.is_empty());
        assert!(block.registrations.is_empty());
        assert!(block.anchors.is_empty());
        assert!(block.fraud_proofs.is_empty());

        assert_eq!(state.height, 0);
        assert_eq!(state.latest_hash, block.hash);
        assert_eq!(state.fee_state.base_fee, 100);
        assert_eq!(state.fee_state.fee_multiplier, 1000);
    }

    #[test]
    fn test_genesis_hash_is_deterministic() {
        let config = make_genesis_config();
        let (block1, _) = create_genesis_block(&config).unwrap();
        let (block2, _) = create_genesis_block(&config).unwrap();
        assert_eq!(block1.hash, block2.hash);
    }

    #[test]
    fn test_devnet_genesis_deterministic() {
        let (config1, addr1) = devnet_genesis();
        let (config2, addr2) = devnet_genesis();
        // Fixed timestamp ensures identical genesis across all dev nodes.
        assert_eq!(config1.timestamp, config2.timestamp);
        assert_eq!(config1.chain_id, config2.chain_id);
        assert_eq!(config1.allocations, config2.allocations);
        assert_eq!(config1.parameters, config2.parameters);
        assert_eq!(config1.name_registrations, config2.name_registrations);
        assert_eq!(addr1, addr2);
        // Genesis hash must be identical across calls.
        let (block1, _) = create_genesis_block(&config1).unwrap();
        let (block2, _) = create_genesis_block(&config2).unwrap();
        assert_eq!(block1.hash, block2.hash);
    }

    #[test]
    fn test_devnet_genesis_allocation() {
        let (config, founder_addr) = devnet_genesis();
        assert_eq!(config.chain_id, "norn-dev");
        assert_eq!(config.allocations.len(), 1);
        assert_eq!(config.allocations[0].address, founder_addr);
        assert_eq!(config.allocations[0].token_id, NATIVE_TOKEN_ID);
        // 10M NORN = 10_000_000 * 10^12
        assert_eq!(config.allocations[0].amount, 10_000_000_000_000_000_000);
        // Founder address is non-zero
        assert_ne!(founder_addr, [0u8; 20]);
    }

    #[test]
    fn test_devnet_genesis_has_two_validators() {
        let (config, _) = devnet_genesis();
        assert_eq!(
            config.validators.len(),
            2,
            "devnet must have seed + validator"
        );
        // Seed node
        assert_eq!(config.validators[0].pubkey, DEVNET_SEED_PUBKEY);
        assert_eq!(config.validators[0].address, DEVNET_SEED_ADDRESS);
        // Validator node
        assert_eq!(config.validators[1].pubkey, DEVNET_VALIDATOR_PUBKEY);
        assert_eq!(config.validators[1].address, DEVNET_VALIDATOR_ADDRESS);
        // Both have the same stake
        assert_eq!(config.validators[0].stake, config.validators[1].stake);
    }

    #[test]
    fn test_devnet_genesis_block_has_valid_hash() {
        let (config, _) = devnet_genesis();
        let (block, _state) = create_genesis_block(&config).unwrap();
        assert_ne!(block.hash, [0u8; 32]);
        assert_eq!(block.height, 0);
        assert_eq!(block.prev_hash, [0u8; 32]);
    }

    #[test]
    fn test_genesis_roundtrip_via_file() {
        let tmp = tempfile::tempdir().unwrap();
        let config = make_genesis_config();

        // Write genesis config
        let config_path = tmp.path().join("genesis-config.json");
        let config_json = serde_json::to_string_pretty(&config).unwrap();
        std::fs::write(&config_path, config_json).unwrap();

        // Generate genesis
        let output_path = tmp.path().join("genesis.json");
        generate_genesis(config_path.to_str().unwrap(), output_path.to_str().unwrap()).unwrap();

        // Load genesis
        let (loaded_config, loaded_block, loaded_state) =
            load_genesis(output_path.to_str().unwrap()).unwrap();

        assert_eq!(loaded_config.chain_id, config.chain_id);
        assert_eq!(loaded_block.height, 0);
        assert_eq!(loaded_state.height, 0);
    }
}

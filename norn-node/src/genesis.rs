use norn_crypto::hash::blake3_hash;
use norn_types::genesis::GenesisConfig;
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
        fraud_proofs: Vec::new(),
        fraud_proofs_root: [0u8; 32],
        timestamp: config.timestamp,
        proposer: [0u8; 32],
        validator_signatures: Vec::new(),
    };

    // Compute the genesis block hash from its fields.
    block.hash = compute_genesis_hash(&block, &config.chain_id);

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

/// Compute a deterministic hash for the genesis block, incorporating the chain_id.
fn compute_genesis_hash(block: &WeaveBlock, chain_id: &str) -> [u8; 32] {
    let mut data = Vec::new();
    data.extend_from_slice(chain_id.as_bytes());
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
            },
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

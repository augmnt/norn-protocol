use std::collections::HashMap;

use norn_crypto::hash::blake3_hash;
use norn_types::loom::{Loom, LoomBytecode, LoomConfig, LoomStateTransition, Participant};
use norn_types::primitives::*;

use crate::error::LoomError;
use crate::gas::DEFAULT_GAS_LIMIT;
use crate::host::LoomHostState;
use crate::runtime::LoomRuntime;
use crate::state::LoomState;

/// Manages the lifecycle of looms: deployment, participant management,
/// execution, and state anchoring.
pub struct LoomManager {
    /// Registered looms keyed by their LoomId.
    looms: HashMap<LoomId, Loom>,
    /// Deployed bytecodes keyed by LoomId.
    bytecodes: HashMap<LoomId, LoomBytecode>,
    /// Per-loom key-value state.
    states: HashMap<LoomId, LoomState>,
}

impl LoomManager {
    /// Create a new, empty loom manager.
    pub fn new() -> Self {
        Self {
            looms: HashMap::new(),
            bytecodes: HashMap::new(),
            states: HashMap::new(),
        }
    }

    /// Deploy a new loom with the given configuration and bytecode.
    ///
    /// Returns the loom ID on success.
    pub fn deploy(
        &mut self,
        config: LoomConfig,
        operator: PublicKey,
        bytecode: Vec<u8>,
        timestamp: Timestamp,
    ) -> Result<LoomId, LoomError> {
        let loom_id = config.loom_id;

        // Validate bytecode is not empty.
        if bytecode.is_empty() {
            return Err(LoomError::InvalidBytecode {
                reason: "bytecode cannot be empty".to_string(),
            });
        }

        let wasm_hash = blake3_hash(&bytecode);

        let loom_bytecode = LoomBytecode {
            loom_id,
            wasm_hash,
            bytecode,
        };

        let initial_state = LoomState::new(loom_id);
        let state_hash = initial_state.compute_hash();

        let loom = Loom {
            config,
            operator,
            participants: Vec::new(),
            state_hash,
            version: 0,
            active: true,
            last_updated: timestamp,
        };

        self.looms.insert(loom_id, loom);
        self.bytecodes.insert(loom_id, loom_bytecode);
        self.states.insert(loom_id, initial_state);

        Ok(loom_id)
    }

    /// Add a participant to a loom.
    pub fn join(
        &mut self,
        loom_id: &LoomId,
        pubkey: PublicKey,
        address: Address,
        timestamp: Timestamp,
    ) -> Result<(), LoomError> {
        let loom = self
            .looms
            .get_mut(loom_id)
            .ok_or(LoomError::LoomNotFound { loom_id: *loom_id })?;

        // Check participant limit.
        let active_count = loom.participants.iter().filter(|p| p.active).count();
        if active_count >= loom.config.max_participants {
            return Err(LoomError::ParticipantLimitExceeded {
                count: active_count + 1,
                max: loom.config.max_participants,
            });
        }

        // Check for duplicate address.
        if let Some(existing) = loom.participants.iter_mut().find(|p| p.address == address) {
            // Reactivate if previously left.
            if !existing.active {
                existing.active = true;
                existing.joined_at = timestamp;
                return Ok(());
            }
            // Already active -- no-op.
            return Ok(());
        }

        loom.participants.push(Participant {
            pubkey,
            address,
            joined_at: timestamp,
            active: true,
        });

        Ok(())
    }

    /// Remove (deactivate) a participant from a loom.
    pub fn leave(&mut self, loom_id: &LoomId, address: &Address) -> Result<(), LoomError> {
        let loom = self
            .looms
            .get_mut(loom_id)
            .ok_or(LoomError::LoomNotFound { loom_id: *loom_id })?;

        let participant = loom
            .participants
            .iter_mut()
            .find(|p| p.address == *address && p.active)
            .ok_or(LoomError::NotParticipant { address: *address })?;

        participant.active = false;
        Ok(())
    }

    /// Execute a transaction against a loom contract.
    ///
    /// Runs the Wasm bytecode with the given input and returns a
    /// `LoomStateTransition` capturing the before/after state hashes.
    pub fn execute(
        &mut self,
        loom_id: &LoomId,
        input: &[u8],
        sender: Address,
        block_height: u64,
        timestamp: u64,
    ) -> Result<LoomStateTransition, LoomError> {
        // Validate loom exists.
        let loom = self
            .looms
            .get(loom_id)
            .ok_or(LoomError::LoomNotFound { loom_id: *loom_id })?;

        // Validate sender is a participant.
        let is_participant = loom
            .participants
            .iter()
            .any(|p| p.address == sender && p.active);
        if !is_participant {
            return Err(LoomError::NotParticipant { address: sender });
        }

        // Get current state.
        let state = self
            .states
            .get(loom_id)
            .ok_or(LoomError::LoomNotFound { loom_id: *loom_id })?;
        let prev_state_hash = state.compute_hash();

        // Set up host state with the loom's current data.
        let mut host_state = LoomHostState::new(sender, block_height, timestamp, DEFAULT_GAS_LIMIT);
        host_state.state = state.data.clone();

        // Get bytecode.
        let bytecode_entry = self
            .bytecodes
            .get(loom_id)
            .ok_or(LoomError::LoomNotFound { loom_id: *loom_id })?;

        // Instantiate and execute.
        let runtime = LoomRuntime::new()?;
        let mut instance = runtime.instantiate(&bytecode_entry.bytecode, host_state)?;
        let outputs = instance.call_execute(input)?;

        // Extract updated state from the host.
        let host_state = instance.into_host_state();

        // Update the loom's stored state.
        let loom_state = self
            .states
            .get_mut(loom_id)
            .ok_or(LoomError::LoomNotFound { loom_id: *loom_id })?;
        loom_state.data = host_state.state;
        let new_state_hash = loom_state.compute_hash();

        // Update loom metadata.
        let loom = self
            .looms
            .get_mut(loom_id)
            .ok_or(LoomError::LoomNotFound { loom_id: *loom_id })?;
        loom.state_hash = new_state_hash;
        loom.version += 1;
        loom.last_updated = timestamp;

        Ok(LoomStateTransition {
            loom_id: *loom_id,
            prev_state_hash,
            new_state_hash,
            inputs: input.to_vec(),
            outputs,
        })
    }

    /// Return the current state hash and version for a loom (anchoring).
    pub fn anchor(&self, loom_id: &LoomId) -> Result<(Hash, Version), LoomError> {
        let loom = self
            .looms
            .get(loom_id)
            .ok_or(LoomError::LoomNotFound { loom_id: *loom_id })?;
        Ok((loom.state_hash, loom.version))
    }

    /// Get a reference to a loom by ID.
    pub fn get_loom(&self, loom_id: &LoomId) -> Option<&Loom> {
        self.looms.get(loom_id)
    }

    /// Get a reference to a loom's bytecode.
    pub fn get_bytecode(&self, loom_id: &LoomId) -> Option<&LoomBytecode> {
        self.bytecodes.get(loom_id)
    }

    /// Get a reference to a loom's state.
    pub fn get_state(&self, loom_id: &LoomId) -> Option<&LoomState> {
        self.states.get(loom_id)
    }

    /// Restore a previously persisted loom (used during state rebuild).
    pub fn restore_loom(
        &mut self,
        loom_id: LoomId,
        loom: Loom,
        bytecode: LoomBytecode,
        state_data: HashMap<Vec<u8>, Vec<u8>>,
    ) {
        let mut state = LoomState::new(loom_id);
        state.data = state_data;
        self.looms.insert(loom_id, loom);
        self.bytecodes.insert(loom_id, bytecode);
        self.states.insert(loom_id, state);
    }

    /// List all deployed looms.
    pub fn list_looms(&self) -> Vec<(&LoomId, &Loom)> {
        self.looms.iter().collect()
    }

    /// Query a loom contract (read-only, no state changes).
    pub fn query(
        &self,
        loom_id: &LoomId,
        input: &[u8],
        sender: Address,
        block_height: u64,
        timestamp: u64,
    ) -> Result<Vec<u8>, LoomError> {
        // Validate loom exists.
        let _loom = self
            .looms
            .get(loom_id)
            .ok_or(LoomError::LoomNotFound { loom_id: *loom_id })?;

        // Get current state.
        let state = self
            .states
            .get(loom_id)
            .ok_or(LoomError::LoomNotFound { loom_id: *loom_id })?;

        // Set up host state with the loom's current data.
        let mut host_state = LoomHostState::new(sender, block_height, timestamp, DEFAULT_GAS_LIMIT);
        host_state.state = state.data.clone();

        // Get bytecode.
        let bytecode_entry = self
            .bytecodes
            .get(loom_id)
            .ok_or(LoomError::LoomNotFound { loom_id: *loom_id })?;

        // Instantiate and execute (read-only — state is discarded).
        let runtime = LoomRuntime::new()?;
        let mut instance = runtime.instantiate(&bytecode_entry.bytecode, host_state)?;
        let outputs = instance.call_execute(input)?;

        Ok(outputs)
    }
}

impl Default for LoomManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(loom_id: LoomId) -> LoomConfig {
        LoomConfig {
            loom_id,
            name: "test-loom".to_string(),
            max_participants: 10,
            min_participants: 1,
            accepted_tokens: vec![NATIVE_TOKEN_ID],
            config_data: vec![],
        }
    }

    fn simple_wasm() -> Vec<u8> {
        let wat = r#"
            (module
                (func (export "execute") (param i32 i32) (result i32)
                    i32.const 42
                )
            )
        "#;
        wat::parse_str(wat).expect("failed to compile WAT")
    }

    #[test]
    fn test_deploy() {
        let mut manager = LoomManager::new();
        let loom_id = [1u8; 32];
        let config = test_config(loom_id);
        let operator = [2u8; 32];

        let result = manager.deploy(config, operator, simple_wasm(), 1000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), loom_id);

        let loom = manager.get_loom(&loom_id).unwrap();
        assert!(loom.active);
        assert_eq!(loom.version, 0);
        assert_eq!(loom.operator, operator);
    }

    #[test]
    fn test_deploy_empty_bytecode() {
        let mut manager = LoomManager::new();
        let loom_id = [1u8; 32];
        let config = test_config(loom_id);
        let result = manager.deploy(config, [2u8; 32], vec![], 1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_join_and_leave() {
        let mut manager = LoomManager::new();
        let loom_id = [1u8; 32];
        let config = test_config(loom_id);
        manager
            .deploy(config, [2u8; 32], simple_wasm(), 1000)
            .unwrap();

        let address = [3u8; 20];
        let pubkey = [3u8; 32];

        // Join.
        manager.join(&loom_id, pubkey, address, 1001).unwrap();
        let loom = manager.get_loom(&loom_id).unwrap();
        assert_eq!(loom.participants.len(), 1);
        assert!(loom.participants[0].active);

        // Leave.
        manager.leave(&loom_id, &address).unwrap();
        let loom = manager.get_loom(&loom_id).unwrap();
        assert!(!loom.participants[0].active);
    }

    #[test]
    fn test_participant_limit() {
        let mut manager = LoomManager::new();
        let loom_id = [1u8; 32];
        let mut config = test_config(loom_id);
        config.max_participants = 2;
        manager
            .deploy(config, [0u8; 32], simple_wasm(), 1000)
            .unwrap();

        manager.join(&loom_id, [1u8; 32], [1u8; 20], 1001).unwrap();
        manager.join(&loom_id, [2u8; 32], [2u8; 20], 1002).unwrap();

        // Third participant should fail.
        let result = manager.join(&loom_id, [3u8; 32], [3u8; 20], 1003);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute() {
        let mut manager = LoomManager::new();
        let loom_id = [1u8; 32];
        let config = test_config(loom_id);
        manager
            .deploy(config, [2u8; 32], simple_wasm(), 1000)
            .unwrap();

        let sender = [3u8; 20];
        manager.join(&loom_id, [3u8; 32], sender, 1001).unwrap();

        let transition = manager.execute(&loom_id, &[], sender, 100, 1002).unwrap();
        assert_eq!(transition.loom_id, loom_id);
        assert_eq!(transition.outputs, 42i32.to_le_bytes().to_vec());
    }

    #[test]
    fn test_execute_non_participant() {
        let mut manager = LoomManager::new();
        let loom_id = [1u8; 32];
        let config = test_config(loom_id);
        manager
            .deploy(config, [2u8; 32], simple_wasm(), 1000)
            .unwrap();

        let outsider = [99u8; 20];
        let result = manager.execute(&loom_id, &[], outsider, 100, 1002);
        assert!(result.is_err());
    }

    #[test]
    fn test_anchor() {
        let mut manager = LoomManager::new();
        let loom_id = [1u8; 32];
        let config = test_config(loom_id);
        manager
            .deploy(config, [2u8; 32], simple_wasm(), 1000)
            .unwrap();

        let (hash, version) = manager.anchor(&loom_id).unwrap();
        assert_eq!(version, 0);
        // Hash should be the hash of the empty state.
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_full_lifecycle() {
        let mut manager = LoomManager::new();
        let loom_id = [1u8; 32];
        let config = test_config(loom_id);

        // Deploy.
        manager
            .deploy(config, [0u8; 32], simple_wasm(), 1000)
            .unwrap();

        // Join.
        let addr_a = [10u8; 20];
        let addr_b = [20u8; 20];
        manager.join(&loom_id, [10u8; 32], addr_a, 1001).unwrap();
        manager.join(&loom_id, [20u8; 32], addr_b, 1002).unwrap();

        // Execute.
        let transition = manager.execute(&loom_id, &[], addr_a, 50, 1003).unwrap();
        assert_eq!(transition.loom_id, loom_id);

        // Anchor.
        let (hash, version) = manager.anchor(&loom_id).unwrap();
        assert_eq!(version, 1);
        assert_eq!(hash, manager.get_loom(&loom_id).unwrap().state_hash);

        // Leave.
        manager.leave(&loom_id, &addr_a).unwrap();
        let loom = manager.get_loom(&loom_id).unwrap();
        assert!(!loom.participants[0].active);
        assert!(loom.participants[1].active);
    }
}

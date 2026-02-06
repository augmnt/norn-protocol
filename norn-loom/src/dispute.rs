use std::collections::HashMap;

use norn_types::loom::{LoomBytecode, LoomStateTransition};
use norn_types::primitives::Address;

use crate::error::LoomError;
use crate::gas::DEFAULT_GAS_LIMIT;
use crate::host::LoomHostState;
use crate::runtime::LoomRuntime;
use crate::state::LoomState;

/// The result of challenging a loom state transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisputeResult {
    /// The transition is valid -- re-execution produced the same state hash.
    Valid,
    /// The transition is invalid -- re-execution produced a different state hash.
    Invalid { reason: String },
}

/// Challenge a loom state transition by deterministically re-executing it.
///
/// The transition is re-executed from the provided `initial_state` using the
/// same bytecode, sender, and block context. If the resulting state hash
/// matches `transition.new_state_hash`, the transition is deemed valid;
/// otherwise it is invalid (a fraud).
pub fn challenge_transition(
    transition: &LoomStateTransition,
    bytecode: &LoomBytecode,
    initial_state: &HashMap<Vec<u8>, Vec<u8>>,
    sender: Address,
    block_height: u64,
    timestamp: u64,
) -> Result<DisputeResult, LoomError> {
    // Verify the initial state hash matches what the transition claims.
    let mut pre_state = LoomState::new(transition.loom_id);
    pre_state.data = initial_state.clone();
    let pre_hash = pre_state.compute_hash();

    if pre_hash != transition.prev_state_hash {
        return Ok(DisputeResult::Invalid {
            reason: format!(
                "initial state hash mismatch: computed {:?} but transition claims {:?}",
                pre_hash, transition.prev_state_hash
            ),
        });
    }

    // Set up the host state with the initial data.
    let mut host_state = LoomHostState::new(sender, block_height, timestamp, DEFAULT_GAS_LIMIT);
    host_state.state = initial_state.clone();

    // Instantiate and execute.
    let runtime = LoomRuntime::new()?;
    let mut instance = runtime.instantiate(&bytecode.bytecode, host_state)?;
    let _outputs = instance.call_execute(&transition.inputs)?;

    // Extract the post-execution state and compute its hash.
    let host_state = instance.into_host_state();
    let mut post_state = LoomState::new(transition.loom_id);
    post_state.data = host_state.state;
    let post_hash = post_state.compute_hash();

    // Compare against the claimed new state hash.
    if post_hash != transition.new_state_hash {
        Ok(DisputeResult::Invalid {
            reason: format!(
                "post-execution state hash mismatch: computed {:?} but transition claims {:?}",
                post_hash, transition.new_state_hash
            ),
        })
    } else {
        Ok(DisputeResult::Valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_crypto::hash::blake3_hash;

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

    fn make_bytecode(loom_id: [u8; 32], wasm: &[u8]) -> LoomBytecode {
        LoomBytecode {
            loom_id,
            wasm_hash: blake3_hash(wasm),
            bytecode: wasm.to_vec(),
        }
    }

    /// Helper: execute the wasm against the initial state and return the actual
    /// new state hash.
    fn compute_actual_new_hash(
        bytecode: &LoomBytecode,
        initial_state: &HashMap<Vec<u8>, Vec<u8>>,
        input: &[u8],
        sender: Address,
        block_height: u64,
        timestamp: u64,
        loom_id: [u8; 32],
    ) -> norn_types::primitives::Hash {
        let mut host_state = LoomHostState::new(sender, block_height, timestamp, DEFAULT_GAS_LIMIT);
        host_state.state = initial_state.clone();
        let runtime = LoomRuntime::new().unwrap();
        let mut instance = runtime.instantiate(&bytecode.bytecode, host_state).unwrap();
        let _outputs = instance.call_execute(input).unwrap();
        let host_state = instance.into_host_state();
        let mut post_state = LoomState::new(loom_id);
        post_state.data = host_state.state;
        post_state.compute_hash()
    }

    #[test]
    fn test_valid_transition() {
        let loom_id = [1u8; 32];
        let wasm = simple_wasm();
        let bytecode = make_bytecode(loom_id, &wasm);
        let initial_state: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        let sender = [3u8; 20];
        let block_height = 100;
        let timestamp = 1000;

        // Compute what the actual transition would produce.
        let mut pre_state = LoomState::new(loom_id);
        pre_state.data = initial_state.clone();
        let prev_hash = pre_state.compute_hash();

        let new_hash = compute_actual_new_hash(
            &bytecode,
            &initial_state,
            &[],
            sender,
            block_height,
            timestamp,
            loom_id,
        );

        let transition = LoomStateTransition {
            loom_id,
            prev_state_hash: prev_hash,
            new_state_hash: new_hash,
            inputs: vec![],
            outputs: 42i32.to_le_bytes().to_vec(),
        };

        let result = challenge_transition(
            &transition,
            &bytecode,
            &initial_state,
            sender,
            block_height,
            timestamp,
        )
        .unwrap();
        assert_eq!(result, DisputeResult::Valid);
    }

    #[test]
    fn test_invalid_transition_wrong_hash() {
        let loom_id = [1u8; 32];
        let wasm = simple_wasm();
        let bytecode = make_bytecode(loom_id, &wasm);
        let initial_state: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        let sender = [3u8; 20];
        let block_height = 100;
        let timestamp = 1000;

        let mut pre_state = LoomState::new(loom_id);
        pre_state.data = initial_state.clone();
        let prev_hash = pre_state.compute_hash();

        // Deliberately provide a wrong new_state_hash.
        let transition = LoomStateTransition {
            loom_id,
            prev_state_hash: prev_hash,
            new_state_hash: [0xFFu8; 32], // Wrong hash!
            inputs: vec![],
            outputs: 42i32.to_le_bytes().to_vec(),
        };

        let result = challenge_transition(
            &transition,
            &bytecode,
            &initial_state,
            sender,
            block_height,
            timestamp,
        )
        .unwrap();
        assert!(matches!(result, DisputeResult::Invalid { .. }));
    }

    #[test]
    fn test_invalid_pre_state_hash() {
        let loom_id = [1u8; 32];
        let wasm = simple_wasm();
        let bytecode = make_bytecode(loom_id, &wasm);
        let initial_state: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        let sender = [3u8; 20];

        // Transition claims a different prev_state_hash than what the initial state produces.
        let transition = LoomStateTransition {
            loom_id,
            prev_state_hash: [0xAAu8; 32], // Wrong prev hash.
            new_state_hash: [0xBBu8; 32],
            inputs: vec![],
            outputs: vec![],
        };

        let result =
            challenge_transition(&transition, &bytecode, &initial_state, sender, 100, 1000)
                .unwrap();
        assert!(matches!(result, DisputeResult::Invalid { .. }));
    }
}

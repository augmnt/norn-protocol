use std::collections::HashSet;

use norn_crypto::address::pubkey_to_address;
use norn_crypto::hash::blake3_hash;
use norn_crypto::keys::verify;
use norn_crypto::merkle::SparseMerkleTree;
use norn_types::primitives::*;
use norn_types::weave::{Registration, WeaveState};

use crate::error::WeaveError;

/// Validate a thread registration.
///
/// Checks: signature, no duplicate thread_id, and thread_id matches pubkey_to_address(owner).
pub fn validate_registration(
    registration: &Registration,
    existing_threads: &HashSet<ThreadId>,
) -> Result<(), WeaveError> {
    // Verify signature.
    let sig_data = registration_signing_data(registration);
    verify(&sig_data, &registration.signature, &registration.owner).map_err(|_| {
        WeaveError::InvalidRegistration {
            reason: "invalid signature".to_string(),
        }
    })?;

    // Check no duplicate thread_id.
    if existing_threads.contains(&registration.thread_id) {
        return Err(WeaveError::DuplicateThread {
            thread_id: registration.thread_id,
        });
    }

    // Verify thread_id matches pubkey_to_address(owner).
    let expected_address = pubkey_to_address(&registration.owner);
    if registration.thread_id != expected_address {
        return Err(WeaveError::InvalidRegistration {
            reason: "thread_id does not match pubkey_to_address(owner)".to_string(),
        });
    }

    Ok(())
}

/// Apply a validated registration to the global weave state.
///
/// Inserts into the threads Merkle tree and increments thread_count.
pub fn apply_registration(
    state: &mut WeaveState,
    merkle_tree: &mut SparseMerkleTree,
    registration: &Registration,
) -> Result<(), WeaveError> {
    let key = blake3_hash(&registration.thread_id);

    let value = borsh::to_vec(&(registration.initial_state_hash, 0u64)).map_err(|e| {
        WeaveError::InvalidRegistration {
            reason: format!("serialization error: {}", e),
        }
    })?;

    merkle_tree.insert(key, value);
    state.threads_root = merkle_tree.root();
    state.thread_count += 1;

    Ok(())
}

/// Compute the data that should be signed for a registration.
fn registration_signing_data(registration: &Registration) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&registration.thread_id);
    data.extend_from_slice(&registration.owner);
    data.extend_from_slice(&registration.initial_state_hash);
    data.extend_from_slice(&registration.timestamp.to_le_bytes());
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_crypto::keys::Keypair;
    use norn_types::weave::FeeState;

    fn make_signed_registration(kp: &Keypair) -> Registration {
        let thread_id = pubkey_to_address(&kp.public_key());
        let mut reg = Registration {
            thread_id,
            owner: kp.public_key(),
            initial_state_hash: [1u8; 32],
            timestamp: 1000,
            signature: [0u8; 64],
        };
        let sig_data = registration_signing_data(&reg);
        reg.signature = kp.sign(&sig_data);
        reg
    }

    fn make_weave_state() -> WeaveState {
        WeaveState {
            height: 0,
            latest_hash: [0u8; 32],
            threads_root: [0u8; 32],
            thread_count: 0,
            fee_state: FeeState {
                base_fee: 100,
                fee_multiplier: 1000,
                epoch_fees: 0,
            },
        }
    }

    #[test]
    fn test_valid_registration() {
        let kp = Keypair::generate();
        let reg = make_signed_registration(&kp);
        let existing = HashSet::new();
        assert!(validate_registration(&reg, &existing).is_ok());
    }

    #[test]
    fn test_duplicate_thread_rejected() {
        let kp = Keypair::generate();
        let reg = make_signed_registration(&kp);
        let mut existing = HashSet::new();
        existing.insert(reg.thread_id);
        assert!(validate_registration(&reg, &existing).is_err());
    }

    #[test]
    fn test_invalid_thread_id() {
        let kp = Keypair::generate();
        let mut reg = make_signed_registration(&kp);
        reg.thread_id = [0u8; 20]; // Wrong thread_id.
                                   // Re-sign with wrong thread_id.
        let sig_data = registration_signing_data(&reg);
        reg.signature = kp.sign(&sig_data);
        let existing = HashSet::new();
        assert!(validate_registration(&reg, &existing).is_err());
    }

    #[test]
    fn test_invalid_signature() {
        let kp = Keypair::generate();
        let mut reg = make_signed_registration(&kp);
        reg.signature[0] ^= 0xff;
        let existing = HashSet::new();
        assert!(validate_registration(&reg, &existing).is_err());
    }

    #[test]
    fn test_apply_registration() {
        let kp = Keypair::generate();
        let reg = make_signed_registration(&kp);
        let mut state = make_weave_state();
        let mut tree = SparseMerkleTree::new();

        apply_registration(&mut state, &mut tree, &reg).unwrap();
        assert_eq!(state.thread_count, 1);
        assert_ne!(state.threads_root, [0u8; 32]);
    }
}

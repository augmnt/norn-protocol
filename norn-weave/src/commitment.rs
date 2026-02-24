use norn_crypto::address::pubkey_to_address;
use norn_crypto::hash::blake3_hash;
use norn_crypto::keys::verify;
use norn_crypto::merkle::SparseMerkleTree;
use norn_types::constants::{MAX_COMMITMENT_AGE, MAX_TIMESTAMP_DRIFT};
use norn_types::primitives::*;
use norn_types::weave::{CommitmentUpdate, WeaveState};

use crate::error::WeaveError;

/// Validate a commitment update.
///
/// Checks: signature validity, version monotonicity, and staleness.
pub fn validate_commitment(
    commitment: &CommitmentUpdate,
    current_version: Option<Version>,
    current_time: Timestamp,
) -> Result<(), WeaveError> {
    // Verify signature over all fields except the signature itself.
    let sig_data = commitment_signing_data(commitment);
    verify(&sig_data, &commitment.signature, &commitment.owner).map_err(|_| {
        WeaveError::InvalidCommitment {
            reason: "invalid signature".to_string(),
        }
    })?;

    // Verify the owner pubkey actually derives the claimed thread_id.
    let expected_address = pubkey_to_address(&commitment.owner);
    if commitment.thread_id != expected_address {
        return Err(WeaveError::InvalidCommitment {
            reason: "owner pubkey does not derive thread_id".to_string(),
        });
    }

    // Check version monotonicity.
    if let Some(cv) = current_version {
        if commitment.version <= cv {
            return Err(WeaveError::InvalidCommitment {
                reason: format!(
                    "version not monotonically increasing: {} <= {}",
                    commitment.version, cv
                ),
            });
        }
    }

    // Reject future-dated commitments.
    if commitment.timestamp > current_time + MAX_TIMESTAMP_DRIFT {
        return Err(WeaveError::InvalidCommitment {
            reason: "commitment timestamp too far in the future".to_string(),
        });
    }

    // Staleness check.
    if current_time > commitment.timestamp {
        let age = current_time - commitment.timestamp;
        if age > MAX_COMMITMENT_AGE {
            return Err(WeaveError::StaleCommitment {
                age,
                max_age: MAX_COMMITMENT_AGE,
            });
        }
    }

    Ok(())
}

/// Apply a validated commitment to the global weave state.
///
/// Updates the threads Merkle tree and the threads_root in state.
pub fn apply_commitment(
    state: &mut WeaveState,
    merkle_tree: &mut SparseMerkleTree,
    commitment: &CommitmentUpdate,
) -> Result<(), WeaveError> {
    // Build the key: hash of thread_id.
    let key = blake3_hash(&commitment.thread_id);

    // Build the value: borsh(state_hash, version).
    let value = borsh::to_vec(&(commitment.state_hash, commitment.version)).map_err(|e| {
        WeaveError::InvalidCommitment {
            reason: format!("serialization error: {}", e),
        }
    })?;

    merkle_tree.insert(key, value);
    state.threads_root = merkle_tree.root();

    Ok(())
}

/// Compute the data that should be signed for a commitment.
fn commitment_signing_data(commitment: &CommitmentUpdate) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&commitment.thread_id);
    data.extend_from_slice(&commitment.owner);
    data.extend_from_slice(&commitment.version.to_le_bytes());
    data.extend_from_slice(&commitment.state_hash);
    data.extend_from_slice(&commitment.prev_commitment_hash);
    data.extend_from_slice(&commitment.knot_count.to_le_bytes());
    data.extend_from_slice(&commitment.timestamp.to_le_bytes());
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_crypto::address::pubkey_to_address;
    use norn_crypto::keys::Keypair;
    use norn_types::weave::FeeState;

    fn make_signed_commitment(
        kp: &Keypair,
        version: Version,
        timestamp: Timestamp,
    ) -> CommitmentUpdate {
        let thread_id = pubkey_to_address(&kp.public_key());
        let mut c = CommitmentUpdate {
            thread_id,
            owner: kp.public_key(),
            version,
            state_hash: [1u8; 32],
            prev_commitment_hash: [0u8; 32],
            knot_count: 1,
            timestamp,
            signature: [0u8; 64],
        };
        let sig_data = commitment_signing_data(&c);
        c.signature = kp.sign(&sig_data);
        c
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
    fn test_valid_commitment() {
        let kp = Keypair::generate();
        let c = make_signed_commitment(&kp, 1, 1000);
        assert!(validate_commitment(&c, None, 1000).is_ok());
    }

    #[test]
    fn test_invalid_signature() {
        let kp = Keypair::generate();
        let mut c = make_signed_commitment(&kp, 1, 1000);
        c.signature[0] ^= 0xff;
        assert!(validate_commitment(&c, None, 1000).is_err());
    }

    #[test]
    fn test_version_monotonicity() {
        let kp = Keypair::generate();
        let c = make_signed_commitment(&kp, 1, 1000);
        // Current version 1, commitment version 1 => not strictly increasing.
        assert!(validate_commitment(&c, Some(1), 1000).is_err());
        // Current version 0, commitment version 1 => ok.
        assert!(validate_commitment(&c, Some(0), 1000).is_ok());
    }

    #[test]
    fn test_staleness() {
        let kp = Keypair::generate();
        let c = make_signed_commitment(&kp, 1, 1000);
        // Current time far in the future.
        let stale_time = 1000 + MAX_COMMITMENT_AGE + 1;
        assert!(validate_commitment(&c, None, stale_time).is_err());
    }

    #[test]
    fn test_thread_id_mismatch() {
        let kp_a = Keypair::generate();
        let kp_b = Keypair::generate();
        // Sign with key A but set thread_id to key B's address.
        let wrong_thread_id = pubkey_to_address(&kp_b.public_key());
        let mut c = CommitmentUpdate {
            thread_id: wrong_thread_id,
            owner: kp_a.public_key(),
            version: 1,
            state_hash: [1u8; 32],
            prev_commitment_hash: [0u8; 32],
            knot_count: 1,
            timestamp: 1000,
            signature: [0u8; 64],
        };
        let sig_data = commitment_signing_data(&c);
        c.signature = kp_a.sign(&sig_data);
        let result = validate_commitment(&c, None, 1000);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("owner pubkey does not derive thread_id"),
            "expected thread_id mismatch error, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_apply_commitment() {
        let kp = Keypair::generate();
        let c = make_signed_commitment(&kp, 1, 1000);
        let mut state = make_weave_state();
        let mut tree = SparseMerkleTree::new();

        apply_commitment(&mut state, &mut tree, &c).unwrap();
        assert_ne!(state.threads_root, [0u8; 32]);
    }
}

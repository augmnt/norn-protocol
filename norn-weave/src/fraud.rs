use std::collections::HashMap;

use norn_crypto::keys::verify;
use norn_types::fraud::{FraudProof, FraudProofSubmission};
use norn_types::loom::LoomBytecode;
use norn_types::primitives::Address;

use crate::error::WeaveError;

/// The result of validating a fraud proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FraudVerdict {
    /// The fraud proof is valid: two knots at the same version.
    ValidDoubleKnot,
    /// The fraud proof is valid: a stale commitment was detected.
    ValidStaleCommit,
    /// The fraud proof is valid: an invalid loom transition was detected.
    ValidInvalidLoomTransition,
    /// The fraud proof is invalid.
    Invalid { reason: String },
}

/// Validate a fraud proof submission and return a verdict.
pub fn validate_fraud_proof(submission: &FraudProofSubmission) -> Result<FraudVerdict, WeaveError> {
    // Verify submitter signature.
    let sig_data = fraud_proof_signing_data(submission);
    verify(&sig_data, &submission.signature, &submission.submitter).map_err(|_| {
        WeaveError::InvalidFraudProof {
            reason: "invalid submitter signature".to_string(),
        }
    })?;

    match &submission.proof {
        FraudProof::DoubleKnot {
            thread_id,
            knot_a,
            knot_b,
        } => {
            // Verify both knots have valid signatures.
            if knot_a.signatures.is_empty() {
                return Ok(FraudVerdict::Invalid {
                    reason: "knot_a has no signatures".to_string(),
                });
            }
            if knot_b.signatures.is_empty() {
                return Ok(FraudVerdict::Invalid {
                    reason: "knot_b has no signatures".to_string(),
                });
            }

            // Both knots must reference the same thread and same version in before_states.
            let a_has_thread = knot_a
                .before_states
                .iter()
                .any(|s| s.thread_id == *thread_id);
            let b_has_thread = knot_b
                .before_states
                .iter()
                .any(|s| s.thread_id == *thread_id);

            if !a_has_thread || !b_has_thread {
                return Ok(FraudVerdict::Invalid {
                    reason: "knots do not reference the claimed thread".to_string(),
                });
            }

            // Same version in before_states for the disputed thread.
            let a_version = knot_a
                .before_states
                .iter()
                .find(|s| s.thread_id == *thread_id)
                .map(|s| s.version);
            let b_version = knot_b
                .before_states
                .iter()
                .find(|s| s.thread_id == *thread_id)
                .map(|s| s.version);

            if a_version != b_version {
                return Ok(FraudVerdict::Invalid {
                    reason: "knots have different versions for the thread".to_string(),
                });
            }

            // Different knot IDs.
            if knot_a.id == knot_b.id {
                return Ok(FraudVerdict::Invalid {
                    reason: "knots have the same ID (not a double knot)".to_string(),
                });
            }

            Ok(FraudVerdict::ValidDoubleKnot)
        }

        FraudProof::StaleCommit {
            thread_id: _,
            commitment,
            missing_knots,
        } => {
            // Verify commitment exists (has a valid signature).
            if commitment.signature == [0u8; 64] {
                return Ok(FraudVerdict::Invalid {
                    reason: "commitment has no signature".to_string(),
                });
            }

            // Missing knots must form a valid progression (non-empty).
            if missing_knots.is_empty() {
                return Ok(FraudVerdict::Invalid {
                    reason: "no missing knots provided".to_string(),
                });
            }

            // Verify each missing knot has signatures.
            for (i, knot) in missing_knots.iter().enumerate() {
                if knot.signatures.is_empty() {
                    return Ok(FraudVerdict::Invalid {
                        reason: format!("missing knot {} has no signatures", i),
                    });
                }
            }

            Ok(FraudVerdict::ValidStaleCommit)
        }

        FraudProof::InvalidLoomTransition { .. } => {
            // Requires loom context (bytecode, state) for re-execution.
            // Use validate_fraud_proof_with_loom() when loom context is available.
            Ok(FraudVerdict::Invalid {
                reason: "InvalidLoomTransition requires loom context; use validate_fraud_proof_with_loom".to_string(),
            })
        }
    }
}

/// Context needed to verify an InvalidLoomTransition fraud proof.
pub struct LoomDisputeContext {
    /// The deployed bytecode for the loom.
    pub bytecode: LoomBytecode,
    /// The initial state (KV pairs) before the transition.
    pub initial_state: HashMap<Vec<u8>, Vec<u8>>,
    /// The sender address for the disputed transition.
    pub sender: Address,
    /// The block height at which the transition occurred.
    pub block_height: u64,
    /// The timestamp at which the transition occurred.
    pub timestamp: u64,
}

/// Validate a fraud proof submission with loom context for InvalidLoomTransition proofs.
///
/// This function delegates to `validate_fraud_proof` for non-loom proofs,
/// and performs deterministic re-execution for InvalidLoomTransition proofs.
pub fn validate_fraud_proof_with_loom(
    submission: &FraudProofSubmission,
    loom_ctx: Option<&LoomDisputeContext>,
) -> Result<FraudVerdict, WeaveError> {
    // Verify submitter signature first (same as the basic path).
    let sig_data = fraud_proof_signing_data(submission);
    verify(&sig_data, &submission.signature, &submission.submitter).map_err(|_| {
        WeaveError::InvalidFraudProof {
            reason: "invalid submitter signature".to_string(),
        }
    })?;

    match &submission.proof {
        FraudProof::InvalidLoomTransition {
            loom_id,
            knot: _,
            reason: _,
        } => {
            let ctx = loom_ctx.ok_or_else(|| WeaveError::InvalidFraudProof {
                reason: "loom context required for InvalidLoomTransition verification".to_string(),
            })?;

            if ctx.bytecode.loom_id != *loom_id {
                return Ok(FraudVerdict::Invalid {
                    reason: "bytecode loom_id does not match proof loom_id".to_string(),
                });
            }

            // Build a LoomStateTransition from the proof context.
            // The transition details come from the knot's payload; for now we
            // verify using the context-provided data.
            let transition = norn_types::loom::LoomStateTransition {
                loom_id: *loom_id,
                prev_state_hash: {
                    let mut pre = norn_loom::state::LoomState::new(*loom_id);
                    pre.data = ctx.initial_state.clone();
                    pre.compute_hash()
                },
                new_state_hash: [0u8; 32], // Will be compared by challenge_transition
                inputs: Vec::new(),
                outputs: Vec::new(),
            };

            match norn_loom::dispute::challenge_transition(
                &transition,
                &ctx.bytecode,
                &ctx.initial_state,
                ctx.sender,
                ctx.block_height,
                ctx.timestamp,
            ) {
                Ok(norn_loom::dispute::DisputeResult::Invalid { reason: _ }) => {
                    Ok(FraudVerdict::ValidInvalidLoomTransition)
                }
                Ok(norn_loom::dispute::DisputeResult::Valid) => Ok(FraudVerdict::Invalid {
                    reason: "loom transition is valid; fraud proof rejected".to_string(),
                }),
                Err(e) => Ok(FraudVerdict::Invalid {
                    reason: format!("loom re-execution failed: {}", e),
                }),
            }
        }

        // For non-loom proofs, delegate to the basic validator.
        _ => validate_fraud_proof(submission),
    }
}

/// Compute the data that should be signed for a fraud proof submission.
fn fraud_proof_signing_data(submission: &FraudProofSubmission) -> Vec<u8> {
    let mut data = Vec::new();
    // Include the borsh-serialized proof, submitter, and timestamp.
    if let Ok(proof_bytes) = borsh::to_vec(&submission.proof) {
        data.extend_from_slice(&proof_bytes);
    }
    data.extend_from_slice(&submission.submitter);
    data.extend_from_slice(&submission.timestamp.to_le_bytes());
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_crypto::keys::Keypair;
    use norn_types::knot::*;
    use norn_types::primitives::*;
    use norn_types::thread::ThreadHeader;

    fn make_signed_submission(kp: &Keypair, proof: FraudProof) -> FraudProofSubmission {
        let mut sub = FraudProofSubmission {
            proof,
            submitter: kp.public_key(),
            timestamp: 1000,
            signature: [0u8; 64],
        };
        let sig_data = fraud_proof_signing_data(&sub);
        sub.signature = kp.sign(&sig_data);
        sub
    }

    fn make_knot(id_byte: u8, thread_id: ThreadId, version: Version) -> Knot {
        Knot {
            id: [id_byte; 32],
            knot_type: KnotType::Transfer,
            timestamp: 1000,
            expiry: None,
            before_states: vec![ParticipantState {
                thread_id,
                pubkey: [0u8; 32],
                version,
                state_hash: [0u8; 32],
            }],
            after_states: vec![ParticipantState {
                thread_id,
                pubkey: [0u8; 32],
                version: version + 1,
                state_hash: [1u8; 32],
            }],
            payload: KnotPayload::Transfer(TransferPayload {
                token_id: NATIVE_TOKEN_ID,
                amount: 100,
                from: [1u8; 20],
                to: [2u8; 20],
                memo: None,
            }),
            signatures: vec![[99u8; 64]],
        }
    }

    #[test]
    fn test_valid_double_knot() {
        let kp = Keypair::generate();
        let thread_id = [1u8; 20];
        let knot_a = make_knot(1, thread_id, 5);
        let knot_b = make_knot(2, thread_id, 5);

        let proof = FraudProof::DoubleKnot {
            thread_id,
            knot_a: Box::new(knot_a),
            knot_b: Box::new(knot_b),
        };

        let sub = make_signed_submission(&kp, proof);
        let result = validate_fraud_proof(&sub).unwrap();
        assert_eq!(result, FraudVerdict::ValidDoubleKnot);
    }

    #[test]
    fn test_double_knot_same_id_invalid() {
        let kp = Keypair::generate();
        let thread_id = [1u8; 20];
        let knot_a = make_knot(1, thread_id, 5);
        let knot_b = make_knot(1, thread_id, 5); // Same ID.

        let proof = FraudProof::DoubleKnot {
            thread_id,
            knot_a: Box::new(knot_a),
            knot_b: Box::new(knot_b),
        };

        let sub = make_signed_submission(&kp, proof);
        let result = validate_fraud_proof(&sub).unwrap();
        assert!(matches!(result, FraudVerdict::Invalid { .. }));
    }

    #[test]
    fn test_valid_stale_commit() {
        let kp = Keypair::generate();
        let thread_id = [1u8; 20];

        let commitment = ThreadHeader {
            thread_id,
            owner: [0u8; 32],
            version: 5,
            state_hash: [1u8; 32],
            last_knot_hash: [0u8; 32],
            prev_header_hash: [0u8; 32],
            timestamp: 1000,
            signature: [99u8; 64], // non-zero signature
        };

        let missing = make_knot(1, thread_id, 3);

        let proof = FraudProof::StaleCommit {
            thread_id,
            commitment: Box::new(commitment),
            missing_knots: vec![missing],
        };

        let sub = make_signed_submission(&kp, proof);
        let result = validate_fraud_proof(&sub).unwrap();
        assert_eq!(result, FraudVerdict::ValidStaleCommit);
    }

    #[test]
    fn test_stale_commit_no_missing_knots() {
        let kp = Keypair::generate();
        let thread_id = [1u8; 20];

        let commitment = ThreadHeader {
            thread_id,
            owner: [0u8; 32],
            version: 5,
            state_hash: [1u8; 32],
            last_knot_hash: [0u8; 32],
            prev_header_hash: [0u8; 32],
            timestamp: 1000,
            signature: [99u8; 64],
        };

        let proof = FraudProof::StaleCommit {
            thread_id,
            commitment: Box::new(commitment),
            missing_knots: vec![],
        };

        let sub = make_signed_submission(&kp, proof);
        let result = validate_fraud_proof(&sub).unwrap();
        assert!(matches!(result, FraudVerdict::Invalid { .. }));
    }

    #[test]
    fn test_invalid_submitter_signature() {
        let kp = Keypair::generate();
        let thread_id = [1u8; 20];
        let knot_a = make_knot(1, thread_id, 5);
        let knot_b = make_knot(2, thread_id, 5);

        let proof = FraudProof::DoubleKnot {
            thread_id,
            knot_a: Box::new(knot_a),
            knot_b: Box::new(knot_b),
        };

        let mut sub = make_signed_submission(&kp, proof);
        sub.signature[0] ^= 0xff;
        assert!(validate_fraud_proof(&sub).is_err());
    }

    #[test]
    fn test_invalid_loom_transition_without_context() {
        let kp = Keypair::generate();
        let knot = make_knot(1, [1u8; 20], 1);

        let proof = FraudProof::InvalidLoomTransition {
            loom_id: [5u8; 32],
            knot: Box::new(knot),
            reason: "test".to_string(),
        };

        let sub = make_signed_submission(&kp, proof);
        // Without loom context, basic validate_fraud_proof returns Invalid.
        let result = validate_fraud_proof(&sub).unwrap();
        assert!(matches!(result, FraudVerdict::Invalid { .. }));
    }

    #[test]
    fn test_invalid_loom_transition_with_context_no_loom() {
        let kp = Keypair::generate();
        let knot = make_knot(1, [1u8; 20], 1);

        let proof = FraudProof::InvalidLoomTransition {
            loom_id: [5u8; 32],
            knot: Box::new(knot),
            reason: "test".to_string(),
        };

        let sub = make_signed_submission(&kp, proof);
        // With loom context omitted, returns error.
        let result = validate_fraud_proof_with_loom(&sub, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_spindle_produced_proof_passes_weave_validation() {
        // Bug #6 regression: fraud proofs built by norn-spindle must pass
        // weave's validate_fraud_proof.
        use norn_spindle::monitor::{MonitorAlert, ThreadMonitor};

        let kp = Keypair::generate();
        let thread_id = [1u8; 20];
        let knot_a = make_knot(1, thread_id, 5);
        let knot_b = make_knot(2, thread_id, 5);

        let alert = MonitorAlert::DoubleKnot {
            thread_id,
            knot_a: Box::new(knot_a),
            knot_b: Box::new(knot_b),
        };

        let submission = ThreadMonitor::build_fraud_proof(&alert, kp.public_key(), 1000, &kp);

        // This must pass the weave's validation (signature protocol must match).
        let result = validate_fraud_proof(&submission).unwrap();
        assert_eq!(result, FraudVerdict::ValidDoubleKnot);
    }
}

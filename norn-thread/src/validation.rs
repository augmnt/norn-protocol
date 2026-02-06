use norn_crypto::keys::verify;
use norn_types::constants::MAX_TIMESTAMP_DRIFT;
use norn_types::error::NornError;
use norn_types::knot::*;
use norn_types::primitives::*;
use norn_types::thread::ThreadState;

use crate::knot::compute_knot_id;
use crate::state::compute_state_hash;

/// Context needed to validate a knot against current thread states.
pub struct ValidationContext {
    /// Current versions for each participant (indexed by thread ID).
    pub versions: Vec<(ThreadId, Version)>,
    /// Current state hashes for each participant (indexed by thread ID).
    pub state_hashes: Vec<(ThreadId, Hash)>,
    /// Expected after-state hashes (computed by the validator).
    pub expected_after_hashes: Vec<(ThreadId, Hash)>,
    /// Current time for timestamp validation.
    pub current_time: Timestamp,
    /// Timestamp of the previous knot (0 if this is the first).
    pub previous_knot_timestamp: Timestamp,
}

/// Validate a knot against all 9 rules from spec §6.4.
/// Returns Ok(()) if the knot is valid, or the first error encountered.
pub fn validate_knot(knot: &Knot, ctx: &ValidationContext) -> Result<(), NornError> {
    // Pre-check: minimum 2 participants required for a valid knot.
    if knot.before_states.len() < 2 {
        return Err(NornError::InsufficientParticipants {
            required: 2,
            actual: knot.before_states.len(),
        });
    }
    // Pre-check: before and after state counts must match.
    if knot.before_states.len() != knot.after_states.len() {
        return Err(NornError::InsufficientParticipants {
            required: knot.before_states.len(),
            actual: knot.after_states.len(),
        });
    }

    validate_rule_1_signatures(knot)?;
    validate_rule_2_knot_id(knot)?;
    validate_rule_3_before_versions(knot, ctx)?;
    validate_rule_4_after_versions(knot)?;
    validate_rule_5_before_state_hashes(knot, ctx)?;
    validate_rule_6_after_state_hashes(knot, ctx)?;
    validate_rule_7_payload_consistency(knot)?;
    validate_rule_8_timestamp(knot, ctx)?;
    validate_rule_9_expiry(knot, ctx)?;
    Ok(())
}

/// Rule 1: All signatures are valid Ed25519 over the knot ID.
pub fn validate_rule_1_signatures(knot: &Knot) -> Result<(), NornError> {
    if knot.signatures.len() != knot.before_states.len() {
        return Err(NornError::InvalidSignature { signer_index: 0 });
    }

    for (i, (sig, participant)) in knot
        .signatures
        .iter()
        .zip(knot.before_states.iter())
        .enumerate()
    {
        verify(&knot.id, sig, &participant.pubkey)
            .map_err(|_| NornError::InvalidSignature { signer_index: i })?;
    }
    Ok(())
}

/// Rule 2: Knot ID == BLAKE3(all fields except signatures).
pub fn validate_rule_2_knot_id(knot: &Knot) -> Result<(), NornError> {
    let computed = compute_knot_id(knot);
    if computed != knot.id {
        return Err(NornError::KnotIdMismatch {
            expected: computed,
            actual: knot.id,
        });
    }
    Ok(())
}

/// Rule 3: Each participant's before.version == current version.
pub fn validate_rule_3_before_versions(
    knot: &Knot,
    ctx: &ValidationContext,
) -> Result<(), NornError> {
    for (i, participant) in knot.before_states.iter().enumerate() {
        if let Some((_, expected_version)) = ctx
            .versions
            .iter()
            .find(|(tid, _)| *tid == participant.thread_id)
        {
            if participant.version != *expected_version {
                return Err(NornError::VersionMismatch {
                    participant_index: i,
                    expected: *expected_version,
                    actual: participant.version,
                });
            }
        }
    }
    Ok(())
}

/// Rule 4: Each participant's after.version == before.version + 1.
pub fn validate_rule_4_after_versions(knot: &Knot) -> Result<(), NornError> {
    for (i, (before, after)) in knot
        .before_states
        .iter()
        .zip(knot.after_states.iter())
        .enumerate()
    {
        let expected = before
            .version
            .checked_add(1)
            .ok_or(NornError::VersionOverflow)?;
        if after.version != expected {
            return Err(NornError::VersionMismatch {
                participant_index: i,
                expected,
                actual: after.version,
            });
        }
    }
    Ok(())
}

/// Rule 5: Each participant's before.state_hash == current state hash.
pub fn validate_rule_5_before_state_hashes(
    knot: &Knot,
    ctx: &ValidationContext,
) -> Result<(), NornError> {
    for (i, participant) in knot.before_states.iter().enumerate() {
        if let Some((_, expected_hash)) = ctx
            .state_hashes
            .iter()
            .find(|(tid, _)| *tid == participant.thread_id)
        {
            if participant.state_hash != *expected_hash {
                return Err(NornError::StateHashMismatch {
                    participant_index: i,
                });
            }
        }
    }
    Ok(())
}

/// Rule 6: Each participant's after.state_hash == correct hash of resulting state.
pub fn validate_rule_6_after_state_hashes(
    knot: &Knot,
    ctx: &ValidationContext,
) -> Result<(), NornError> {
    for (i, after) in knot.after_states.iter().enumerate() {
        if let Some((_, expected_hash)) = ctx
            .expected_after_hashes
            .iter()
            .find(|(tid, _)| *tid == after.thread_id)
        {
            if after.state_hash != *expected_hash {
                return Err(NornError::StateHashMismatch {
                    participant_index: i,
                });
            }
        }
    }
    Ok(())
}

/// Rule 7: Payload is internally consistent.
pub fn validate_rule_7_payload_consistency(knot: &Knot) -> Result<(), NornError> {
    match &knot.payload {
        KnotPayload::Transfer(transfer) => {
            if transfer.amount == 0 {
                return Err(NornError::InvalidAmount);
            }
            if let Some(memo) = &transfer.memo {
                if memo.len() > norn_types::constants::MAX_MEMO_SIZE {
                    return Err(NornError::PayloadInconsistent {
                        reason: format!(
                            "memo too large: {} > {}",
                            memo.len(),
                            norn_types::constants::MAX_MEMO_SIZE
                        ),
                    });
                }
            }
            // Verify from/to match participants
            validate_transfer_participants(transfer, knot)?;
            Ok(())
        }
        KnotPayload::MultiTransfer(multi) => {
            if multi.transfers.is_empty() {
                return Err(NornError::PayloadInconsistent {
                    reason: "multi-transfer has no transfers".to_string(),
                });
            }
            if multi.transfers.len() > norn_types::constants::MAX_MULTI_TRANSFERS {
                return Err(NornError::PayloadInconsistent {
                    reason: format!(
                        "too many transfers: {} > {}",
                        multi.transfers.len(),
                        norn_types::constants::MAX_MULTI_TRANSFERS
                    ),
                });
            }
            for transfer in &multi.transfers {
                if transfer.amount == 0 {
                    return Err(NornError::InvalidAmount);
                }
            }
            Ok(())
        }
        KnotPayload::LoomInteraction(loom) => {
            match loom.interaction_type {
                LoomInteractionType::Deposit | LoomInteractionType::Withdraw => {
                    if loom.token_id.is_none() || loom.amount.is_none() {
                        return Err(NornError::PayloadInconsistent {
                            reason: "deposit/withdraw requires token_id and amount".to_string(),
                        });
                    }
                    if loom.amount == Some(0) {
                        return Err(NornError::InvalidAmount);
                    }
                }
                LoomInteractionType::StateUpdate => {}
            }
            Ok(())
        }
    }
}

/// Rule 8: Timestamp within acceptable range.
pub fn validate_rule_8_timestamp(knot: &Knot, ctx: &ValidationContext) -> Result<(), NornError> {
    let max_allowed = ctx.current_time + MAX_TIMESTAMP_DRIFT;
    if knot.timestamp > max_allowed {
        return Err(NornError::TimestampTooFuture {
            timestamp: knot.timestamp,
            max_allowed,
        });
    }

    if ctx.previous_knot_timestamp > 0 && knot.timestamp < ctx.previous_knot_timestamp {
        return Err(NornError::TimestampBeforePrevious {
            timestamp: knot.timestamp,
            previous: ctx.previous_knot_timestamp,
        });
    }

    Ok(())
}

/// Rule 9: If expiry is set, current time < expiry.
pub fn validate_rule_9_expiry(knot: &Knot, ctx: &ValidationContext) -> Result<(), NornError> {
    if let Some(expiry) = knot.expiry {
        if ctx.current_time >= expiry {
            return Err(NornError::KnotExpired {
                expiry,
                current: ctx.current_time,
            });
        }
    }
    Ok(())
}

/// Validate that transfer from/to addresses match knot participants.
fn validate_transfer_participants(
    transfer: &TransferPayload,
    knot: &Knot,
) -> Result<(), NornError> {
    let has_from = knot
        .before_states
        .iter()
        .any(|p| p.thread_id == transfer.from);
    let has_to = knot
        .before_states
        .iter()
        .any(|p| p.thread_id == transfer.to);

    if !has_from || !has_to {
        return Err(NornError::PayloadInconsistent {
            reason: "transfer from/to must be knot participants".to_string(),
        });
    }
    Ok(())
}

/// Build a ValidationContext from current thread states for a simple two-party transfer.
#[allow(clippy::too_many_arguments)]
pub fn build_transfer_context(
    sender_id: ThreadId,
    sender_version: Version,
    sender_state: &ThreadState,
    sender_after_state: &ThreadState,
    receiver_id: ThreadId,
    receiver_version: Version,
    receiver_state: &ThreadState,
    receiver_after_state: &ThreadState,
    current_time: Timestamp,
    previous_knot_timestamp: Timestamp,
) -> ValidationContext {
    ValidationContext {
        versions: vec![(sender_id, sender_version), (receiver_id, receiver_version)],
        state_hashes: vec![
            (sender_id, compute_state_hash(sender_state)),
            (receiver_id, compute_state_hash(receiver_state)),
        ],
        expected_after_hashes: vec![
            (sender_id, compute_state_hash(sender_after_state)),
            (receiver_id, compute_state_hash(receiver_after_state)),
        ],
        current_time,
        previous_knot_timestamp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knot::{add_signature, sign_knot, KnotBuilder};
    use norn_crypto::address::pubkey_to_address;
    use norn_crypto::keys::Keypair;

    #[allow(dead_code)]
    struct TestSetup {
        knot: Knot,
        sender_kp: Keypair,
        receiver_kp: Keypair,
        sender_addr: Address,
        receiver_addr: Address,
        sender_state: ThreadState,
        receiver_state: ThreadState,
        sender_after: ThreadState,
        receiver_after: ThreadState,
    }

    fn setup() -> TestSetup {
        let sender_kp = Keypair::generate();
        let receiver_kp = Keypair::generate();
        let sender_addr = pubkey_to_address(&sender_kp.public_key());
        let receiver_addr = pubkey_to_address(&receiver_kp.public_key());

        let mut sender_state = ThreadState::new();
        sender_state.credit(NATIVE_TOKEN_ID, 1000).unwrap();
        let receiver_state = ThreadState::new();

        let mut sender_after = sender_state.clone();
        let mut receiver_after = receiver_state.clone();
        sender_after.debit(&NATIVE_TOKEN_ID, 500);
        receiver_after.credit(NATIVE_TOKEN_ID, 500).unwrap();

        let mut knot = KnotBuilder::transfer(1000)
            .add_before_state(sender_addr, sender_kp.public_key(), 0, &sender_state)
            .add_before_state(receiver_addr, receiver_kp.public_key(), 0, &receiver_state)
            .add_after_state(sender_addr, sender_kp.public_key(), 1, &sender_after)
            .add_after_state(receiver_addr, receiver_kp.public_key(), 1, &receiver_after)
            .with_payload(KnotPayload::Transfer(TransferPayload {
                token_id: NATIVE_TOKEN_ID,
                amount: 500,
                from: sender_addr,
                to: receiver_addr,
                memo: None,
            }))
            .build()
            .unwrap();

        let sig1 = sign_knot(&knot, &sender_kp);
        let sig2 = sign_knot(&knot, &receiver_kp);
        add_signature(&mut knot, sig1);
        add_signature(&mut knot, sig2);

        TestSetup {
            knot,
            sender_kp,
            receiver_kp,
            sender_addr,
            receiver_addr,
            sender_state,
            receiver_state,
            sender_after,
            receiver_after,
        }
    }

    fn make_context(s: &TestSetup) -> ValidationContext {
        build_transfer_context(
            s.sender_addr,
            0,
            &s.sender_state,
            &s.sender_after,
            s.receiver_addr,
            0,
            &s.receiver_state,
            &s.receiver_after,
            1000,
            0,
        )
    }

    #[test]
    fn test_valid_knot_passes_all_rules() {
        let s = setup();
        let ctx = make_context(&s);
        assert!(validate_knot(&s.knot, &ctx).is_ok());
    }

    #[test]
    fn test_rule_1_invalid_signature() {
        let s = setup();
        let ctx = make_context(&s);
        let mut bad_knot = s.knot.clone();
        bad_knot.signatures[0] = [0u8; 64]; // Corrupt signature
        assert!(matches!(
            validate_knot(&bad_knot, &ctx),
            Err(NornError::InvalidSignature { signer_index: 0 })
        ));
    }

    #[test]
    fn test_rule_1_missing_signature() {
        let s = setup();
        let ctx = make_context(&s);
        let mut bad_knot = s.knot.clone();
        bad_knot.signatures.pop(); // Remove one signature
        assert!(matches!(
            validate_knot(&bad_knot, &ctx),
            Err(NornError::InvalidSignature { .. })
        ));
    }

    #[test]
    fn test_rule_2_wrong_knot_id() {
        let s = setup();
        let ctx = make_context(&s);
        let mut bad_knot = s.knot.clone();
        bad_knot.id = [0xFFu8; 32]; // Wrong ID
                                    // This will fail at rule 1 first (sigs signed over real ID) or rule 2
        assert!(validate_knot(&bad_knot, &ctx).is_err());
    }

    #[test]
    fn test_rule_3_wrong_before_version() {
        let s = setup();
        // Context expects version 5, but knot has version 0
        let ctx = ValidationContext {
            versions: vec![(s.sender_addr, 5), (s.receiver_addr, 0)],
            ..make_context(&s)
        };
        assert!(matches!(
            validate_rule_3_before_versions(&s.knot, &ctx),
            Err(NornError::VersionMismatch {
                participant_index: 0,
                expected: 5,
                actual: 0,
            })
        ));
    }

    #[test]
    fn test_rule_4_wrong_after_version() {
        let s = setup();
        let mut bad_knot = s.knot.clone();
        // after version should be before + 1 = 1, set it to 5
        bad_knot.after_states[0].version = 5;
        assert!(matches!(
            validate_rule_4_after_versions(&bad_knot),
            Err(NornError::VersionMismatch {
                participant_index: 0,
                expected: 1,
                actual: 5,
            })
        ));
    }

    #[test]
    fn test_rule_5_wrong_before_state_hash() {
        let s = setup();
        let wrong_hash = [0xABu8; 32];
        let ctx = ValidationContext {
            state_hashes: vec![(s.sender_addr, wrong_hash), (s.receiver_addr, wrong_hash)],
            ..make_context(&s)
        };
        assert!(matches!(
            validate_rule_5_before_state_hashes(&s.knot, &ctx),
            Err(NornError::StateHashMismatch { .. })
        ));
    }

    #[test]
    fn test_rule_6_wrong_after_state_hash() {
        let s = setup();
        let wrong_hash = [0xABu8; 32];
        let ctx = ValidationContext {
            expected_after_hashes: vec![(s.sender_addr, wrong_hash), (s.receiver_addr, wrong_hash)],
            ..make_context(&s)
        };
        assert!(matches!(
            validate_rule_6_after_state_hashes(&s.knot, &ctx),
            Err(NornError::StateHashMismatch { .. })
        ));
    }

    #[test]
    fn test_rule_7_zero_amount() {
        let s = setup();
        let mut bad_knot = s.knot.clone();
        bad_knot.payload = KnotPayload::Transfer(TransferPayload {
            token_id: NATIVE_TOKEN_ID,
            amount: 0,
            from: s.sender_addr,
            to: s.receiver_addr,
            memo: None,
        });
        assert!(matches!(
            validate_rule_7_payload_consistency(&bad_knot),
            Err(NornError::InvalidAmount)
        ));
    }

    #[test]
    fn test_rule_7_memo_too_large() {
        let s = setup();
        let mut bad_knot = s.knot.clone();
        bad_knot.payload = KnotPayload::Transfer(TransferPayload {
            token_id: NATIVE_TOKEN_ID,
            amount: 500,
            from: s.sender_addr,
            to: s.receiver_addr,
            memo: Some(vec![0u8; 1000]),
        });
        assert!(matches!(
            validate_rule_7_payload_consistency(&bad_knot),
            Err(NornError::PayloadInconsistent { .. })
        ));
    }

    #[test]
    fn test_rule_8_timestamp_too_future() {
        let s = setup();
        let ctx = ValidationContext {
            current_time: 500, // Knot timestamp is 1000
            ..make_context(&s)
        };
        // 1000 > 500 + 300 = 800
        assert!(matches!(
            validate_rule_8_timestamp(&s.knot, &ctx),
            Err(NornError::TimestampTooFuture { .. })
        ));
    }

    #[test]
    fn test_rule_8_timestamp_before_previous() {
        let s = setup();
        let ctx = ValidationContext {
            current_time: 2000,
            previous_knot_timestamp: 1500, // Knot timestamp is 1000 < 1500
            ..make_context(&s)
        };
        assert!(matches!(
            validate_rule_8_timestamp(&s.knot, &ctx),
            Err(NornError::TimestampBeforePrevious { .. })
        ));
    }

    #[test]
    fn test_rule_9_expired_knot() {
        let s = setup();
        let mut bad_knot = s.knot.clone();
        bad_knot.expiry = Some(500); // Expired
        let ctx = ValidationContext {
            current_time: 1000,
            ..make_context(&s)
        };
        assert!(matches!(
            validate_rule_9_expiry(&bad_knot, &ctx),
            Err(NornError::KnotExpired { .. })
        ));
    }

    #[test]
    fn test_rule_9_not_expired() {
        let s = setup();
        let mut good_knot = s.knot.clone();
        good_knot.expiry = Some(5000);
        let ctx = ValidationContext {
            current_time: 1000,
            ..make_context(&s)
        };
        assert!(validate_rule_9_expiry(&good_knot, &ctx).is_ok());
    }

    #[test]
    fn test_version_overflow_returns_error() {
        let s = setup();
        let mut bad_knot = s.knot.clone();
        // Set before version to u64::MAX so +1 would overflow.
        bad_knot.before_states[0].version = u64::MAX;
        bad_knot.after_states[0].version = 0; // Wrapping would give 0
        assert!(matches!(
            validate_rule_4_after_versions(&bad_knot),
            Err(NornError::VersionOverflow)
        ));
    }

    #[test]
    fn test_insufficient_participants_rejected() {
        let kp = Keypair::generate();
        let addr = pubkey_to_address(&kp.public_key());
        let state = ThreadState::new();

        // Build a single-participant knot.
        let mut knot = KnotBuilder::transfer(1000)
            .add_before_state(addr, kp.public_key(), 0, &state)
            .add_after_state(addr, kp.public_key(), 1, &state)
            .with_payload(KnotPayload::Transfer(TransferPayload {
                token_id: NATIVE_TOKEN_ID,
                amount: 100,
                from: addr,
                to: addr,
                memo: None,
            }))
            .build()
            .unwrap();

        let sig = sign_knot(&knot, &kp);
        add_signature(&mut knot, sig);

        let ctx = ValidationContext {
            versions: vec![(addr, 0)],
            state_hashes: vec![(addr, compute_state_hash(&state))],
            expected_after_hashes: vec![(addr, compute_state_hash(&state))],
            current_time: 1000,
            previous_knot_timestamp: 0,
        };

        assert!(matches!(
            validate_knot(&knot, &ctx),
            Err(NornError::InsufficientParticipants {
                required: 2,
                actual: 1
            })
        ));
    }

    #[test]
    fn test_mismatched_before_after_count_rejected() {
        let s = setup();
        let mut bad_knot = s.knot.clone();
        // Remove one after state to create mismatch.
        bad_knot.after_states.pop();
        let ctx = make_context(&s);
        assert!(matches!(
            validate_knot(&bad_knot, &ctx),
            Err(NornError::InsufficientParticipants { .. })
        ));
    }
}

use norn_types::error::NornError;
use norn_types::knot::{
    KnotPayload, LoomInteractionPayload, LoomInteractionType, MultiTransferPayload, TransferPayload,
};
use norn_types::primitives::{Address, Hash};
use norn_types::thread::ThreadState;

/// Compute the state hash of a ThreadState (BLAKE3 of borsh serialization).
pub fn compute_state_hash(state: &ThreadState) -> Hash {
    let encoded = borsh::to_vec(state).expect("ThreadState serialization should not fail");
    norn_crypto::hash::blake3_hash(&encoded)
}

/// Apply a transfer payload to a sender and receiver thread state.
/// Returns error if the sender has insufficient balance.
pub fn apply_transfer(
    sender_state: &mut ThreadState,
    receiver_state: &mut ThreadState,
    payload: &TransferPayload,
) -> Result<(), NornError> {
    if payload.amount == 0 {
        return Err(NornError::InvalidAmount);
    }

    if !sender_state.has_balance(&payload.token_id, payload.amount) {
        return Err(NornError::InsufficientBalance {
            available: sender_state.balance(&payload.token_id),
            required: payload.amount,
        });
    }

    sender_state.debit(&payload.token_id, payload.amount);
    receiver_state.credit(payload.token_id, payload.amount)?;
    Ok(())
}

/// Apply a multi-transfer payload to participating thread states.
/// `states` is a map from Address to mutable ThreadState reference.
pub fn apply_multi_transfer(
    states: &mut std::collections::BTreeMap<Address, ThreadState>,
    payload: &MultiTransferPayload,
) -> Result<(), NornError> {
    // Validate all transfers first, tracking cumulative debits per (sender, token)
    // to catch cases where the same sender is debited multiple times.
    let mut cumulative_debits: std::collections::BTreeMap<(Address, Hash), u128> =
        std::collections::BTreeMap::new();

    for transfer in &payload.transfers {
        if transfer.amount == 0 {
            return Err(NornError::InvalidAmount);
        }
        let sender = states
            .get(&transfer.from)
            .ok_or(NornError::ThreadNotFound(transfer.from))?;
        // Also verify receiver exists.
        if !states.contains_key(&transfer.to) {
            return Err(NornError::ThreadNotFound(transfer.to));
        }

        let key = (transfer.from, transfer.token_id);
        let total_debit = cumulative_debits.entry(key).or_insert(0);
        *total_debit =
            total_debit
                .checked_add(transfer.amount)
                .ok_or(NornError::PayloadInconsistent {
                    reason: "transfer amounts overflow".to_string(),
                })?;

        if !sender.has_balance(&transfer.token_id, *total_debit) {
            return Err(NornError::InsufficientBalance {
                available: sender.balance(&transfer.token_id),
                required: *total_debit,
            });
        }
    }

    // Apply all transfers — debits are guaranteed to succeed by validation above.
    for transfer in &payload.transfers {
        let sender = states.get_mut(&transfer.from).unwrap();
        let debited = sender.debit(&transfer.token_id, transfer.amount);
        debug_assert!(debited, "debit must succeed after validation");

        let receiver = states.get_mut(&transfer.to).unwrap();
        receiver.credit(transfer.token_id, transfer.amount)?;
    }

    Ok(())
}

/// Apply a loom interaction payload to a thread state.
pub fn apply_loom_interaction(
    state: &mut ThreadState,
    payload: &LoomInteractionPayload,
) -> Result<(), NornError> {
    match payload.interaction_type {
        LoomInteractionType::Deposit => {
            let token_id = payload.token_id.ok_or(NornError::PayloadInconsistent {
                reason: "deposit requires token_id".to_string(),
            })?;
            let amount = payload.amount.ok_or(NornError::PayloadInconsistent {
                reason: "deposit requires amount".to_string(),
            })?;
            if amount == 0 {
                return Err(NornError::InvalidAmount);
            }
            if !state.has_balance(&token_id, amount) {
                return Err(NornError::InsufficientBalance {
                    available: state.balance(&token_id),
                    required: amount,
                });
            }
            state.debit(&token_id, amount);
            // Track loom membership
            state.looms.entry(payload.loom_id).or_default();
            Ok(())
        }
        LoomInteractionType::Withdraw => {
            // Verify thread is a member of this loom before allowing withdrawal.
            if !state.looms.contains_key(&payload.loom_id) {
                return Err(NornError::NotLoomParticipant);
            }
            let token_id = payload.token_id.ok_or(NornError::PayloadInconsistent {
                reason: "withdraw requires token_id".to_string(),
            })?;
            let amount = payload.amount.ok_or(NornError::PayloadInconsistent {
                reason: "withdraw requires amount".to_string(),
            })?;
            if amount == 0 {
                return Err(NornError::InvalidAmount);
            }
            state.credit(token_id, amount)?;
            Ok(())
        }
        LoomInteractionType::StateUpdate => {
            // State updates are opaque — just validate the loom membership exists
            if !state.looms.contains_key(&payload.loom_id) {
                return Err(NornError::NotLoomParticipant);
            }
            Ok(())
        }
    }
}

/// Apply a knot payload to the relevant thread states.
pub fn apply_payload(
    sender_state: &mut ThreadState,
    receiver_state: &mut ThreadState,
    payload: &KnotPayload,
) -> Result<(), NornError> {
    match payload {
        KnotPayload::Transfer(transfer) => apply_transfer(sender_state, receiver_state, transfer),
        KnotPayload::MultiTransfer(_) => {
            // Multi-transfer requires the multi-state variant
            Err(NornError::PayloadInconsistent {
                reason: "multi-transfer requires apply_multi_transfer".to_string(),
            })
        }
        KnotPayload::LoomInteraction(loom) => apply_loom_interaction(sender_state, loom),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_types::primitives::NATIVE_TOKEN_ID;

    #[test]
    fn test_compute_state_hash_deterministic() {
        let state = ThreadState::new();
        let h1 = compute_state_hash(&state);
        let h2 = compute_state_hash(&state);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_compute_state_hash_changes() {
        let mut state = ThreadState::new();
        let h1 = compute_state_hash(&state);
        state.credit(NATIVE_TOKEN_ID, 1000).unwrap();
        let h2 = compute_state_hash(&state);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_apply_transfer() {
        let mut sender = ThreadState::new();
        sender.credit(NATIVE_TOKEN_ID, 1000).unwrap();
        let mut receiver = ThreadState::new();

        let payload = TransferPayload {
            token_id: NATIVE_TOKEN_ID,
            amount: 500,
            from: [1u8; 20],
            to: [2u8; 20],
            memo: None,
        };

        apply_transfer(&mut sender, &mut receiver, &payload).unwrap();
        assert_eq!(sender.balance(&NATIVE_TOKEN_ID), 500);
        assert_eq!(receiver.balance(&NATIVE_TOKEN_ID), 500);
    }

    #[test]
    fn test_apply_transfer_insufficient_balance() {
        let mut sender = ThreadState::new();
        sender.credit(NATIVE_TOKEN_ID, 100).unwrap();
        let mut receiver = ThreadState::new();

        let payload = TransferPayload {
            token_id: NATIVE_TOKEN_ID,
            amount: 500,
            from: [1u8; 20],
            to: [2u8; 20],
            memo: None,
        };

        assert!(apply_transfer(&mut sender, &mut receiver, &payload).is_err());
        // State should be unchanged
        assert_eq!(sender.balance(&NATIVE_TOKEN_ID), 100);
        assert_eq!(receiver.balance(&NATIVE_TOKEN_ID), 0);
    }

    #[test]
    fn test_apply_transfer_zero_amount() {
        let mut sender = ThreadState::new();
        sender.credit(NATIVE_TOKEN_ID, 1000).unwrap();
        let mut receiver = ThreadState::new();

        let payload = TransferPayload {
            token_id: NATIVE_TOKEN_ID,
            amount: 0,
            from: [1u8; 20],
            to: [2u8; 20],
            memo: None,
        };

        assert!(apply_transfer(&mut sender, &mut receiver, &payload).is_err());
    }

    // ─── Multi-transfer tests (Bug #1 regression) ─────────────────────────

    #[test]
    fn test_multi_transfer_basic() {
        let addr_a = [1u8; 20];
        let addr_b = [2u8; 20];
        let mut states = std::collections::BTreeMap::new();
        let mut state_a = ThreadState::new();
        state_a.credit(NATIVE_TOKEN_ID, 1000).unwrap();
        states.insert(addr_a, state_a);
        states.insert(addr_b, ThreadState::new());

        let payload = MultiTransferPayload {
            transfers: vec![TransferPayload {
                token_id: NATIVE_TOKEN_ID,
                amount: 500,
                from: addr_a,
                to: addr_b,
                memo: None,
            }],
        };

        apply_multi_transfer(&mut states, &payload).unwrap();
        assert_eq!(states[&addr_a].balance(&NATIVE_TOKEN_ID), 500);
        assert_eq!(states[&addr_b].balance(&NATIVE_TOKEN_ID), 500);
    }

    #[test]
    fn test_multi_transfer_same_sender_double_spend_rejected() {
        // Bug #1: same sender debited twice beyond balance should fail.
        let addr_a = [1u8; 20];
        let addr_b = [2u8; 20];
        let mut states = std::collections::BTreeMap::new();
        let mut state_a = ThreadState::new();
        state_a.credit(NATIVE_TOKEN_ID, 100).unwrap();
        states.insert(addr_a, state_a);
        states.insert(addr_b, ThreadState::new());

        let payload = MultiTransferPayload {
            transfers: vec![
                TransferPayload {
                    token_id: NATIVE_TOKEN_ID,
                    amount: 60,
                    from: addr_a,
                    to: addr_b,
                    memo: None,
                },
                TransferPayload {
                    token_id: NATIVE_TOKEN_ID,
                    amount: 60,
                    from: addr_a,
                    to: addr_b,
                    memo: None,
                },
            ],
        };

        // Total debit is 120, but sender only has 100 — must be rejected.
        let result = apply_multi_transfer(&mut states, &payload);
        assert!(result.is_err());
        // State unchanged on failure.
        assert_eq!(states[&addr_a].balance(&NATIVE_TOKEN_ID), 100);
        assert_eq!(states[&addr_b].balance(&NATIVE_TOKEN_ID), 0);
    }

    #[test]
    fn test_multi_transfer_receiver_not_found() {
        let addr_a = [1u8; 20];
        let addr_b = [2u8; 20];
        let mut states = std::collections::BTreeMap::new();
        let mut state_a = ThreadState::new();
        state_a.credit(NATIVE_TOKEN_ID, 1000).unwrap();
        states.insert(addr_a, state_a);
        // addr_b not in states

        let payload = MultiTransferPayload {
            transfers: vec![TransferPayload {
                token_id: NATIVE_TOKEN_ID,
                amount: 100,
                from: addr_a,
                to: addr_b,
                memo: None,
            }],
        };

        assert!(apply_multi_transfer(&mut states, &payload).is_err());
    }

    // ─── Loom interaction tests (Bug #2 regression) ────────────────────────

    #[test]
    fn test_loom_deposit_adds_membership() {
        let mut state = ThreadState::new();
        state.credit(NATIVE_TOKEN_ID, 1000).unwrap();
        let loom_id = [5u8; 32];

        let payload = LoomInteractionPayload {
            loom_id,
            interaction_type: LoomInteractionType::Deposit,
            token_id: Some(NATIVE_TOKEN_ID),
            amount: Some(500),
            data: vec![],
        };

        apply_loom_interaction(&mut state, &payload).unwrap();
        assert_eq!(state.balance(&NATIVE_TOKEN_ID), 500);
        assert!(state.looms.contains_key(&loom_id));
    }

    #[test]
    fn test_loom_withdraw_requires_membership() {
        // Bug #2: withdraw without being a loom member must fail.
        let mut state = ThreadState::new();
        let loom_id = [5u8; 32];

        let payload = LoomInteractionPayload {
            loom_id,
            interaction_type: LoomInteractionType::Withdraw,
            token_id: Some(NATIVE_TOKEN_ID),
            amount: Some(500),
            data: vec![],
        };

        let result = apply_loom_interaction(&mut state, &payload);
        assert!(result.is_err());
        assert_eq!(state.balance(&NATIVE_TOKEN_ID), 0);
    }

    #[test]
    fn test_loom_withdraw_with_membership_succeeds() {
        let mut state = ThreadState::new();
        let loom_id = [5u8; 32];
        // Add membership.
        state.looms.entry(loom_id).or_default();

        let payload = LoomInteractionPayload {
            loom_id,
            interaction_type: LoomInteractionType::Withdraw,
            token_id: Some(NATIVE_TOKEN_ID),
            amount: Some(500),
            data: vec![],
        };

        apply_loom_interaction(&mut state, &payload).unwrap();
        assert_eq!(state.balance(&NATIVE_TOKEN_ID), 500);
    }

    #[test]
    fn test_loom_state_update_requires_membership() {
        let mut state = ThreadState::new();
        let loom_id = [5u8; 32];

        let payload = LoomInteractionPayload {
            loom_id,
            interaction_type: LoomInteractionType::StateUpdate,
            token_id: None,
            amount: None,
            data: vec![],
        };

        assert!(apply_loom_interaction(&mut state, &payload).is_err());
    }

    // ─── ThreadState credit/debit tests ────────────────────────────────────

    #[test]
    fn test_credit_debit_basic() {
        let mut state = ThreadState::new();
        state.credit(NATIVE_TOKEN_ID, 1000).unwrap();
        assert_eq!(state.balance(&NATIVE_TOKEN_ID), 1000);

        assert!(state.debit(&NATIVE_TOKEN_ID, 400));
        assert_eq!(state.balance(&NATIVE_TOKEN_ID), 600);

        // Debit more than available fails.
        assert!(!state.debit(&NATIVE_TOKEN_ID, 700));
        assert_eq!(state.balance(&NATIVE_TOKEN_ID), 600);
    }

    #[test]
    fn test_debit_nonexistent_token() {
        let mut state = ThreadState::new();
        let token = [99u8; 32];
        assert!(!state.debit(&token, 1));
    }

    #[test]
    fn test_debit_removes_zero_balance() {
        let mut state = ThreadState::new();
        state.credit(NATIVE_TOKEN_ID, 100).unwrap();
        assert!(state.debit(&NATIVE_TOKEN_ID, 100));
        // Balance entry should be removed.
        assert!(!state.balances.contains_key(&NATIVE_TOKEN_ID));
    }
}

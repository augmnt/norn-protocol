use norn_types::error::NornError;
use norn_types::knot::Knot;
use norn_types::primitives::*;
use norn_types::thread::ThreadHeader;

use crate::knot::compute_knot_id;

/// Verify a chain of knots from a known commitment header for a specific thread.
///
/// This verifies that:
/// 1. Each knot in the chain references the correct previous state
/// 2. Versions increment correctly
/// 3. Knot IDs are computed correctly
/// 4. The chain starts from the committed state
///
/// Returns the final thread header state (version + state_hash) if valid.
pub fn verify_knot_chain(
    commitment: &ThreadHeader,
    thread_id: &ThreadId,
    knots: &[Knot],
) -> Result<(Version, Hash), NornError> {
    let mut current_version = commitment.version;
    let mut current_state_hash = commitment.state_hash;

    for (i, knot) in knots.iter().enumerate() {
        // Verify knot ID
        let computed_id = compute_knot_id(knot);
        if computed_id != knot.id {
            return Err(NornError::InvalidKnotChain { index: i });
        }

        // Find this thread's before state in the knot
        let before = knot
            .before_states
            .iter()
            .find(|p| p.thread_id == *thread_id)
            .ok_or(NornError::InvalidKnotChain { index: i })?;

        // Verify before state matches current chain state
        if before.version != current_version {
            return Err(NornError::InvalidKnotChain { index: i });
        }

        if before.state_hash != current_state_hash {
            return Err(NornError::InvalidKnotChain { index: i });
        }

        // Find this thread's after state in the knot
        let after = knot
            .after_states
            .iter()
            .find(|p| p.thread_id == *thread_id)
            .ok_or(NornError::InvalidKnotChain { index: i })?;

        // Verify version increments
        if after.version != before.version + 1 {
            return Err(NornError::InvalidKnotChain { index: i });
        }

        current_version = after.version;
        current_state_hash = after.state_hash;
    }

    Ok((current_version, current_state_hash))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knot::{add_signature, sign_knot, KnotBuilder};
    use crate::state::compute_state_hash;
    use norn_crypto::address::pubkey_to_address;
    use norn_crypto::keys::Keypair;
    use norn_types::knot::{KnotPayload, TransferPayload};
    use norn_types::thread::ThreadState;

    fn make_chain(count: usize) -> (ThreadHeader, ThreadId, Vec<Knot>, Keypair, Keypair) {
        let sender_kp = Keypair::generate();
        let receiver_kp = Keypair::generate();
        let sender_addr = pubkey_to_address(&sender_kp.public_key());
        let receiver_addr = pubkey_to_address(&receiver_kp.public_key());

        let mut sender_state = ThreadState::new();
        sender_state.credit(NATIVE_TOKEN_ID, 1_000_000).unwrap();
        let receiver_state = ThreadState::new();

        let sender_state_hash = compute_state_hash(&sender_state);

        let commitment = ThreadHeader {
            thread_id: sender_addr,
            owner: sender_kp.public_key(),
            version: 0,
            state_hash: sender_state_hash,
            last_knot_hash: [0u8; 32],
            prev_header_hash: [0u8; 32],
            timestamp: 1000,
            signature: [0u8; 64],
        };

        let mut knots = Vec::new();
        let mut s_version = 0u64;
        let mut r_version = 0u64;
        let mut s_state = sender_state.clone();
        let mut r_state = receiver_state.clone();

        for i in 0..count {
            let amount = 10;
            let mut s_after = s_state.clone();
            let mut r_after = r_state.clone();
            s_after.debit(&NATIVE_TOKEN_ID, amount);
            r_after.credit(NATIVE_TOKEN_ID, amount).unwrap();

            let mut knot = KnotBuilder::transfer(1000 + i as u64 + 1)
                .add_before_state(sender_addr, sender_kp.public_key(), s_version, &s_state)
                .add_before_state(receiver_addr, receiver_kp.public_key(), r_version, &r_state)
                .add_after_state(sender_addr, sender_kp.public_key(), s_version + 1, &s_after)
                .add_after_state(
                    receiver_addr,
                    receiver_kp.public_key(),
                    r_version + 1,
                    &r_after,
                )
                .with_payload(KnotPayload::Transfer(TransferPayload {
                    token_id: NATIVE_TOKEN_ID,
                    amount,
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

            s_version += 1;
            r_version += 1;
            s_state = s_after;
            r_state = r_after;

            knots.push(knot);
        }

        (commitment, sender_addr, knots, sender_kp, receiver_kp)
    }

    #[test]
    fn test_valid_chain() {
        let (commitment, thread_id, knots, _, _) = make_chain(10);
        let result = verify_knot_chain(&commitment, &thread_id, &knots);
        assert!(result.is_ok());
        let (version, _) = result.unwrap();
        assert_eq!(version, 10);
    }

    #[test]
    fn test_empty_chain() {
        let (commitment, thread_id, _, _, _) = make_chain(0);
        let result = verify_knot_chain(&commitment, &thread_id, &[]);
        assert!(result.is_ok());
        let (version, state_hash) = result.unwrap();
        assert_eq!(version, 0);
        assert_eq!(state_hash, commitment.state_hash);
    }

    #[test]
    fn test_chain_with_wrong_knot_id() {
        let (commitment, thread_id, mut knots, _, _) = make_chain(5);
        knots[2].id = [0xFFu8; 32]; // Corrupt knot ID
        let result = verify_knot_chain(&commitment, &thread_id, &knots);
        assert!(matches!(
            result,
            Err(NornError::InvalidKnotChain { index: 2 })
        ));
    }

    #[test]
    fn test_chain_with_gap() {
        let (commitment, thread_id, mut knots, _, _) = make_chain(5);
        knots.remove(2); // Create a gap
        let result = verify_knot_chain(&commitment, &thread_id, &knots);
        // Should fail at index 2 because version/state won't match
        assert!(result.is_err());
    }

    #[test]
    fn test_chain_wrong_initial_state() {
        let (mut commitment, thread_id, knots, _, _) = make_chain(5);
        commitment.state_hash = [0xABu8; 32]; // Wrong initial state
        let result = verify_knot_chain(&commitment, &thread_id, &knots);
        assert!(matches!(
            result,
            Err(NornError::InvalidKnotChain { index: 0 })
        ));
    }
}

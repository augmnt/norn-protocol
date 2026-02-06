//! End-to-end integration test exercising the full Norn flow:
//! wallet creation → thread registration → transfer → block production → balance verification.

use norn_crypto::address::pubkey_to_address;
use norn_crypto::keys::Keypair;
use norn_thread::knot::{add_signature, sign_knot, KnotBuilder};
use norn_thread::state::{apply_transfer, compute_state_hash};
use norn_thread::thread::Thread;
use norn_types::knot::{KnotPayload, TransferPayload};
use norn_types::primitives::*;
use norn_types::weave::{
    CommitmentUpdate, FeeState, Registration, Validator, ValidatorSet, WeaveState,
};
use norn_weave::engine::WeaveEngine;

/// Helper: build a signed Registration for a keypair.
fn make_signed_registration(kp: &Keypair, timestamp: Timestamp) -> Registration {
    let thread_id = pubkey_to_address(&kp.public_key());
    let mut reg = Registration {
        thread_id,
        owner: kp.public_key(),
        initial_state_hash: [0u8; 32],
        timestamp,
        signature: [0u8; 64],
    };
    let sig_data = registration_signing_data(&reg);
    reg.signature = kp.sign(&sig_data);
    reg
}

/// Helper: compute the signing data for a Registration.
fn registration_signing_data(reg: &Registration) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&reg.thread_id);
    data.extend_from_slice(&reg.owner);
    data.extend_from_slice(&reg.initial_state_hash);
    data.extend_from_slice(&reg.timestamp.to_le_bytes());
    data
}

/// Helper: compute the signing data for a CommitmentUpdate.
fn commitment_signing_data(c: &CommitmentUpdate) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&c.thread_id);
    data.extend_from_slice(&c.owner);
    data.extend_from_slice(&c.version.to_le_bytes());
    data.extend_from_slice(&c.state_hash);
    data.extend_from_slice(&c.prev_commitment_hash);
    data.extend_from_slice(&c.knot_count.to_le_bytes());
    data.extend_from_slice(&c.timestamp.to_le_bytes());
    data
}

/// Helper: build a signed CommitmentUpdate from a Thread.
fn make_signed_commitment(thread: &Thread, timestamp: Timestamp) -> CommitmentUpdate {
    let state_hash = compute_state_hash(thread.current_state());
    let mut c = CommitmentUpdate {
        thread_id: *thread.address(),
        owner: thread.public_key(),
        version: thread.version(),
        state_hash,
        prev_commitment_hash: [0u8; 32],
        knot_count: thread.uncommitted_count() as u64,
        timestamp,
        signature: [0u8; 64],
    };
    let sig_data = commitment_signing_data(&c);
    c.signature = thread.keypair().sign(&sig_data);
    c
}

fn make_validator_set(kp: &Keypair) -> ValidatorSet {
    let pubkey = kp.public_key();
    ValidatorSet {
        validators: vec![Validator {
            pubkey,
            address: pubkey_to_address(&pubkey),
            stake: 1_000_000_000_000,
            active: true,
        }],
        total_stake: 1_000_000_000_000,
        epoch: 0,
    }
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
fn test_full_e2e_flow() {
    // ── Step 1: Create validator keypair and weave engine ──
    let validator_kp = Keypair::generate();
    let vs = make_validator_set(&validator_kp);
    let mut engine = WeaveEngine::new(validator_kp, vs, make_weave_state());

    assert_eq!(engine.weave_state().height, 0);
    assert_eq!(engine.thread_count(), 0);

    // ── Step 2: Create Alice and Bob keypairs + threads ──
    let alice_kp = Keypair::generate();
    let bob_kp = Keypair::generate();

    let mut alice_thread = Thread::new(
        Keypair::from_seed(&norn_crypto::hash::blake3_hash(&alice_kp.public_key())),
        1000,
    );
    // We need Bob's thread too, but we'll use the bob_kp directly.
    let mut bob_thread = Thread::new(
        Keypair::from_seed(&norn_crypto::hash::blake3_hash(&bob_kp.public_key())),
        1000,
    );

    let alice_addr = *alice_thread.address();
    let bob_addr = *bob_thread.address();

    // ── Step 3: Credit Alice with 1000 NORN ──
    alice_thread
        .current_state_mut()
        .credit(NATIVE_TOKEN_ID, 1000)
        .unwrap();
    assert_eq!(alice_thread.current_state().balance(&NATIVE_TOKEN_ID), 1000);
    assert_eq!(bob_thread.current_state().balance(&NATIVE_TOKEN_ID), 0);

    // ── Step 4: Register Alice's thread ──
    let alice_reg = make_signed_registration(alice_thread.keypair(), 1000);
    assert!(engine.add_registration(alice_reg).is_ok());

    // ── Step 5: Register Bob's thread ──
    let bob_reg = make_signed_registration(bob_thread.keypair(), 1000);
    assert!(engine.add_registration(bob_reg).is_ok());

    // ── Step 6: Produce block → verify registrations ──
    let block1 = engine.produce_block(1001).expect("should produce block 1");
    assert_eq!(block1.height, 1);
    assert_eq!(block1.registrations.len(), 2);
    assert_eq!(block1.commitments.len(), 0);
    assert_eq!(engine.thread_count(), 2);
    assert!(engine.known_threads().contains(&alice_addr));
    assert!(engine.known_threads().contains(&bob_addr));
    assert_eq!(engine.weave_state().height, 1);

    let root_after_registrations = engine.weave_state().threads_root;
    assert_ne!(root_after_registrations, [0u8; 32]);

    // ── Step 7: Build a transfer knot: Alice sends 500 NORN to Bob ──
    let alice_before = alice_thread.current_state().clone();
    let bob_before = bob_thread.current_state().clone();

    let mut alice_after = alice_before.clone();
    let mut bob_after = bob_before.clone();
    apply_transfer(
        &mut alice_after,
        &mut bob_after,
        &TransferPayload {
            token_id: NATIVE_TOKEN_ID,
            amount: 500,
            from: alice_addr,
            to: bob_addr,
            memo: None,
        },
    )
    .expect("transfer should succeed");

    let mut knot = KnotBuilder::transfer(1002)
        .add_before_state(
            alice_addr,
            alice_thread.public_key(),
            alice_thread.version(),
            &alice_before,
        )
        .add_before_state(
            bob_addr,
            bob_thread.public_key(),
            bob_thread.version(),
            &bob_before,
        )
        .add_after_state(
            alice_addr,
            alice_thread.public_key(),
            alice_thread.version() + 1,
            &alice_after,
        )
        .add_after_state(
            bob_addr,
            bob_thread.public_key(),
            bob_thread.version() + 1,
            &bob_after,
        )
        .with_payload(KnotPayload::Transfer(TransferPayload {
            token_id: NATIVE_TOKEN_ID,
            amount: 500,
            from: alice_addr,
            to: bob_addr,
            memo: None,
        }))
        .build()
        .expect("knot build should succeed");

    // ── Step 8: Sign knot with both keypairs ──
    let alice_sig = sign_knot(&knot, alice_thread.keypair());
    let bob_sig = sign_knot(&knot, bob_thread.keypair());
    add_signature(&mut knot, alice_sig);
    add_signature(&mut knot, bob_sig);

    // ── Step 9: Apply knot to both threads ──
    alice_thread
        .apply_knot(knot.clone(), alice_after.clone())
        .expect("alice apply_knot should succeed");
    bob_thread
        .apply_knot(knot, bob_after.clone())
        .expect("bob apply_knot should succeed");

    // Verify local thread balances.
    assert_eq!(alice_thread.current_state().balance(&NATIVE_TOKEN_ID), 500);
    assert_eq!(bob_thread.current_state().balance(&NATIVE_TOKEN_ID), 500);

    // ── Step 10: Commit both threads → create ThreadHeaders ──
    let _alice_header = alice_thread.commit(1003);
    let _bob_header = bob_thread.commit(1003);

    // Verify commits cleared uncommitted knots.
    assert_eq!(alice_thread.uncommitted_count(), 0);
    assert_eq!(bob_thread.uncommitted_count(), 0);

    // ── Step 11: Create and submit CommitmentUpdates ──
    let alice_commitment = make_signed_commitment(&alice_thread, 1003);
    let bob_commitment = make_signed_commitment(&bob_thread, 1003);

    assert!(engine.add_commitment(alice_commitment).is_ok());
    assert!(engine.add_commitment(bob_commitment).is_ok());

    // ── Step 12: Produce block → verify commitments applied ──
    let block2 = engine.produce_block(1004).expect("should produce block 2");
    assert_eq!(block2.height, 2);
    assert_eq!(block2.commitments.len(), 2);
    assert_eq!(block2.registrations.len(), 0);
    assert_eq!(engine.weave_state().height, 2);

    // ── Step 13: Verify balances (thread-level) ──
    assert_eq!(alice_thread.current_state().balance(&NATIVE_TOKEN_ID), 500);
    assert_eq!(bob_thread.current_state().balance(&NATIVE_TOKEN_ID), 500);
    assert_eq!(alice_thread.version(), 1);
    assert_eq!(bob_thread.version(), 1);

    // ── Step 14: Verify weave state height advanced ──
    assert_eq!(engine.weave_state().height, 2);
    assert_ne!(engine.weave_state().latest_hash, [0u8; 32]);

    // ── Step 15: Verify Merkle tree roots changed ──
    let root_after_commitments = engine.weave_state().threads_root;
    assert_ne!(root_after_commitments, [0u8; 32]);
    assert_ne!(root_after_commitments, root_after_registrations);

    // Verify last_block is stored correctly.
    let last = engine.last_block().expect("should have last block");
    assert_eq!(last.height, 2);
}

#[test]
fn test_empty_block_not_produced() {
    let validator_kp = Keypair::generate();
    let vs = make_validator_set(&validator_kp);
    let mut engine = WeaveEngine::new(validator_kp, vs, make_weave_state());

    // No items in mempool → no block produced.
    assert!(engine.produce_block(1000).is_none());
    assert_eq!(engine.weave_state().height, 0);
}

#[test]
fn test_duplicate_registration_rejected() {
    let validator_kp = Keypair::generate();
    let vs = make_validator_set(&validator_kp);
    let mut engine = WeaveEngine::new(validator_kp, vs, make_weave_state());

    let kp = Keypair::generate();
    let reg = make_signed_registration(&kp, 1000);
    assert!(engine.add_registration(reg.clone()).is_ok());

    // Produce block to commit the registration.
    engine.produce_block(1001).expect("should produce block");
    assert_eq!(engine.thread_count(), 1);

    // Try to register the same thread again.
    let reg2 = make_signed_registration(&kp, 1002);
    assert!(engine.add_registration(reg2).is_err());
}

#[test]
fn test_multiple_transfers_across_blocks() {
    let validator_kp = Keypair::generate();
    let vs = make_validator_set(&validator_kp);
    let mut engine = WeaveEngine::new(validator_kp, vs, make_weave_state());

    let alice_kp = Keypair::generate();
    let bob_kp = Keypair::generate();
    let mut alice = Thread::new(
        Keypair::from_seed(&norn_crypto::hash::blake3_hash(&alice_kp.public_key())),
        1000,
    );
    let mut bob = Thread::new(
        Keypair::from_seed(&norn_crypto::hash::blake3_hash(&bob_kp.public_key())),
        1000,
    );
    let alice_addr = *alice.address();
    let bob_addr = *bob.address();

    // Credit Alice.
    alice
        .current_state_mut()
        .credit(NATIVE_TOKEN_ID, 1000)
        .unwrap();

    // Register both.
    engine
        .add_registration(make_signed_registration(alice.keypair(), 1000))
        .unwrap();
    engine
        .add_registration(make_signed_registration(bob.keypair(), 1000))
        .unwrap();
    engine.produce_block(1001).unwrap();

    // Perform 3 transfers: 100 each.
    for i in 0..3u64 {
        let ts = 1002 + i;
        let alice_before = alice.current_state().clone();
        let bob_before = bob.current_state().clone();
        let mut alice_after = alice_before.clone();
        let mut bob_after = bob_before.clone();
        apply_transfer(
            &mut alice_after,
            &mut bob_after,
            &TransferPayload {
                token_id: NATIVE_TOKEN_ID,
                amount: 100,
                from: alice_addr,
                to: bob_addr,
                memo: None,
            },
        )
        .unwrap();

        let mut knot = KnotBuilder::transfer(ts)
            .add_before_state(
                alice_addr,
                alice.public_key(),
                alice.version(),
                &alice_before,
            )
            .add_before_state(bob_addr, bob.public_key(), bob.version(), &bob_before)
            .add_after_state(
                alice_addr,
                alice.public_key(),
                alice.version() + 1,
                &alice_after,
            )
            .add_after_state(bob_addr, bob.public_key(), bob.version() + 1, &bob_after)
            .with_payload(KnotPayload::Transfer(TransferPayload {
                token_id: NATIVE_TOKEN_ID,
                amount: 100,
                from: alice_addr,
                to: bob_addr,
                memo: None,
            }))
            .build()
            .unwrap();

        let sig1 = sign_knot(&knot, alice.keypair());
        let sig2 = sign_knot(&knot, bob.keypair());
        add_signature(&mut knot, sig1);
        add_signature(&mut knot, sig2);

        alice.apply_knot(knot.clone(), alice_after).unwrap();
        bob.apply_knot(knot, bob_after).unwrap();
    }

    // Commit and submit.
    alice.commit(1010);
    bob.commit(1010);

    engine
        .add_commitment(make_signed_commitment(&alice, 1010))
        .unwrap();
    engine
        .add_commitment(make_signed_commitment(&bob, 1010))
        .unwrap();
    let block = engine.produce_block(1011).unwrap();
    assert_eq!(block.commitments.len(), 2);

    // Alice: 1000 - 300 = 700, Bob: 0 + 300 = 300.
    assert_eq!(alice.current_state().balance(&NATIVE_TOKEN_ID), 700);
    assert_eq!(bob.current_state().balance(&NATIVE_TOKEN_ID), 300);
    assert_eq!(alice.version(), 3);
    assert_eq!(bob.version(), 3);
}

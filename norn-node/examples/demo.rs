//! Interactive demo: run with `cargo run --example demo -p norn-node`
//!
//! Demonstrates the full Norn flow:
//! 1. Create wallets for Alice and Bob
//! 2. Register threads on the weave
//! 3. Transfer NORN tokens between them
//! 4. Produce blocks and verify balances

use norn_crypto::address::pubkey_to_address;
use norn_crypto::keys::Keypair;
use norn_thread::knot::{add_signature, sign_knot, KnotBuilder};
use norn_thread::state::{apply_transfer, compute_state_hash};
use norn_thread::thread::Thread;
use norn_types::constants::ONE_NORN;
use norn_types::knot::{KnotPayload, TransferPayload};
use norn_types::primitives::*;
use norn_types::weave::{
    CommitmentUpdate, FeeState, Registration, Validator, ValidatorSet, WeaveState,
};
use norn_weave::engine::WeaveEngine;

fn registration_signing_data(reg: &Registration) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&reg.thread_id);
    data.extend_from_slice(&reg.owner);
    data.extend_from_slice(&reg.initial_state_hash);
    data.extend_from_slice(&reg.timestamp.to_le_bytes());
    data
}

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

fn main() {
    println!("=== Norn Chain - End-to-End Demo ===\n");

    // ── 1. Initialize validator and weave engine ──
    println!("[1] Initializing validator and weave engine...");
    let validator_kp = Keypair::generate();
    let pubkey = validator_kp.public_key();
    let vs = ValidatorSet {
        validators: vec![Validator {
            pubkey,
            address: pubkey_to_address(&pubkey),
            stake: 1_000_000_000_000,
            active: true,
        }],
        total_stake: 1_000_000_000_000,
        epoch: 0,
    };
    let initial_state = WeaveState {
        height: 0,
        latest_hash: [0u8; 32],
        threads_root: [0u8; 32],
        thread_count: 0,
        fee_state: FeeState {
            base_fee: 100,
            fee_multiplier: 1000,
            epoch_fees: 0,
        },
    };
    let mut engine = WeaveEngine::new(validator_kp, vs, initial_state);
    println!("    Validator pubkey: {}", hex::encode(pubkey));
    println!(
        "    Weave height: {}, threads: {}\n",
        engine.weave_state().height,
        engine.thread_count()
    );

    // ── 2. Create Alice and Bob wallets ──
    println!("[2] Creating Alice and Bob wallets...");
    let alice_seed = norn_crypto::hash::blake3_hash(b"alice-demo-seed");
    let bob_seed = norn_crypto::hash::blake3_hash(b"bob-demo-seed");
    let mut alice = Thread::new(Keypair::from_seed(&alice_seed), 1000);
    let mut bob = Thread::new(Keypair::from_seed(&bob_seed), 1000);
    let alice_addr = *alice.address();
    let bob_addr = *bob.address();
    println!("    Alice address: {}", hex::encode(alice_addr));
    println!("    Bob   address: {}\n", hex::encode(bob_addr));

    // ── 3. Credit Alice with 1000 NORN ──
    println!("[3] Crediting Alice with 1000 NORN...");
    alice
        .current_state_mut()
        .credit(NATIVE_TOKEN_ID, 1000 * ONE_NORN)
        .unwrap();
    println!(
        "    Alice balance: {} NORN\n",
        alice.current_state().balance(&NATIVE_TOKEN_ID) / ONE_NORN
    );

    // ── 4. Register threads ──
    println!("[4] Registering Alice's thread...");
    let alice_reg = make_signed_registration(alice.keypair(), 1000);
    engine.add_registration(alice_reg).unwrap();
    println!("    Alice thread registered in mempool.");

    println!("    Registering Bob's thread...");
    let bob_reg = make_signed_registration(bob.keypair(), 1000);
    engine.add_registration(bob_reg).unwrap();
    println!("    Bob thread registered in mempool.\n");

    // ── 5. Produce Block #1 (registrations) ──
    println!("[5] Producing Block #1...");
    let block1 = engine
        .produce_block(1001)
        .expect("block should be produced");
    println!("    Block #{} produced!", block1.height);
    println!("    Hash:          {}", hex::encode(block1.hash));
    println!("    Registrations: {}", block1.registrations.len());
    println!("    Commitments:   {}", block1.commitments.len());
    println!("    Thread count:  {}\n", engine.thread_count());

    // ── 6. Transfer: Alice sends 500 NORN to Bob ──
    println!("[6] Building transfer knot: Alice -> Bob (500 NORN)...");
    let alice_before = alice.current_state().clone();
    let bob_before = bob.current_state().clone();
    let mut alice_after = alice_before.clone();
    let mut bob_after = bob_before.clone();
    let transfer_amount: Amount = 500 * ONE_NORN;

    apply_transfer(
        &mut alice_after,
        &mut bob_after,
        &TransferPayload {
            token_id: NATIVE_TOKEN_ID,
            amount: transfer_amount,
            from: alice_addr,
            to: bob_addr,
            memo: None,
        },
    )
    .expect("transfer should succeed");

    let mut knot = KnotBuilder::transfer(1002)
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
            amount: transfer_amount,
            from: alice_addr,
            to: bob_addr,
            memo: None,
        }))
        .build()
        .expect("knot build should succeed");

    println!("    Knot ID: {}", hex::encode(knot.id));

    // Sign with both parties.
    let sig_a = sign_knot(&knot, alice.keypair());
    let sig_b = sign_knot(&knot, bob.keypair());
    add_signature(&mut knot, sig_a);
    add_signature(&mut knot, sig_b);
    println!("    Signed by Alice and Bob.");

    // Apply to both threads.
    alice
        .apply_knot(knot.clone(), alice_after)
        .expect("alice apply_knot");
    bob.apply_knot(knot, bob_after).expect("bob apply_knot");
    println!(
        "    Alice balance after: {} NORN",
        alice.current_state().balance(&NATIVE_TOKEN_ID) / ONE_NORN
    );
    println!(
        "    Bob   balance after: {} NORN\n",
        bob.current_state().balance(&NATIVE_TOKEN_ID) / ONE_NORN
    );

    // ── 7. Commit threads ──
    println!("[7] Committing thread state...");
    let alice_header = alice.commit(1003);
    let bob_header = bob.commit(1003);
    println!(
        "    Alice committed: version={}, state_hash={}",
        alice_header.version,
        hex::encode(alice_header.state_hash)
    );
    println!(
        "    Bob   committed: version={}, state_hash={}\n",
        bob_header.version,
        hex::encode(bob_header.state_hash)
    );

    // ── 8. Submit commitment updates to the weave ──
    println!("[8] Submitting commitments to weave...");
    let alice_cu = make_signed_commitment(&alice, 1003);
    let bob_cu = make_signed_commitment(&bob, 1003);
    engine.add_commitment(alice_cu).unwrap();
    engine.add_commitment(bob_cu).unwrap();
    println!("    Both commitments accepted into mempool.\n");

    // ── 9. Produce Block #2 (commitments) ──
    println!("[9] Producing Block #2...");
    let block2 = engine
        .produce_block(1004)
        .expect("block should be produced");
    println!("    Block #{} produced!", block2.height);
    println!("    Hash:          {}", hex::encode(block2.hash));
    println!("    Registrations: {}", block2.registrations.len());
    println!("    Commitments:   {}\n", block2.commitments.len());

    // ── 10. Final state ──
    println!("[10] Final weave state:");
    let state = engine.weave_state();
    println!("    Height:       {}", state.height);
    println!("    Latest hash:  {}", hex::encode(state.latest_hash));
    println!("    Threads root: {}", hex::encode(state.threads_root));
    println!("    Thread count: {}", state.thread_count);
    println!();
    println!(
        "    Alice: {} NORN (version {})",
        alice.current_state().balance(&NATIVE_TOKEN_ID) / ONE_NORN,
        alice.version()
    );
    println!(
        "    Bob:   {} NORN (version {})",
        bob.current_state().balance(&NATIVE_TOKEN_ID) / ONE_NORN,
        bob.version()
    );

    println!("\n=== Demo complete! ===");
}

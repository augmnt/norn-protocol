//! TPS benchmark: run with `cargo run --example bench_tps -p norn-node --release`
//!
//! Measures Norn protocol throughput across the full transaction pipeline:
//! keypair generation, transfer construction, knot signing, thread state
//! management, commitment creation, mempool ingestion, and block production.

use std::time::Instant;

use norn_crypto::address::pubkey_to_address;
use norn_crypto::hash::blake3_hash;
use norn_crypto::keys::{batch_verify, verify, Keypair};
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

// ── Helpers (same as demo.rs) ───────────────────────────────────────────

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

// ── Formatting helpers ──────────────────────────────────────────────────

fn fmt_ops(ops: f64) -> String {
    if ops >= 1_000_000.0 {
        format!("{:>7.2}M", ops / 1_000_000.0)
    } else if ops >= 1_000.0 {
        format!("{:>7.1}K", ops / 1_000.0)
    } else {
        format!("{:>8.0}", ops)
    }
}

fn fmt_count(n: usize) -> String {
    if n >= 1_000 {
        format!("{},{:03}", n / 1000, n % 1000)
    } else {
        format!("{}", n)
    }
}

fn fmt_duration(secs: f64) -> String {
    if secs >= 1.0 {
        format!("{:.2}s", secs)
    } else {
        format!("{:.1}ms", secs * 1000.0)
    }
}

// ── Engine factory ──────────────────────────────────────────────────────

fn make_engine() -> WeaveEngine {
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
    WeaveEngine::new(validator_kp, vs, initial_state)
}

// ── Micro-benchmarks ────────────────────────────────────────────────────

fn bench_keygen(n: usize) -> f64 {
    let start = Instant::now();
    for _ in 0..n {
        let _ = std::hint::black_box(Keypair::generate());
    }
    n as f64 / start.elapsed().as_secs_f64()
}

fn bench_sign(n: usize) -> f64 {
    let kp = Keypair::generate();
    let msg = [0xABu8; 32];
    let start = Instant::now();
    for _ in 0..n {
        let _ = std::hint::black_box(kp.sign(&msg));
    }
    n as f64 / start.elapsed().as_secs_f64()
}

fn bench_verify_single(n: usize) -> f64 {
    let kp = Keypair::generate();
    let msg = [0xABu8; 32];
    let sig = kp.sign(&msg);
    let pk = kp.public_key();
    let start = Instant::now();
    for _ in 0..n {
        let _ = std::hint::black_box(verify(&msg, &sig, &pk));
    }
    n as f64 / start.elapsed().as_secs_f64()
}

fn bench_batch_verify(n: usize) -> f64 {
    let keypairs: Vec<Keypair> = (0..n)
        .map(|i| {
            let seed = blake3_hash(&(i as u64).to_le_bytes());
            Keypair::from_seed(&seed)
        })
        .collect();
    let messages: Vec<[u8; 32]> = (0..n)
        .map(|i| blake3_hash(&(i as u64 + 0x1000000).to_le_bytes()))
        .collect();
    let signatures: Vec<Signature> = keypairs
        .iter()
        .zip(messages.iter())
        .map(|(kp, msg)| kp.sign(msg))
        .collect();
    let pubkeys: Vec<PublicKey> = keypairs.iter().map(|kp| kp.public_key()).collect();
    let msg_refs: Vec<&[u8]> = messages.iter().map(|m| m.as_slice()).collect();

    let start = Instant::now();
    let _ = std::hint::black_box(batch_verify(&msg_refs, &signatures, &pubkeys));
    n as f64 / start.elapsed().as_secs_f64()
}

fn bench_blake3(n: usize) -> f64 {
    let data = [0u8; 256];
    let start = Instant::now();
    for _ in 0..n {
        let _ = std::hint::black_box(blake3_hash(&data));
    }
    n as f64 / start.elapsed().as_secs_f64()
}

fn bench_apply_transfer(n: usize) -> f64 {
    use norn_types::thread::ThreadState;
    let mut senders: Vec<_> = (0..n)
        .map(|_| {
            let mut s = ThreadState::new();
            s.credit(NATIVE_TOKEN_ID, 1000 * ONE_NORN).unwrap();
            s
        })
        .collect();
    let mut receivers: Vec<_> = (0..n).map(|_| ThreadState::new()).collect();
    let payload = TransferPayload {
        token_id: NATIVE_TOKEN_ID,
        amount: 100 * ONE_NORN,
        from: [1u8; 20],
        to: [2u8; 20],
        memo: None,
    };

    let start = Instant::now();
    for i in 0..n {
        apply_transfer(&mut senders[i], &mut receivers[i], &payload).unwrap();
    }
    n as f64 / start.elapsed().as_secs_f64()
}

fn bench_mempool_ingestion(n: usize) -> f64 {
    let ts_base: Timestamp = 200_000;
    let mut engine = make_engine();

    // Register threads and produce registration block
    let threads: Vec<Thread> = (0..n)
        .map(|i| {
            let seed = blake3_hash(&(i as u64 + 0x8000000).to_le_bytes());
            let kp = Keypair::from_seed(&seed);
            let mut t = Thread::new(kp, ts_base);
            t.current_state_mut()
                .credit(NATIVE_TOKEN_ID, 1000 * ONE_NORN)
                .unwrap();
            t
        })
        .collect();

    for t in &threads {
        engine
            .add_registration(make_signed_registration(t.keypair(), ts_base))
            .unwrap();
    }
    engine.produce_block(ts_base + 1, [0u8; 32]);

    // Pre-build commitments, then measure just the add_commitment calls
    let commitments: Vec<CommitmentUpdate> = threads
        .iter()
        .map(|t| make_signed_commitment(t, ts_base + 2))
        .collect();

    let start = Instant::now();
    for c in commitments {
        engine.add_commitment(c).unwrap();
    }
    n as f64 / start.elapsed().as_secs_f64()
}

// ── End-to-end TPS benchmark ────────────────────────────────────────────

struct E2eResult {
    n: usize,
    commits: usize,
    setup_secs: f64,
    registration_block_secs: f64,
    transfer_secs: f64,
    ingestion_secs: f64,
    block_production_secs: f64,
}

fn bench_e2e(n: usize) -> E2eResult {
    let n = if !n.is_multiple_of(2) { n + 1 } else { n };
    let ts_base: Timestamp = 100_000;

    let setup_start = Instant::now();

    // 1. Create engine + threads
    let mut engine = make_engine();
    let mut threads: Vec<Thread> = (0..n)
        .map(|i| {
            let seed = blake3_hash(&(i as u64).to_le_bytes());
            let kp = Keypair::from_seed(&seed);
            let mut t = Thread::new(kp, ts_base);
            t.current_state_mut()
                .credit(NATIVE_TOKEN_ID, 1000 * ONE_NORN)
                .unwrap();
            t
        })
        .collect();

    // 2. Register all threads
    for t in &threads {
        engine
            .add_registration(make_signed_registration(t.keypair(), ts_base))
            .unwrap();
    }
    let reg_start = Instant::now();
    engine
        .produce_block(ts_base + 1, [0u8; 32])
        .expect("registration block");
    let registration_block_secs = reg_start.elapsed().as_secs_f64();

    // 3. Build transfers in pairs, sign, apply, commit
    let ts_knot = ts_base + 2;
    let ts_commit = ts_base + 3;
    let transfer_amount: Amount = 100 * ONE_NORN;

    let transfer_start = Instant::now();
    for pair_idx in 0..(n / 2) {
        let i = pair_idx * 2;
        let j = i + 1;

        let (left, right) = threads.split_at_mut(j);
        let sender = &mut left[i];
        let receiver = &mut right[0];

        let sender_addr = *sender.address();
        let receiver_addr = *receiver.address();

        let sender_before = sender.current_state().clone();
        let receiver_before = receiver.current_state().clone();
        let mut sender_after = sender_before.clone();
        let mut receiver_after = receiver_before.clone();

        let payload = TransferPayload {
            token_id: NATIVE_TOKEN_ID,
            amount: transfer_amount,
            from: sender_addr,
            to: receiver_addr,
            memo: None,
        };

        apply_transfer(&mut sender_after, &mut receiver_after, &payload).unwrap();

        let mut knot = KnotBuilder::transfer(ts_knot)
            .add_before_state(
                sender_addr,
                sender.public_key(),
                sender.version(),
                &sender_before,
            )
            .add_before_state(
                receiver_addr,
                receiver.public_key(),
                receiver.version(),
                &receiver_before,
            )
            .add_after_state(
                sender_addr,
                sender.public_key(),
                sender.version() + 1,
                &sender_after,
            )
            .add_after_state(
                receiver_addr,
                receiver.public_key(),
                receiver.version() + 1,
                &receiver_after,
            )
            .with_payload(KnotPayload::Transfer(payload))
            .build()
            .unwrap();

        let sig_s = sign_knot(&knot, sender.keypair());
        let sig_r = sign_knot(&knot, receiver.keypair());
        add_signature(&mut knot, sig_s);
        add_signature(&mut knot, sig_r);

        sender.apply_knot(knot.clone(), sender_after).unwrap();
        receiver.apply_knot(knot, receiver_after).unwrap();

        sender.commit(ts_commit);
        receiver.commit(ts_commit);
    }
    let transfer_secs = transfer_start.elapsed().as_secs_f64();

    // 4. Submit commitments to mempool
    let ingest_start = Instant::now();
    for t in &threads {
        engine
            .add_commitment(make_signed_commitment(t, ts_commit))
            .unwrap();
    }
    let ingestion_secs = ingest_start.elapsed().as_secs_f64();

    let setup_secs = setup_start.elapsed().as_secs_f64();

    // 5. Produce block (the core measurement)
    let block_start = Instant::now();
    let block = engine
        .produce_block(ts_commit + 1, [0u8; 32])
        .expect("commitment block");
    let block_production_secs = block_start.elapsed().as_secs_f64();

    E2eResult {
        n,
        commits: block.commitments.len(),
        setup_secs,
        registration_block_secs,
        transfer_secs,
        ingestion_secs,
        block_production_secs,
    }
}

fn print_e2e_result(r: &E2eResult) {
    let tps = r.commits as f64 / r.block_production_secs;
    println!(
        "  {:>5} wallets | {:>5} commits | {} block | {} TPS",
        fmt_count(r.n),
        fmt_count(r.commits),
        fmt_duration(r.block_production_secs),
        fmt_ops(tps),
    );
    println!(
        "    breakdown: reg_block={} transfers={} ingest={} total_setup={}",
        fmt_duration(r.registration_block_secs),
        fmt_duration(r.transfer_secs),
        fmt_duration(r.ingestion_secs),
        fmt_duration(r.setup_secs),
    );
}

// ── Main ────────────────────────────────────────────────────────────────

fn main() {
    let is_release = cfg!(not(debug_assertions));

    println!();
    println!("\x1b[35m═══════════════════════════════════════════════════\x1b[0m");
    println!(
        "\x1b[35;1m  NORN PROTOCOL — TPS BENCHMARK (v{})\x1b[0m",
        env!("CARGO_PKG_VERSION")
    );
    println!("\x1b[35m═══════════════════════════════════════════════════\x1b[0m");
    println!();
    println!(
        "  Build: {}",
        if is_release {
            "release"
        } else {
            "\x1b[33mdebug (run with --release for accurate results)\x1b[0m"
        }
    );
    println!();

    // ── Micro-benchmarks ────────────────────────────────────────────
    println!("\x1b[36m── Micro-benchmarks ───────────────────────────────\x1b[0m");

    let ops = bench_keygen(10_000);
    println!(
        "  Ed25519 keygen        10,000 ops    {} ops/sec",
        fmt_ops(ops)
    );

    let ops = bench_sign(10_000);
    println!(
        "  Ed25519 sign          10,000 ops    {} ops/sec",
        fmt_ops(ops)
    );

    let ops = bench_verify_single(10_000);
    println!(
        "  Ed25519 verify        10,000 ops    {} ops/sec",
        fmt_ops(ops)
    );

    let ops = bench_batch_verify(10_000);
    println!(
        "  Ed25519 batch verify  10,000 sigs   {} ops/sec",
        fmt_ops(ops)
    );

    let ops = bench_blake3(100_000);
    println!(
        "  BLAKE3 hash (256B)   100,000 ops    {} ops/sec",
        fmt_ops(ops)
    );

    let ops = bench_apply_transfer(10_000);
    println!(
        "  Transfer apply        10,000 ops    {} ops/sec",
        fmt_ops(ops)
    );

    let ops = bench_mempool_ingestion(100);
    println!(
        "  Mempool ingestion        100 ops    {} ops/sec",
        fmt_ops(ops)
    );

    println!();

    // ── End-to-end TPS ──────────────────────────────────────────────
    println!("\x1b[36m── End-to-End TPS ─────────────────────────────────\x1b[0m");
    println!("  Full pipeline: keygen + register + transfer + sign + commit + produce_block");
    println!();

    for &size in &[10, 50, 100, 200, 500] {
        let r = bench_e2e(size);
        print_e2e_result(&r);
        println!();
    }

    println!("  \x1b[33mNote: Block production scales O(n^2) due to per-commitment");
    println!("  merkle root recomputation in SparseMerkleTree.\x1b[0m");
    println!();
    println!("\x1b[35m═══════════════════════════════════════════════════\x1b[0m");
    println!();
}

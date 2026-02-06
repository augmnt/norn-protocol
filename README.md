# Norn Protocol

[![CI](https://github.com/augmnt/norn-protocol/actions/workflows/ci.yml/badge.svg)](https://github.com/augmnt/norn-protocol/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

**Your thread. Your fate. The chain just watches.**

---

## What is Norn?

Norn is a radically minimal blockchain protocol that reimagines the relationship between users and the chain. Rather than forcing every transaction through global consensus -- the bottleneck that limits every existing blockchain -- Norn treats the chain as a **courtroom**, not a bank.

Users transact directly with each other using cryptographic signatures, maintaining their own personal state histories called *Threads*. The chain intervenes only when there is a dispute, processing fraud proofs rather than transactions. This architectural inversion moves the vast majority of economic activity off-chain by design, with the anchor chain serving as a minimal, efficient arbiter of last resort.

For complex multi-party logic, off-chain smart contracts called *Looms* provide WebAssembly-powered programmability with on-chain fraud proof guarantees. The result is a protocol where bilateral exchange is instant, free, and private -- and the chain exists only to keep everyone honest.

## Key Properties

- **Unlimited bilateral throughput** -- Two parties can exchange value as fast as they can sign messages. No block size limit, no gas auction, no mempool congestion.
- **Phone-runnable full nodes** -- The anchor chain processes only commitments and fraud proofs, keeping on-chain state minimal. A full node runs on a modern smartphone.
- **Zero-fee P2P transfers** -- Bilateral transactions incur no on-chain fee. Only periodic commitments to the anchor chain carry a small dynamic fee.
- **Privacy by default** -- The chain never sees transaction details, balances, or counterparties. It sees only cryptographic commitments.
- **Instant bilateral finality** -- A transaction is final the moment both parties sign. No confirmation time, no block wait.
- **Fraud-proof security** -- Cheating is detectable and punishable through economic penalties. Honest behavior is the Nash equilibrium.

## Architecture

Norn's architecture consists of six core components:

| Component | Description |
|-----------|-------------|
| **Threads** | Personal state chains -- each user maintains their own signed history of state transitions, stored locally on their device. |
| **Knots** | Atomic state transitions -- bilateral or multilateral agreements that tie Threads together, signed by all participants. |
| **Weave** | The anchor chain -- a minimal HotStuff BFT blockchain that processes commitments, registrations, and fraud proofs. |
| **Looms** | Off-chain smart contracts -- WebAssembly programs that execute off-chain with on-chain fraud proof guarantees. |
| **Spindles** | Watchtower services -- monitor the Weave on behalf of offline users and submit fraud proofs when misbehavior is detected. |
| **Relays** | P2P message buffers -- asynchronous message delivery between Threads via the libp2p protocol stack. |

```mermaid
flowchart TB
    subgraph Threads
        A["Thread A<br/>(Alice)"]
        B["Thread B<br/>(Bob)"]
    end

    A <-->|"Bilateral Knots<br/>(instant, free, private)"| B

    A -->|"Periodic commitments<br/>(state hash + version)"| W
    B -->|"Periodic commitments<br/>(state hash + version)"| W

    subgraph W["The Weave (Anchor Chain -- HotStuff BFT Consensus)"]
        C[Commitments]
        R[Registrations]
        F[Fraud Proofs]
        L2[Looms]
    end

    SP["Spindles<br/>(Watchtower Services)"] --> W
    LM["Looms<br/>(Off-chain Contracts)"] --> W
    RL["Relays<br/>(P2P Message Buffers)"] --> W
```

## Repository Structure

| Crate | Description |
|-------|-------------|
| `norn-types` | Shared type definitions (Thread, Knot, Weave, Loom, consensus, fraud proof, genesis, network message types) |
| `norn-crypto` | Cryptographic operations (Ed25519 keys, BLAKE3 hashing, Merkle trees, BIP-39 seeds, SLIP-0010 HD derivation, XChaCha20 encryption) |
| `norn-thread` | Thread management (Thread chain, Knot creation/validation, state management, version tracking) |
| `norn-storage` | Storage abstraction (KvStore trait with memory, SQLite, and RocksDB backends; Merkle, Thread, and Weave stores) |
| `norn-relay` | P2P networking (libp2p behaviour, protocol codec, peer discovery, relay service, Spindle registry) |
| `norn-weave` | Anchor chain (block production, commitment processing, HotStuff consensus, dynamic fees, fraud proof verification, staking) |
| `norn-loom` | Smart contract runtime (Wasm runtime, host functions, gas metering, Loom lifecycle, dispute resolution) |
| `norn-spindle` | Watchtower service (Weave monitoring, fraud proof construction, rate limiting, service orchestration) |
| `norn-node` | Full node binary (CLI, node configuration, genesis handling, JSON-RPC server, wallet CLI, metrics) |

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)

### Build

```bash
cargo build --workspace
```

### Test

```bash
cargo test --workspace
```

### Lint

```bash
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

### Run the Demo

```bash
cargo run --example demo -p norn-node
```

## Wallet CLI

The `norn-node` binary includes a full-featured wallet CLI with 17 subcommands for key management, transfers, Thread inspection, and encrypted keystore backup.

```bash
# Create a new wallet
cargo run -p norn-node -- wallet create

# List wallets
cargo run -p norn-node -- wallet list

# Check balance
cargo run -p norn-node -- wallet balance --address <ADDRESS>

# Send tokens
cargo run -p norn-node -- wallet send --to <ADDRESS> --amount <AMOUNT>
```

Wallets are stored in `~/.norn/wallets/` with Argon2id key derivation and XChaCha20-Poly1305 authenticated encryption.

## Documentation

- [White Paper](docs/Norn_Protocol_White_Paper.md) -- Design philosophy, architecture overview, and protocol comparison
- [Protocol Specification v2.0](docs/Norn_Protocol_Specification_v2.0.md) -- Complete technical specification

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the [MIT License](LICENSE).

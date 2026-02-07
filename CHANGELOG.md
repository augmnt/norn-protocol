# Changelog

All notable changes to the Norn Protocol will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-02-07

### Added

- Persistent storage with RocksDB backend and automatic state rebuild on restart
- Multi-node P2P devnet with peer discovery and state sync
- Production-ready CLI with network identity (`dev`/`testnet`/`mainnet`), token economics, and `cargo install` support
- Full codebase audit (WS1-WS8): wired consensus message routing, spindle watchtower, fee collection, fraud proof submission RPC, Prometheus metrics endpoint, RPC auth middleware
- Devnet genesis with founder allocation (10M NORN to Augmnt founder address)
- P2P genesis hash validation to prevent cross-network state sync
- Secured founder wallet — replaced public seed with private keypair stored in encrypted local keystore
- 488 tests (up from 430+), 21 RPC endpoints, 20 wallet subcommands

### Security

- RPC API key authentication via tower HTTP middleware
- Founder keypair stored in encrypted local keystore (no public seed exposure)
- Genesis hash validation prevents connecting to wrong network

## [0.2.0] - 2026-02-07

### Added

- NornNames native name registry (register, resolve, list owned names)
- P2P relay with state sync via libp2p request-response
- Fully functional CLI with StateManager, real RPC endpoints, and `--dev` mode
- CLI startup banner (ANSI Shadow "NORN" art)
- 3 new wallet subcommands: `register-name`, `resolve`, `names` (17 → 20 total)

## [0.1.0] - 2026-02-06

### Added

- **norn-types**: Shared type definitions for Thread, Knot, Weave, Loom, consensus, fraud proof, genesis, and network message types
- **norn-crypto**: Ed25519 signatures, BLAKE3 hashing, Merkle trees, BIP-39 mnemonics, SLIP-0010 HD derivation, XChaCha20-Poly1305 encryption, Shamir's Secret Sharing
- **norn-thread**: Thread chain management, Knot creation and validation, state tracking, version management
- **norn-storage**: KvStore trait abstraction with memory, SQLite, and RocksDB backends; Merkle, Thread, and Weave stores
- **norn-relay**: P2P networking via libp2p with gossipsub, protocol codec, peer discovery, relay service, Spindle registry
- **norn-weave**: Anchor chain with block production, commitment processing, HotStuff BFT consensus, dynamic EIP-1559-style fees, fraud proof verification, staking
- **norn-loom**: Off-chain smart contract runtime with WebAssembly (Wasmtime), host functions, gas metering, Loom lifecycle, dispute resolution
- **norn-spindle**: Watchtower service with thread monitoring, fraud proof construction, rate limiting, service orchestration
- **norn-node**: Full node binary with CLI, JSON-RPC server (jsonrpsee), wallet CLI (20 subcommands), Prometheus metrics, genesis handling
- Wallet keystore with Argon2id KDF (per-wallet random salt), XChaCha20-Poly1305 authenticated encryption, and backward-compatible v1/v2 support
- Testnet faucet endpoint (feature-gated behind `testnet` feature, enabled by default)
- Protocol Specification v2.0 (2150+ lines) and White Paper (856 lines)
- CI pipeline with build, test, clippy, and format checks
- 430+ tests including end-to-end, wallet, regression, Shamir, consensus, and validation tests

### Security

- Wallet keystore v3 with per-wallet random salt (replaces fixed salt in v2)
- Faucet endpoint gated behind `#[cfg(feature = "testnet")]` compile-time flag
- Zero `unsafe` blocks across the entire codebase

# Changelog

All notable changes to the Norn Protocol will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.19.0] - 2026-02-15

### Added

- Validator reward distribution — epoch fees redistributed to validators proportional to stake (replaces fee-burn model)
- `norn wallet rewards` CLI command for viewing pending epoch fees and projected validator rewards
- `norn_getValidatorRewards` RPC endpoint
- `FormButton` component with contextual disabled hints across all 20 wallet form pages
- `FieldError` inline validation component for address and hex inputs
- Contracts page Browse/Interact tabs with deployed contract discovery and structured result display
- `scripts/setup-node.sh` one-command node installer

### Changed

- `debit_fee()` no longer burns fees — supply is preserved, fees flow to validators
- Disabled button styling improved: `opacity-30` + `cursor-not-allowed` (was `opacity-50` + `pointer-events-none`)

## [0.18.0] - 2026-02-12

### Added

- Block explorer (`explorer/`) — Next.js 15 + shadcn/ui with real-time block, transaction, token, and contract views
- Web wallet (`wallet/`) — PWA with mobile-first design, encrypted keystore, passkey recovery, 11 smart contract apps
- Wallet extension for Chrome — sign transactions from web apps
- Website (`website/`) — norn.network marketing site with documentation
- Synthetic transfer records for complete transaction history (genesis, fees, mints, burns)
- `norn_getBlockTransactions` RPC endpoint
- Address labels for known addresses (genesis, faucet)
- 8 example smart contracts: governance, escrow, vesting, treasury, timelock, swap, splitter, crowdfund
- Balance history and activity charts in wallet dashboard
- Block production timing metrics with microsecond precision

### Fixed

- Faucet credit propagation through blocks to all nodes (v0.18.1)
- Token symbol display in RPC responses (v0.18.4)
- Custom token decimal formatting in transfers (v0.18.3)

## [0.17.0] - 2026-02-11

### Added

- WebSocket subscriptions — real-time streaming for blocks, transfers, tokens, looms, and pending transactions
- Cross-contract calls via `norn_call_contract` host function with re-entrancy protection (`MAX_CALL_DEPTH = 8`)
- TypeScript SDK (`@norn-protocol/sdk`) — NornClient, NornWallet, NornSubscriber, address utilities (51 tests)
- `RpcBroadcasters` for grouped event distribution across 5 channels

## [0.16.0] - 2026-02-10

### Added

- Multi-validator HotStuff BFT consensus with leader rotation
- Staking system with `stake`, `unstake` commands and bonding period
- Slashing for conflicting block production
- State proofs — Merkle proof generation and verification for account balances
- Dynamic validator set — join/leave based on stake weight
- Epoch system — 1000-block epochs for fee accumulation and validator rotation

### Changed

- PROTOCOL_VERSION 6 → 8, SCHEMA_VERSION 6 → 7 (breaking)

## [0.15.0] - 2026-02-10

### Added

- Tuple `StorageKey` — `Map<(Address, Address), u128>` composite keys
- `safe_add`/`safe_sub`/`safe_mul` checked arithmetic returning `ContractError`
- `Response::with_action()`, `add_address()`, `add_u128()` builder methods
- `event!` macro for concise event construction
- `Item::init()`/`Map::init()` panicking save for initialization
- `Item::update_or()`/`Map::update_or()` update with default
- `Response::merge()` for stdlib composability
- Test address constants: `ALICE`, `BOB`, `CHARLIE`, `DAVE`
- `assert_data::<T>()` and `assert_err_contains()` test helpers

## [0.14.0] - 2026-02-09

### Added

- `#[norn_contract]` proc macro — eliminates ~60% of boilerplate
- Direct method calls in tests (`counter.increment(&ctx)`)
- Typed init params via proc macro
- `norn-sdk-macros` crate

## [0.13.0] - 2026-02-09

### Added

- Typed `InitMsg` — `type Init = MyInitMsg` for constructor parameters
- Structured events — `Event::new("Transfer").add_attribute("from", hex)`
- Standard library — `Ownable`, `Pausable`, `Norn20` (ERC20-equivalent) composable modules
- `IndexedMap` — iterable map with `keys()`, `range()`, `len()`

### Changed

- `Contract` trait now requires `type Init` associated type and second param on `fn init()`

## [0.12.0] - 2026-02-09

### Added

- `Item<T>` / `Map<K, V>` type-safe storage primitives
- `Response` builder pattern for contract return values
- `ensure!`, `ensure_eq!`, `ensure_ne!` guard macros
- `TestEnv` native test harness (no Wasm needed for testing)

### Changed

- `ContractResult` changed from `Result<Vec<u8>, ContractError>` to `Result<Response, ContractError>`

## [0.10.0] - 2026-02-08

### Added

- Loom Phase 2 — full smart contract execution with `norn-sdk`
- `norn_uploadLoomBytecode`, `norn_executeLoom`, `norn_queryLoom`, `norn_joinLoom`, `norn_leaveLoom` RPC methods
- Counter contract example (`examples/counter/`)
- 5 new wallet commands: `upload-bytecode`, `execute-loom`, `query-loom`, `join-loom`, `leave-loom`

## [0.9.0] - 2026-02-08

### Added

- Loom Phase 1 — smart contract registration and deployment
- `LoomRegistration` in `WeaveBlock` with P2P propagation
- `norn_deployLoom`, `norn_getLoomInfo`, `norn_listLooms` RPC methods
- 3 new wallet commands: `deploy-loom`, `loom-info`, `list-looms`

### Changed

- PROTOCOL_VERSION 5 → 6, SCHEMA_VERSION 4 → 5 (breaking)

## [0.8.0] - 2026-02-07

### Added

- NT-1 fungible token system — create, mint, burn custom tokens
- Token operations in `WeaveBlock` with P2P propagation
- 6 new RPC endpoints for token management
- 5 new wallet commands: `create-token`, `mint-token`, `burn-token`, `token-info`, `list-tokens`
- Token symbol resolution in CLI (`--token MTK`) (v0.8.1)
- `token-balances` command for all non-zero holdings (v0.8.1)
- comfy-table formatted CLI output (v0.8.2)

### Changed

- PROTOCOL_VERSION 4 → 5, SCHEMA_VERSION 3 → 4 (breaking)

## [0.5.0] - 2026-02-07

### Added

- Consensus-level NornNames with P2P propagation via blocks
- Rolling upgrade system with envelope protocol and dual-decode codec
- mDNS auto-discovery and `--boot-node` CLI flag
- Default bootstrap node with DNS transport

### Changed

- PROTOCOL_VERSION 1 → 2 (NornNames wire format)
- SCHEMA_VERSION 1 → 2 (WeaveBlock schema change)

## [0.3.0] - 2026-02-07

### Added

- Persistent storage with RocksDB backend and automatic state rebuild on restart
- Multi-node P2P devnet with peer discovery and state sync
- Production-ready CLI with network identity (`dev`/`testnet`/`mainnet`), token economics, and `cargo install` support
- Full codebase audit (WS1-WS8): wired consensus message routing, spindle watchtower, fee collection, fraud proof submission RPC, Prometheus metrics endpoint, RPC auth middleware
- Devnet genesis with founder allocation (10M NORN to Augmnt founder address)
- P2P genesis hash validation to prevent cross-network state sync
- Secured founder wallet — replaced public seed with private keypair stored in encrypted local keystore
- 4 new wallet commands: `node-info`, `fees`, `validators`, `whoami` (20 → 24 total); `send` alias for `transfer`
- 488 tests (up from 430+), 21 RPC endpoints, 24 wallet subcommands

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
- **norn-node**: Full node binary with CLI, JSON-RPC server (jsonrpsee), wallet CLI (24 subcommands), Prometheus metrics, genesis handling
- Wallet keystore with Argon2id KDF (per-wallet random salt), XChaCha20-Poly1305 authenticated encryption, and backward-compatible v1/v2 support
- Testnet faucet endpoint (feature-gated behind `testnet` feature, enabled by default)
- Protocol Specification v2.0 (2150+ lines) and White Paper (856 lines)
- CI pipeline with build, test, clippy, and format checks
- 430+ tests including end-to-end, wallet, regression, Shamir, consensus, and validation tests

### Security

- Wallet keystore v3 with per-wallet random salt (replaces fixed salt in v2)
- Faucet endpoint gated behind `#[cfg(feature = "testnet")]` compile-time flag
- Zero `unsafe` blocks across the entire codebase

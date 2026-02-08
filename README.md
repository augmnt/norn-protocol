# Norn

**Your thread. Your fate. The chain just watches.**

<p>
  <a href="https://github.com/augmnt/norn-protocol/actions/workflows/ci.yml"><img src="https://github.com/augmnt/norn-protocol/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-stable-orange.svg" alt="Rust"></a>
  <a href="https://github.com/augmnt/norn-protocol/releases/tag/v0.9.0"><img src="https://img.shields.io/badge/version-0.9.0-green.svg" alt="Version"></a>
</p>

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

## Installation

### From Git (latest)

```bash
cargo install --git https://github.com/augmnt/norn-protocol norn-node
```

### From Source

```bash
git clone https://github.com/augmnt/norn-protocol
cd norn-protocol
cargo install --path norn-node
```

After installation, the `norn` command is available:

```bash
norn --version
norn wallet create --name mywallet
norn run --dev
```

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

## Network Modes

Norn supports three network modes, selectable via `--network` flag or `network_id` in `norn.toml`:

| Mode | Chain ID | Faucet | Use Case |
|------|----------|--------|----------|
| `dev` | `norn-dev` | Enabled (60s cooldown) | Local development, solo validator |
| `testnet` | `norn-testnet-1` | Enabled (1hr cooldown) | Public testing, multi-node |
| `mainnet` | `norn-mainnet` | Disabled | Production deployment |

```bash
# Dev mode (default)
norn run --dev

# Testnet mode
norn run --dev --network testnet

# Mainnet mode (requires genesis file)
norn run --network mainnet --genesis genesis/mainnet.json
```

## Repository Structure

| Crate | Description |
|-------|-------------|
| `norn-types` | Shared type definitions (Thread, Knot, Weave, Loom, consensus, fraud proof, genesis, network message types) |
| `norn-crypto` | Cryptographic operations (Ed25519 keys, BLAKE3 hashing, Merkle trees, BIP-39 seeds, SLIP-0010 HD derivation, XChaCha20 encryption) |
| `norn-thread` | Thread management (Thread chain, Knot creation/validation, state management, version tracking) |
| `norn-storage` | Storage abstraction (KvStore trait with memory, SQLite, and RocksDB backends; Merkle, Thread, and Weave stores) |
| `norn-relay` | P2P networking (libp2p behaviour, protocol codec, peer discovery, relay service, state sync, Spindle registry) |
| `norn-weave` | Anchor chain (block production, commitment processing, HotStuff consensus, dynamic fees, fraud proof verification, staking) |
| `norn-loom` | Smart contract runtime (Wasm runtime, host functions, gas metering, Loom lifecycle, dispute resolution) |
| `norn-spindle` | Watchtower service (Weave monitoring, fraud proof construction, rate limiting, service orchestration) |
| `norn-node` | Full node binary (CLI, node configuration, genesis handling, JSON-RPC server with API key auth, wallet CLI, NornNames, NT-1 tokens, Loom smart contracts, Prometheus metrics endpoint, fraud proof submission, spindle watchtower integration) |

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

The `norn` binary includes a full-featured wallet CLI with 36 subcommands for key management, transfers, NornNames, custom tokens, Loom smart contracts, Thread inspection, and encrypted keystore backup.

```bash
# Create a new wallet
norn wallet create --name alice

# List wallets
norn wallet list

# Check balance (native NORN or custom token by symbol)
norn wallet balance --address <ADDRESS>
norn wallet balance --token MTK

# Send tokens (by address or NornName)
norn wallet transfer --to <ADDRESS_OR_NAME> --amount <AMOUNT>
norn wallet transfer --to <ADDRESS_OR_NAME> --amount 100 --token MTK
# Or use the `send` alias
norn wallet send --to <ADDRESS_OR_NAME> --amount <AMOUNT>

# Register a NornName (costs 1 NORN, burned)
norn wallet register-name --name alice

# Resolve a NornName to its owner address
norn wallet resolve --name alice

# List names owned by the active wallet
norn wallet names

# Configure wallet (network, RPC URL)
norn wallet config --network testnet
norn wallet config --rpc-url http://my-node:9741

# Check node connectivity
norn wallet node-info

# View current fees
norn wallet fees

# View validator set
norn wallet validators

# Active wallet dashboard
norn wallet whoami
```

### Token Management

```bash
# Create a custom fungible token (costs 10 NORN, burned)
norn wallet create-token --name "My Token" --symbol MTK --decimals 8 --max-supply 1000000 --initial-supply 1000

# Mint tokens (creator only)
norn wallet mint-token --token MTK --to 0x<ADDRESS> --amount 500

# Burn tokens (any holder)
norn wallet burn-token --token MTK --amount 100

# Query token metadata
norn wallet token-info MTK

# List all registered tokens
norn wallet list-tokens

# View all token balances for the active wallet
norn wallet token-balances
```

The `--token` flag on `balance` and `transfer` accepts token symbols (e.g., `MTK`), `NORN`/`native` for the native token, or a 64-character hex token ID.

Wallets are stored in `~/.norn/wallets/` with Argon2id key derivation and XChaCha20-Poly1305 authenticated encryption.

## NornNames

NornNames is Norn's native **consensus-level** name system, mapping human-readable names to owner addresses as a user-friendly alternative to hex addresses. Names are included in `WeaveBlock`s and propagate to all nodes via P2P gossip, making them globally visible across the network.

### Naming Rules

| Rule | Constraint |
|------|-----------|
| Length | 3--32 characters |
| Character set | Lowercase ASCII letters (`a-z`), digits (`0-9`), hyphens (`-`) |
| Hyphens | Must not start or end with a hyphen |
| Uniqueness | Globally unique, first-come first-served |

**Valid names:** `alice`, `bob-42`, `my-validator`, `norn-relay-1`

**Invalid names:** `ab` (too short), `-alice` (leading hyphen), `bob-` (trailing hyphen), `Alice` (uppercase), `my name` (spaces)

### Registration Cost

Registering a NornName costs **1 NORN**, which is **permanently burned** (debited from the registrant, not credited to anyone), reducing the circulating supply.

### Wallet CLI Usage

```bash
# Register a NornName for the active wallet (submitted to mempool, included in next block)
norn wallet register-name --name alice

# Resolve a NornName to its owner address
norn wallet resolve --name alice

# List names owned by the active wallet
norn wallet names
```

Names work seamlessly in transfers -- pass a NornName instead of a hex address:

```bash
norn wallet send --to alice --amount 10
```

The wallet resolves `alice` to the owner's address via `norn_resolveName` before constructing the transfer.

For full technical details, see the [Protocol Specification, Section 28](docs/Norn_Protocol_Specification_v2.0.md#28-nornnames-name-registry).

## NT-1 Fungible Token Standard

Norn supports **protocol-level custom fungible tokens** via the NT-1 standard. Tokens are consensus-level: definitions, mints, and burns are included in `WeaveBlock`s and propagate to all nodes via P2P gossip.

### Token Operations

| Operation | Who | Fee | Effect |
|-----------|-----|-----|--------|
| **Create** | Anyone | 10 NORN (burned) | Registers a new token with metadata; optionally mints initial supply to creator |
| **Mint** | Token creator only | None | Creates new tokens, credits to recipient |
| **Burn** | Any holder | None | Destroys tokens from burner's balance |

**Token ID** is deterministic: `BLAKE3(creator ++ name ++ symbol ++ decimals ++ max_supply ++ timestamp)`.

**Symbol uniqueness** is enforced at the consensus level -- no two tokens can share the same ticker.

### Token Rules

| Rule | Constraint |
|------|-----------|
| Name length | 1--64 printable ASCII characters |
| Symbol length | 1--12 uppercase alphanumeric characters |
| Decimals | 0--18 |
| Max supply | 0 = unlimited, otherwise enforced on mint |

### Wallet CLI Usage

```bash
# Create a token
norn wallet create-token --name "Wrapped Bitcoin" --symbol WBTC --decimals 8 --max-supply 21000000 --initial-supply 0

# Mint tokens to a recipient (creator only)
norn wallet mint-token --token WBTC --to 0x<ADDRESS> --amount 1000

# Transfer custom tokens
norn wallet transfer --to 0x<ADDRESS> --amount 100 --token WBTC

# Burn tokens from your own balance
norn wallet burn-token --token WBTC --amount 50

# Query token info (by symbol or hex token ID)
norn wallet token-info WBTC

# List all tokens on the network
norn wallet list-tokens

# View all non-zero token holdings for the active wallet
norn wallet token-balances
```

### RPC Methods

| Method | Parameters | Returns | Auth |
|--------|-----------|---------|------|
| `norn_createToken` | `hex` (hex-encoded borsh `TokenDefinition`) | `SubmitResult` | Yes |
| `norn_mintToken` | `hex` (hex-encoded borsh `TokenMint`) | `SubmitResult` | Yes |
| `norn_burnToken` | `hex` (hex-encoded borsh `TokenBurn`) | `SubmitResult` | Yes |
| `norn_getTokenInfo` | `token_id` (hex) | `Option<TokenInfo>` | No |
| `norn_getTokenBySymbol` | `symbol` | `Option<TokenInfo>` | No |
| `norn_listTokens` | `limit`, `offset` | `Vec<TokenInfo>` | No |

For full technical details, see the [Protocol Specification, Section 28b](docs/Norn_Protocol_Specification_v2.0.md#28b-nt-1-fungible-token-standard).

## Loom Smart Contracts

Norn supports **off-chain smart contracts** called Looms -- WebAssembly programs that execute off-chain with on-chain fraud proof guarantees. Loom deployments are consensus-level: registrations are included in `WeaveBlock`s and propagate to all nodes via P2P gossip.

### Loom Operations

| Operation | Who | Fee | Effect |
|-----------|-----|-----|--------|
| **Deploy** | Anyone | 50 NORN (burned) | Registers a new loom with metadata on the network |

**Loom ID** is deterministic: `BLAKE3(name ++ operator ++ timestamp)`.

### Naming Rules

| Rule | Constraint |
|------|-----------|
| Length | 3--64 characters |
| Character set | Lowercase ASCII letters (`a-z`), digits (`0-9`), hyphens (`-`) |
| Hyphens | Must not start or end with a hyphen |

### Wallet CLI Usage

```bash
# Deploy a loom (costs 50 NORN, burned)
norn wallet deploy-loom --name my-contract

# Query loom metadata
norn wallet loom-info <LOOM_ID>

# List all deployed looms
norn wallet list-looms
```

### RPC Methods

| Method | Parameters | Returns | Auth |
|--------|-----------|---------|------|
| `norn_deployLoom` | `hex` (hex-encoded borsh `LoomRegistration`) | `SubmitResult` | Yes |
| `norn_getLoomInfo` | `loom_id` (hex) | `Option<LoomInfo>` | No |
| `norn_listLooms` | `limit`, `offset` | `Vec<LoomInfo>` | No |

For full technical details, see the [Protocol Specification](docs/Norn_Protocol_Specification_v2.0.md).

## Token Economics

NORN has a fixed maximum supply of **1,000,000,000 NORN** (1 billion), enforced at the protocol level.

| Category | % | Amount | Vesting |
|---|---|---|---|
| Founder & Core Team | 15% | 150,000,000 | 4-year linear, 1-year cliff |
| Ecosystem Development | 20% | 200,000,000 | Controlled release over 5 years |
| Validator Rewards | 30% | 300,000,000 | Block rewards over 10+ years |
| Community & Grants | 15% | 150,000,000 | Governance-controlled |
| Treasury Reserve | 10% | 100,000,000 | DAO-governed after decentralization |
| Initial Liquidity | 5% | 50,000,000 | Available at launch |
| Testnet Participants | 5% | 50,000,000 | Airdrop at mainnet launch |

**Deflationary mechanics:** NornNames registration burns 1 NORN per name. NT-1 token creation burns 10 NORN per token. Loom deployment burns 50 NORN per contract. Future fee burning (EIP-1559-style) planned.

For full details, see the [Protocol Specification](docs/Norn_Protocol_Specification_v2.0.md).

## Documentation

- [White Paper](docs/Norn_Protocol_White_Paper.md) -- Design philosophy, architecture overview, and protocol comparison
- [Protocol Specification v2.0](docs/Norn_Protocol_Specification_v2.0.md) -- Complete technical specification

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the [MIT License](LICENSE).

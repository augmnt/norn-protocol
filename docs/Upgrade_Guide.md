# Norn Protocol Upgrade Guide

## Version Compatibility Matrix

| From Version | To Version | P2P Compatible | State Compatible | Action Required |
|-------------|------------|----------------|------------------|-----------------|
| v0.3.0      | v0.5.x     | No             | No               | `--reset-state`, re-register names |
| v0.4.1      | v0.5.x     | No             | No               | `--reset-state`, re-register names |
| v0.5.0      | v0.5.1+    | Yes*           | Yes*             | Restart node |
| v0.7.x      | v0.8.0     | No             | No               | `--reset-state` (PROTOCOL_VERSION 4→5, SCHEMA_VERSION 3→4) |
| v0.8.0      | v0.8.1     | Yes            | Yes              | Restart node (CLI-only changes, no protocol/schema bump) |
| v0.8.x      | v0.9.0     | No             | No               | `--reset-state` (PROTOCOL_VERSION 5→6, SCHEMA_VERSION 4→5) |
| v0.9.x      | v0.10.0    | Yes            | No               | `--reset-state` (SCHEMA_VERSION 5→6, P2P compatible) |
| v0.10.x     | v0.11.0    | Yes            | Yes              | Restart node (SDK-only changes, no protocol/schema bump) |
| v0.11.x     | v0.12.0    | Yes            | Yes              | Restart node (SDK-only changes, no protocol/schema bump) |
| v0.12.x     | v0.13.0    | Yes            | Yes              | Restart node (SDK + runtime internal changes, no protocol/schema bump) |
| v0.13.x     | v0.14.0    | Yes            | Yes              | Restart node (SDK-only changes, no protocol/schema bump) |
| v0.14.x     | v0.15.0    | Yes            | Yes              | Restart node (SDK-only changes, no protocol/schema bump) |
| v0.15.x     | v0.16.0    | No             | No               | `--reset-state` (PROTOCOL_VERSION 6→8, SCHEMA_VERSION 6→7) |
| v0.16.x     | v0.17.0    | No             | No               | `--reset-state` (PROTOCOL_VERSION 8→9) |
| v0.17.x     | v0.18.0    | Yes            | Yes              | Restart node (explorer, wallet extension, no protocol/schema bump) |
| v0.18.x     | v0.19.0    | Yes            | Yes              | Restart node + state reset recommended (fee economics changed) |
| v0.19.0     | v0.19.1    | Yes            | Yes              | No action required (wallet-only changes) |

\* Within a minor version line, compatibility depends on whether PROTOCOL_VERSION or SCHEMA_VERSION was bumped. Check the release notes.

## Upgrade Procedures

### Upgrading from v0.3.0 or v0.4.1 to v0.5.x

These are **breaking upgrades**. Both the P2P wire format and the state schema have changed.

1. **Stop the node.**

2. **Build the new version:**
   ```bash
   cargo build --release -p norn-node
   cargo install --path norn-node
   ```

3. **Start with `--reset-state`:**
   ```bash
   norn run --dev --reset-state
   ```
   This wipes the data directory and starts fresh. Wallet files in `~/.norn/wallets/` are **not** affected.

4. **Re-register NornNames:**
   Names registered on v0.4.x were local-only and are not carried forward. After the node is running and your account is funded:
   ```bash
   norn wallet register-name --name your-name
   ```
   Names are now consensus-level: they propagate via blocks to all nodes on the network.

5. **Reconnect P2P peers:**
   All peers must be running v0.5.x. Nodes with mismatched PROTOCOL_VERSION will reject each other's messages.

### Upgrading from v0.7.x to v0.8.0

This is a **breaking upgrade**. PROTOCOL_VERSION changed from 4 to 5, and SCHEMA_VERSION changed from 3 to 4.

1. **Stop the node.**

2. **Build the new version:**
   ```bash
   cargo build --release -p norn-node
   cargo install --path norn-node
   ```

3. **Start with `--reset-state`:**
   ```bash
   norn run --dev --reset-state
   ```

4. **Re-register NornNames and re-create tokens:**
   After the node is running and your account is funded:
   ```bash
   norn wallet register-name --name your-name
   norn wallet create-token --name "My Token" --symbol MTK --decimals 8 --max-supply 1000000 --initial-supply 1000
   ```

5. **Reconnect P2P peers:**
   All peers must be running v0.8.0. The three new `NornMessage` variants (discriminants 15-17) are not supported by older nodes.

### What's Safe Across Upgrades

- **Wallet files** (`~/.norn/wallets/*.json`): Encrypted keystores are version-independent. Your private keys, addresses, and wallet passwords remain valid across any upgrade.
- **Genesis config files** (`genesis.json`): Forward-compatible via `serde(default)` on the `version` field. Old genesis files work with new nodes.

### What Gets Wiped by `--reset-state`

- Block history
- Thread registrations
- Account balances (re-funded via genesis allocations or faucet)
- NornNames (must re-register)
- Custom tokens (must re-create)
- Deployed looms (must re-deploy)
- Commitment history

## Version Constants

The protocol uses three version constants to detect incompatibilities:

| Constant | Location | Current | Purpose |
|----------|----------|---------|---------|
| `PROTOCOL_VERSION` | `norn-relay/src/protocol.rs` | 9 | P2P wire format version. Mismatch = messages rejected. |
| `SCHEMA_VERSION` | `norn-node/src/state_store.rs` | 7 | Borsh state schema version. Mismatch = node refuses to start (suggests `--reset-state`). |
| `GENESIS_CONFIG_VERSION` | `norn-types/src/genesis.rs` | 1 | Genesis config format version. Included in genesis hash computation. |

## Multi-Node P2P Requirements

All nodes in a P2P network **must** run the same PROTOCOL_VERSION. The P2P wire format includes a version byte in every message:

```
[4-byte length][1-byte PROTOCOL_VERSION][borsh payload]
```

Nodes receiving a message with a different PROTOCOL_VERSION will log a warning and drop the message.

Additionally, nodes validate genesis hash on state sync. Two nodes with different genesis configurations will refuse to sync state with each other.

## Future Upgrade Expectations

- **State migrations** are not yet automated. Breaking schema changes require `--reset-state`.
- **Rolling upgrades** are supported via the envelope protocol, dual-decode codec, and versioned GossipSub topics (introduced in v0.6.0). Nodes can run mixed protocol versions during the upgrade window.
- **`MessageEnvelope`** is the default wire format for protocol v5+. Legacy decode (v3) is also supported for backward compatibility.
- **Binary version negotiation** between peers uses the `identify` protocol's `agent_version` field (e.g., `"norn/5"`). Peers with newer versions trigger an `UpgradeNotice` log message.

## Changelog: v0.5.x Breaking Changes

### NornNames (Consensus-Level)

Previously, NornNames were local-only: registered via RPC, stored in StateManager, invisible to other nodes. In v0.5.x, names are consensus-level:

- `NameRegistration` type added to `WeaveBlock`
- Names propagate via P2P gossip and block inclusion
- `NornMessage::NameRegistration` variant added (discriminant 13)
- Wallet CLI now signs a `NameRegistration` object (not a knot)
- Names submitted to mempool, included in next block

### WeaveBlock Schema Change

Two new fields added to `WeaveBlock`:
- `name_registrations: Vec<NameRegistration>`
- `name_registrations_root: Hash`

This changes the borsh serialization layout, hence SCHEMA_VERSION bump from 1 to 2.

### P2P Protocol Change

New `NornMessage::NameRegistration` variant changes the borsh enum layout, hence PROTOCOL_VERSION bump from 1 to 2.

## Changelog: v0.8.0 Breaking Changes (NT-1 Token System)

### Upgrading from v0.7.x to v0.8.0

This is a **breaking upgrade**. Both the P2P wire format and the state schema have changed.

1. **Stop the node.**

2. **Build the new version:**
   ```bash
   cargo install --path norn-node
   ```

3. **Start with `--reset-state`:**
   ```bash
   norn run --dev --reset-state
   ```

4. **Re-register NornNames and re-create tokens:**
   Names and custom tokens are lost on state reset. After the node is running and your account is funded:
   ```bash
   norn wallet register-name --name your-name
   norn wallet create-token --name "My Token" --symbol MTK --decimals 8 --max-supply 1000000 --initial-supply 1000
   ```

5. **All peers must be running v0.8.0.** The new `NornMessage` variants (discriminants 15-17) are not supported by older nodes.

### WeaveBlock Schema Change

Six new fields added to `WeaveBlock`:
- `token_definitions: Vec<TokenDefinition>`
- `token_definitions_root: Hash`
- `token_mints: Vec<TokenMint>`
- `token_mints_root: Hash`
- `token_burns: Vec<TokenBurn>`
- `token_burns_root: Hash`

This changes the borsh serialization layout, hence SCHEMA_VERSION bump from 3 to 4.

### P2P Protocol Change

Three new `NornMessage` variants change the borsh enum layout:
- `TokenDefinition(TokenDefinition)` — discriminant 15
- `TokenMint(TokenMint)` — discriminant 16
- `TokenBurn(TokenBurn)` — discriminant 17

These are only supported over the v5+ envelope protocol. The legacy codec rejects discriminants > 13. PROTOCOL_VERSION bumped from 4 to 5.

### State Store Change

New key prefix `state:token:` for token persistence. Existing state stores lack this prefix and will not have token data. Combined with the schema version mismatch, `--reset-state` is required.

### New RPC Endpoints

Six new RPC methods: `norn_createToken`, `norn_mintToken`, `norn_burnToken`, `norn_getTokenInfo`, `norn_getTokenBySymbol`, `norn_listTokens`. The three read-only methods are added to the unauthenticated whitelist.

### New Wallet Commands

Five new wallet subcommands: `create-token`, `mint-token`, `burn-token`, `token-info`, `list-tokens`.

### Upgrading from v0.8.0 to v0.8.1

This is a **non-breaking upgrade**. No protocol or schema changes.

1. **Build the new version:**
   ```bash
   cargo install --path norn-node
   ```

2. **Restart the node** (no `--reset-state` needed):
   ```bash
   norn run --dev
   ```

### v0.8.1 Changelog

- **`--token` flag fix:** `balance` and `transfer` now accept token symbols (e.g., `MTK`) via RPC symbol lookup, not just 64-char hex IDs.
- **Token display fix:** Custom token amounts now show their symbol name instead of truncated hex.
- **`token-info NORN`:** Native NORN is handled locally, displaying protocol-level metadata.
- **`whoami` enhancements:** Shows custom token balances (non-zero) and current block height.
- **`balance` block height:** Displays the current block height for timing context.
- **New `token-balances` command:** Lists all non-zero token holdings (NORN + custom tokens) for the active wallet. Supports `--json` and `--rpc-url`.

## Changelog: v0.9.0 Breaking Changes (Loom Smart Contracts)

### Upgrading from v0.8.x to v0.9.0

This is a **breaking upgrade**. PROTOCOL_VERSION changed from 5 to 6, and SCHEMA_VERSION changed from 4 to 5.

1. **Stop the node.**

2. **Build the new version:**
   ```bash
   cargo install --path norn-node
   ```

3. **Start with `--reset-state`:**
   ```bash
   norn run --dev --reset-state
   ```

4. **Re-register NornNames, re-create tokens, and re-deploy looms:**
   Names, custom tokens, and looms are lost on state reset. After the node is running and your account is funded:
   ```bash
   norn wallet register-name --name your-name
   norn wallet create-token --name "My Token" --symbol MTK --decimals 8 --max-supply 1000000 --initial-supply 1000
   norn wallet deploy-loom --name my-contract
   ```

5. **All peers must be running v0.9.0.** The new `NornMessage` variants (discriminants 18-19) are not supported by older nodes.

### WeaveBlock Schema Change

Two new fields added to `WeaveBlock`:
- `loom_deploys: Vec<LoomRegistration>`
- `loom_deploys_root: Hash`

This changes the borsh serialization layout, hence SCHEMA_VERSION bump from 4 to 5.

### P2P Protocol Change

Two new `NornMessage` variants change the borsh enum layout:
- `LoomDeploy(Box<LoomRegistration>)` — discriminant 18
- `LoomExecution(Box<LoomStateTransition>)` — discriminant 19

These are only supported over the v6+ envelope protocol. The legacy codec rejects discriminants > 17. PROTOCOL_VERSION bumped from 5 to 6.

### State Store Change

New key prefix `state:loom:` for loom persistence. Additional prefixes `state:loom_bytecode:` and `state:loom_state:` are reserved for Phase 2 execution. Combined with the schema version mismatch, `--reset-state` is required.

### New RPC Endpoints

Three new RPC methods: `norn_deployLoom`, `norn_getLoomInfo`, `norn_listLooms`. The two read-only methods (`getLoomInfo`, `listLooms`) are added to the unauthenticated whitelist.

### New Wallet Commands

Three new wallet subcommands: `deploy-loom`, `loom-info`, `list-looms` (36 total).

## Changelog: v0.10.0 (Loom Phase 2 — Execution)

### Upgrading from v0.9.x to v0.10.0

This is a **state-incompatible** upgrade. SCHEMA_VERSION changed from 5 to 6. PROTOCOL_VERSION remains at 6 (no P2P changes — execution is off-chain).

1. **Stop the node.**

2. **Build the new version:**
   ```bash
   cargo install --path norn-node
   ```

3. **Start with `--reset-state`:**
   ```bash
   norn run --dev --reset-state
   ```

4. **Re-register NornNames, re-create tokens, and re-deploy looms:**
   ```bash
   norn wallet register-name --name your-name
   norn wallet create-token --name "My Token" --symbol MTK --decimals 8 --max-supply 1000000 --initial-supply 1000
   norn wallet deploy-loom --name my-contract
   norn wallet upload-bytecode --loom-id <LOOM_ID> --bytecode path/to/contract.wasm
   ```

5. **P2P compatible:** v0.9.x and v0.10.0 nodes can communicate since PROTOCOL_VERSION is unchanged. However, v0.9.x nodes cannot execute or query loom bytecode.

### New Crate: norn-sdk

New workspace member `norn-sdk` — a `#![no_std]` contract SDK for writing loom smart contracts in Rust targeting `wasm32-unknown-unknown`. Provides host function bindings, output buffer management, encoding helpers, and memory allocation exports.

### Counter Contract Example

New `examples/counter/` — a working counter contract built with `norn-sdk` demonstrating init, execute (increment/decrement/reset), and query operations.

### State Store Change

The previously-reserved `state:loom_bytecode:` and `state:loom_state:` key prefixes are now active for persisting loom bytecodes and contract state. SCHEMA_VERSION bumped from 5 to 6.

### New RPC Endpoints

Five new RPC methods:
- `norn_uploadLoomBytecode` — upload .wasm bytecode and initialize (auth required)
- `norn_executeLoom` — execute a contract with input data (auth required)
- `norn_queryLoom` — read-only contract query (no auth)
- `norn_joinLoom` — join a loom as participant (auth required)
- `norn_leaveLoom` — leave a loom (auth required)

`norn_queryLoom` added to the unauthenticated whitelist. Total RPC endpoints: 35.

### New Wallet Commands

Five new wallet subcommands: `upload-bytecode`, `execute-loom`, `query-loom`, `join-loom`, `leave-loom` (41 total).

### LoomManager Node Integration

`LoomManager` from `norn-loom` is now wired into the node with `Arc<RwLock<LoomManager>>`. On startup, the node restores loom metadata, bytecodes, and contract state from persistent storage. Execution results are persisted via write-through to the state store.

## Changelog: v0.12.0 (SDK v3 — Developer Experience)

### Upgrading from v0.11.x to v0.12.0

**No `--reset-state` required.** PROTOCOL_VERSION and SCHEMA_VERSION are unchanged. Simply pull, build, and restart.

```bash
git pull && cargo build --release
# Restart your node
```

### SDK Changes

SDK v3 adds typed storage primitives, a Response builder, guard macros, and a native test harness:

- **`Item<T>` / `Map<K, V>`** — Type-safe storage with `save()`, `load()`, `load_or()`, `remove()`
- **`Response` builder** — `Response::new().add_attribute("k", "v").set_data(&val)`
- **`ContractResult`** changed from `Result<Vec<u8>, ContractError>` to `Result<Response, ContractError>`
- **Guard macros** — `ensure!`, `ensure_eq!`, `ensure_ne!` for concise validation
- **`TestEnv`** — Native test harness with mock contexts (no Wasm runtime needed for testing)
- **`addr` module** — `addr_to_hex()`, `hex_to_addr()`, `ZERO_ADDRESS`

### Contract Migration

Existing v0.11.x contracts continue to work unchanged. To adopt v3 features:

1. Replace raw `host::state_set` / `host::state_get` with `Item<T>` / `Map<K, V>`
2. Return `Response` from execute/query instead of raw bytes
3. Use `ensure!` macros instead of manual `if` checks
4. Add native tests with `TestEnv`

## Changelog: v0.13.0 (SDK v4 — Solidity Parity)

### Upgrading from v0.12.x to v0.13.0

**No `--reset-state` required.** PROTOCOL_VERSION and SCHEMA_VERSION are unchanged. Simply pull, build, and restart.

```bash
git pull && cargo build --release
# Restart your node
```

### Breaking SDK Changes

The `Contract` trait signature changed:

```rust
// v0.12.0
impl Contract for MyContract {
    type Exec = Execute;
    type Query = Query;
    fn init(ctx: &Context) -> Self { ... }
}

// v0.13.0
impl Contract for MyContract {
    type Init = Empty;       // NEW: required associated type
    type Exec = Execute;
    type Query = Query;
    fn init(ctx: &Context, _msg: Empty) -> Self { ... }  // NEW: second parameter
}
```

**Migration steps for existing contracts:**

1. Add `type Init = Empty;` to your `Contract` impl
2. Change `fn init(ctx: &Context) -> Self` to `fn init(ctx: &Context, _msg: Empty) -> Self`
3. Update test calls: `MyContract::init(&env.ctx())` → `MyContract::init(&env.ctx(), Empty)`
4. `Empty` is available from `use norn_sdk::prelude::*;`

### New Features

- **Typed InitMsg** — Contracts can define constructor parameters via `type Init = MyInitMsg`
- **Structured Events** — `Event::new("Transfer").add_attribute("from", hex)` with `Response::add_event()`
- **Standard Library** — `Ownable`, `Pausable`, `Norn20` (ERC20-equivalent) composable modules
- **IndexedMap** — Iterable map with `keys()`, `range()`, `len()` — uses client-side key index
- **Runtime fixes** — `execute_loom` RPC now returns real gas, logs, events, and applies pending transfers
- **Output buffer** — Bumped from 4KB to 16KB

### Scaffolding

`norn wallet new-loom` now generates v0.13.0 templates with `type Init = Empty`.

## Changelog: v0.14.0 (SDK v5 — Proc-Macro DX Overhaul)

### Upgrading from v0.13.x to v0.14.0

**No `--reset-state` required.** PROTOCOL_VERSION and SCHEMA_VERSION are unchanged. Simply pull, build, and restart.

```bash
git pull && cargo build --release
# Restart your node
```

### No Breaking Changes

The `#[norn_contract]` proc macro is purely additive. Existing contracts using the manual `Contract` trait + `norn_entry!` approach continue to work without changes. You can migrate at your own pace.

### Migration (optional)

To adopt `#[norn_contract]` on an existing contract:

1. Replace `#[derive(BorshSerialize, BorshDeserialize)]` on the contract struct with `#[norn_contract]`
2. Remove the manual `Execute` and `Query` enums
3. Remove the `impl Contract for MyContract { ... }` block
4. Remove the `norn_entry!(MyContract);` call
5. Add `#[norn_contract]` on the impl block
6. Annotate methods: `#[init]`, `#[execute]`, `#[query]`
7. Update tests: `counter.execute(&ctx, Execute::Increment)` → `counter.increment(&ctx)`

```rust
// v0.13.0 — manual approach
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Counter { value: u64 }

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Execute { Increment }

impl Contract for Counter {
    type Init = Empty;
    type Exec = Execute;
    type Query = Query;
    fn init(_ctx: &Context, _msg: Empty) -> Self { Counter { value: 0 } }
    fn execute(&mut self, _ctx: &Context, msg: Execute) -> ContractResult {
        match msg { Execute::Increment => { self.value += 1; ok(self.value) } }
    }
    // ...
}
norn_entry!(Counter);

// v0.14.0 — proc macro approach
#[norn_contract]
pub struct Counter { value: u64 }

#[norn_contract]
impl Counter {
    #[init]
    pub fn new(_ctx: &Context) -> Self { Counter { value: 0 } }

    #[execute]
    pub fn increment(&mut self, _ctx: &Context) -> ContractResult {
        self.value += 1; ok(self.value)
    }
    // ...
}
```

### New Features

- **`#[norn_contract]` proc macro** — eliminates ~60% of ceremony (borsh derives, enum definitions, match dispatch, norn_entry!)
- **Direct method calls in tests** — `counter.increment(&ctx)` instead of `counter.execute(&ctx, Execute::Increment)`
- **Typed init params** — extra params after `&Context` in `#[init]` automatically become a generated init struct
- **Reference parameter handling** — `&T` method params automatically store `T` in enums, pass `&var` in dispatch
- **Coin example** — new minimal token contract mirroring Solidity's intro Coin example
- **norn-sdk-macros crate** — new workspace member providing the proc macro

### Scaffolding

`norn wallet new-loom` now generates v0.14.0 templates with `#[norn_contract]` syntax.

---

## Upgrading from v0.14.x to v0.15.0

**P2P and state compatible** — SDK-only changes, no protocol or schema bump required.

1. Restart your node with the new binary. No `--reset-state` needed.
2. Update contract crate dependencies: `norn-sdk` tag `v0.14.0` → `v0.15.0`.

### Migration Examples

#### Composite Map keys with tuples (replaces hand-rolled key builders)

```rust
// Before (v0.14.0):
const ALLOWANCES: Map<[u8; 40], u128> = Map::new("allow");
fn allowance_key(owner: &Address, spender: &Address) -> [u8; 40] {
    let mut key = [0u8; 40];
    key[..20].copy_from_slice(owner);
    key[20..].copy_from_slice(spender);
    key
}
ALLOWANCES.load(&allowance_key(&owner, &spender));

// After (v0.15.0):
const ALLOWANCES: Map<(Address, Address), u128> = Map::new("allow");
ALLOWANCES.load(&(owner, spender));
```

**Note**: Do NOT change storage keys in deployed contracts — tuple keys produce the same byte layout as the manual approach for fixed-width types like `Address`, but only for new contracts.

#### Safe math

```rust
// Before:
let new_bal = bal.checked_add(amount).ok_or(ContractError::Overflow)?;

// After:
let new_bal = safe_add(bal, amount)?;
// Also: safe_sub(a, b) returns InsufficientFunds, safe_mul(a, b) returns Overflow
```

#### Response helpers

```rust
// Before:
Ok(Response::new()
    .add_attribute("action", "mint")
    .add_attribute("to", addr_to_hex(&to))
    .add_attribute("amount", format!("{amount}")))

// After:
Ok(Response::with_action("mint")
    .add_address("to", &to)
    .add_u128("amount", amount))
```

#### event! macro

```rust
// Before:
Event::new("Transfer")
    .add_attribute("from", addr_to_hex(&sender))
    .add_attribute("to", addr_to_hex(&to))
    .add_attribute("amount", format!("{amount}"))

// After:
event!("Transfer", from: sender, to: to, amount: amount)
```

#### Item::init() / Map::init() (panicking save for initialization)

```rust
// Before:
OWNER.save(&ctx.sender()).unwrap();

// After:
OWNER.init(&ctx.sender());
```

#### Test helpers

```rust
// Before:
const ALICE: Address = [1u8; 20];
const BOB: Address = [2u8; 20];
let bal: u128 = from_response(&resp).unwrap();
assert_eq!(bal, 42);
assert!(matches!(err, ContractError::Unauthorized));

// After (ALICE, BOB, CHARLIE, DAVE are in norn_sdk::testing::*):
assert_data::<u128>(&resp, &42);
assert_eq!(err, ContractError::Unauthorized); // PartialEq on ContractError
```

#### Map::update_or() / Item::update_or()

```rust
// Load-or-default + transform + save in one call:
BALANCES.update_or(&addr, 0, |bal| Ok(bal + amount))?;
```

#### Display on ContractError

```rust
// Before:
let msg = err.message();
log::error!("contract failed: {msg}");

// After:
log::error!("contract failed: {err}");
```

#### u64 safe math

```rust
// Before:
let new_val = val.checked_add(1).ok_or(ContractError::Overflow)?;

// After:
let new_val = safe_add_u64(val, 1)?;
// Also: safe_sub_u64, safe_mul_u64
```

#### Response::merge() for stdlib composability

```rust
// Compose stdlib response with your own attributes:
let stdlib_resp = Norn20::mint(&to, amount)?;
Ok(Response::with_action("mint_tokens").merge(stdlib_resp))
// Result: your "action" attribute + stdlib's Mint event + data
```

### New Features

- **Tuple `StorageKey`** — `Map<(Address, Address), u128>` eliminates hand-rolled composite key functions
- **`safe_add`/`safe_sub`/`safe_mul`** — checked arithmetic returning `ContractError`
- **`Response::with_action()`** — creates response with "action" attribute pre-set
- **`Response::add_address()`/`add_u128()`** — auto-converting attribute builders
- **`Event::add_address()`/`add_u128()`** — same on events
- **`event!` macro** — `event!("Transfer", from: sender, to: to, amount: amount)`
- **`ToAttributeValue` trait** — compile-time type dispatch for attribute values
- **`Item::init()`/`Map::init()`** — panicking save for contract initialization
- **`Item::update_or()`/`Map::update_or()`** — update with default value
- **`PartialEq` on `ContractError`** — enables `assert_eq!(err, ContractError::Unauthorized)`
- **`Display` on `ContractError`** — enables `format!("{err}")` and `{err}` in format strings
- **`safe_add_u64`/`safe_sub_u64`/`safe_mul_u64`** — u64 checked arithmetic variants
- **`Response::merge(other)`** — combine attributes, events, and data from another response (stdlib composability)
- **Test address constants** — `ALICE`, `BOB`, `CHARLIE`, `DAVE` in `norn_sdk::testing`
- **`assert_data::<T>()`** — combines `from_response().unwrap()` + `assert_eq!`
- **`assert_err_contains()`** — checks error message substring
- **`TestEnv::transfers()`/`clear_transfers()`** — access captured mock transfers

### Scaffolding

`norn wallet new-loom` now generates v0.15.0 templates with all SDK v6 features.

## Changelog: v0.16.0 (Multi-Validator Consensus, Staking & Slashing)

### Upgrading from v0.15.x to v0.16.0

This is a **breaking upgrade**. PROTOCOL_VERSION changed from 6 to 8, and SCHEMA_VERSION changed from 6 to 7.

1. **Stop the node.**

2. **Build the new version:**
   ```bash
   cargo install --path norn-node
   ```

3. **Start with `--reset-state`:**
   ```bash
   norn run --dev --reset-state
   ```

4. **All peers must be running v0.16.0.** New staking-related `NornMessage` variants (discriminant 20) are not supported by older nodes.

### New Features

- **Multi-validator consensus** — HotStuff BFT now supports multiple validators with leader rotation
- **Staking system** — `stake`, `unstake` commands with bonding period enforcement
- **Slashing** — Validators that produce conflicting blocks have their stake slashed
- **State proofs** — Merkle proof generation and verification for account balances
- **Dynamic validator set** — Validators can join and leave the active set based on stake
- **Epoch system** — 1000-block epochs for fee accumulation and validator set rotation

### New Wallet Commands

Three new wallet subcommands: `stake`, `unstake`, `staking-info`.

---

## Changelog: v0.17.0 (WebSocket Subscriptions, Cross-Contract Calls, TypeScript SDK)

### Upgrading from v0.16.x to v0.17.0

This is a **breaking upgrade**. PROTOCOL_VERSION changed from 8 to 9.

1. **Stop the node.**

2. **Build the new version:**
   ```bash
   cargo install --path norn-node
   ```

3. **Start with `--reset-state`:**
   ```bash
   norn run --dev --reset-state
   ```

### New Features

- **WebSocket subscriptions** — Real-time event streaming for blocks, transfers, tokens, looms, and pending transactions via `norn_subscribe*` RPC methods
- **Cross-contract calls** — Looms can call other looms via `norn_call_contract` host function with `CallStack` re-entrancy protection and `MAX_CALL_DEPTH = 8`
- **TypeScript SDK** — `@norn-protocol/sdk` package with `NornClient`, `NornWallet`, `NornSubscriber`, address utilities, and full RPC type coverage (51 tests)
- **RPC broadcasters** — `RpcBroadcasters` struct groups 5 broadcast channels for real-time event distribution

### New RPC Endpoints

- `norn_subscribeBlocks` / `norn_unsubscribeBlocks`
- `norn_subscribeTransfers` / `norn_unsubscribeTransfers`
- `norn_subscribeTokens` / `norn_unsubscribeTokens`
- `norn_subscribeLooms` / `norn_unsubscribeLooms`
- `norn_subscribePendingTransactions` / `norn_unsubscribePendingTransactions`

---

## Changelog: v0.18.0 (Block Explorer, Web Wallet, Token Management)

### Upgrading from v0.17.x to v0.18.0

**No `--reset-state` required.** PROTOCOL_VERSION and SCHEMA_VERSION are unchanged. Simply pull, build, and restart.

```bash
git pull && cargo build --release
# Restart your node
```

### New Features

- **Block explorer** (`explorer/`) — Next.js 15 + shadcn/ui dashboard with real-time block/transaction/token/contract views, address pages, network stats, and WebSocket live updates
- **Web wallet** (`wallet/`) — PWA with mobile-first design, encrypted keystore in IndexedDB, passkey recovery, auto-lock, NornNames, token management, and 11 integrated smart contract apps (governance, escrow, vesting, treasury, timelock, swap, splitter, crowdfund, staking, launchpad, airdrop)
- **Wallet extension** — Chrome extension for signing transactions from web apps
- **Synthetic transfer records** — `log_synthetic_transfer()` creates TransferRecords for non-transfer operations (genesis, fees, mints, burns) for complete transaction history
- **`norn_getBlockTransactions`** — RPC method returning detailed block contents (transfers, token ops, names, looms)
- **Address labels** — Known addresses (genesis, faucet) get human-readable labels in RPC responses
- **Website** (`website/`) — Marketing site at norn.network with documentation, architecture guides, and tokenomics

### Notable Fixes (v0.18.1 — v0.18.4)

- Faucet credit propagation through blocks to all nodes
- Token symbol display in RPC responses and explorer
- Change-password command for wallet CLI
- Improved Merkle tree implementation
- Block production timing metrics

---

## Changelog: v0.19.0 (Validator Rewards, Wallet UX, Contracts Page)

### Upgrading from v0.18.x to v0.19.0

**State reset recommended.** While PROTOCOL_VERSION and SCHEMA_VERSION are unchanged, the fee economics changed: fees are now redistributed to validators instead of being burned. Existing state will have inconsistent supply accounting.

```bash
cargo install --path norn-node
norn run --dev --reset-state
```

### New Features

- **Validator reward distribution** — Commitment fees collected each epoch (1000 blocks) are now distributed to validators proportional to their stake, replacing the previous fee-burn model
- **`norn wallet rewards`** — New CLI command showing pending epoch fees, blocks until next distribution, and per-validator projected rewards
- **`norn_getValidatorRewards`** — New RPC endpoint returning epoch reward information
- **Wallet UX polish** — `FormButton` component with `disabledReason` hints, `FieldError` inline validation, improved disabled button styling (`opacity-30`, `cursor-not-allowed`)
- **Contracts page** — Browse/Interact tabs with deployed contract discovery, input validation, structured result display with events and logs
- **Node setup script** — `scripts/setup-node.sh` one-command installer for new node operators

### Economic Changes

- `debit_fee()` no longer reduces `total_supply_cache` — fees are transferred to validators, not burned
- `compute_reward_distribution()` in `norn-weave/src/fees.rs` — proportional distribution with dust remainder to highest-staked validator
- Epoch boundary trigger in `WeaveEngine` — rewards computed and applied every 1000 blocks

### Wallet UX Changes (All 20 Form Pages)

- Submit buttons show contextual hints when disabled (e.g., "Enter a governance name")
- Address inputs show inline validation errors for invalid hex
- Invalid inputs get red border highlighting

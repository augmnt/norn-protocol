# Norn Protocol Upgrade Guide

## Version Compatibility Matrix

| From Version | To Version | P2P Compatible | State Compatible | Action Required |
|-------------|------------|----------------|------------------|-----------------|
| v0.3.0      | v0.5.x     | No             | No               | `--reset-state`, re-register names |
| v0.4.1      | v0.5.x     | No             | No               | `--reset-state`, re-register names |
| v0.5.0      | v0.5.1+    | Yes*           | Yes*             | Restart node |
| v0.7.x      | v0.8.0     | No             | No               | `--reset-state` (PROTOCOL_VERSION 4→5, SCHEMA_VERSION 3→4) |
| v0.8.0      | v0.8.1     | Yes            | Yes              | Restart node (CLI-only changes, no protocol/schema bump) |

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
- Commitment history

## Version Constants

The protocol uses three version constants to detect incompatibilities:

| Constant | Location | Current | Purpose |
|----------|----------|---------|---------|
| `PROTOCOL_VERSION` | `norn-relay/src/protocol.rs` | 5 | P2P wire format version. Mismatch = messages rejected. |
| `SCHEMA_VERSION` | `norn-node/src/state_store.rs` | 4 | Borsh state schema version. Mismatch = node refuses to start (suggests `--reset-state`). |
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

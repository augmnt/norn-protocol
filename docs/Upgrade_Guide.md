# Norn Protocol Upgrade Guide

## Version Compatibility Matrix

| From Version | To Version | P2P Compatible | State Compatible | Action Required |
|-------------|------------|----------------|------------------|-----------------|
| v0.3.0      | v0.5.x     | No             | No               | `--reset-state`, re-register names |
| v0.4.1      | v0.5.x     | No             | No               | `--reset-state`, re-register names |
| v0.5.0      | v0.5.1+    | Yes*           | Yes*             | Restart node |

\* Within the v0.5.x line, compatibility depends on whether PROTOCOL_VERSION or SCHEMA_VERSION was bumped. Check the release notes.

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

### What's Safe Across Upgrades

- **Wallet files** (`~/.norn/wallets/*.json`): Encrypted keystores are version-independent. Your private keys, addresses, and wallet passwords remain valid across any upgrade.
- **Genesis config files** (`genesis.json`): Forward-compatible via `serde(default)` on the `version` field. Old genesis files work with new nodes.

### What Gets Wiped by `--reset-state`

- Block history
- Thread registrations
- Account balances (re-funded via genesis allocations or faucet)
- NornNames (must re-register)
- Commitment history

## Version Constants

The protocol uses three version constants to detect incompatibilities:

| Constant | Location | Current | Purpose |
|----------|----------|---------|---------|
| `PROTOCOL_VERSION` | `norn-relay/src/protocol.rs` | 2 | P2P wire format version. Mismatch = messages rejected. |
| `SCHEMA_VERSION` | `norn-node/src/state_store.rs` | 2 | Borsh state schema version. Mismatch = node refuses to start (suggests `--reset-state`). |
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
- **Rolling upgrades** are not supported. All nodes must upgrade simultaneously.
- **`MessageEnvelope`** is available for forward-compatible message dispatch but is not yet wired into the default codec.
- **Binary version negotiation** between peers is not implemented; version checks are per-message only.

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

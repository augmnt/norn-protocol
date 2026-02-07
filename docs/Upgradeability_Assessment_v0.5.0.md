# Norn Upgradeability — Post-Implementation Assessment (v0.5.0)

## Context

All 5 upgradeability priorities were implemented and shipped as v0.5.0. This document answers: "How easy is it to upgrade Norn now?"

## Current State (After v0.5.0)

| Layer | Upgrade-Safe? | What Happens on Breaking Change |
|-------|:---:|-----|
| **Config files** (TOML + serde) | Yes | `#[serde(default)]` handles new fields — no action needed |
| **RPC API** (JSON + serde) | Mostly | Adding methods/fields is safe; removing breaks clients |
| **Persisted state** (borsh in KvStore) | **Detected** | `SCHEMA_VERSION` check on boot → clear error + `--reset-state` |
| **P2P messages** (borsh over libp2p) | **Detected** | `PROTOCOL_VERSION` byte checked on receive → clear disconnect log |
| **Genesis config** (borsh-hashed) | **Explicit** | `version` field in `GenesisConfig` → intentional genesis hash changes |
| **New message types** | **Forward-compatible** | `MessageEnvelope` lets nodes skip unknown types instead of crashing |

## Version Constants

| Constant | Location | Current Value |
|----------|----------|:---:|
| `PROTOCOL_VERSION` | `norn-relay/src/protocol.rs` | 1 |
| `SCHEMA_VERSION` | `norn-node/src/state_store.rs` | 1 |
| `GENESIS_CONFIG_VERSION` | `norn-types/src/genesis.rs` | 1 |

## The Upgrade Workflow for Devnet Operators

### Scenario 1: Non-breaking change (new RPC method, config field, internal refactor)
- Just deploy the new binary. No coordination needed.

### Scenario 2: Breaking P2P change (add/modify NornMessage variant or borsh struct)
1. Bump `PROTOCOL_VERSION` in `norn-relay/src/protocol.rs`
2. Bump `SCHEMA_VERSION` in `norn-node/src/state_store.rs`
3. Deploy new binary
4. Old nodes see: `"protocol version mismatch: peer sent v2, we run v1 — disconnecting"`
5. Old nodes trying to restart see: `"state store schema version mismatch: store is v1, binary expects v2 — run with --reset-state to wipe and restart"`
6. Operator runs: `norn run --dev --reset-state` → clean start

### Scenario 3: Genesis config change
1. Bump `GENESIS_CONFIG_VERSION` in `norn-types/src/genesis.rs`
2. Genesis hash changes deterministically → P2P peers on old genesis get rejected with clear log
3. All nodes must `--reset-state` to join the new chain

## What's Easy Now vs. Before

| Before v0.5.0 | After v0.5.0 |
|---|---|
| Silent deserialization panics | Clear version mismatch errors with actionable messages |
| Manual data dir hunting to reset | `norn run --dev --reset-state` one-liner |
| No way to tell if peers are compatible | Version byte checked on every P2P message |
| Genesis hash changes silently split network | Explicit version field makes splits intentional |
| New message types crash old nodes | `MessageEnvelope` available for forward-compatible dispatch |

## Implementation Details

### P2P Wire Format
```
[4-byte BE length][1-byte PROTOCOL_VERSION][borsh payload]
```
- Version byte is checked on every `decode_message()` call
- Mismatch returns `RelayError::VersionMismatch { peer, ours }`
- Async stream functions (`read_length_prefixed_message`, `write_length_prefixed_message`) also enforce versioning

### Schema Version Check
- `StateStore::check_schema_version()` runs on boot
- Reads persisted version from `__schema_version` key
- Legacy stores (no version key) are treated as version 0 and auto-upgraded
- Version mismatch produces a clear error directing the operator to `--reset-state`

### Genesis Hash Validation
- `StateRequest` and `StateResponse` carry `genesis_hash: [u8; 32]`
- `compute_genesis_hash()` includes `config.version` as the first field in the hash
- Mismatched genesis hashes are rejected with a warning log showing both hashes in hex

### MessageEnvelope
- Versioned wrapper: `{ version: u8, message_type: u8, payload: Vec<u8> }`
- `NornMessage::discriminant()` provides stable u8 for each variant (0–12)
- `MessageEnvelope::unwrap_message()` returns `Option<NornMessage>` — `None` for unknown types
- Not yet wired into the codec (future step when new message types need to coexist)

### --reset-state Flag
- `norn run --dev --reset-state` wipes the data directory before starting
- Logs a warning with the data directory path before removal
- Safe no-op if the directory doesn't exist

## What's Still Manual (Acceptable for Devnet)

- **No automatic state migration** — breaking schema changes require `--reset-state` (full wipe). This is fine for devnet; migrations are a mainnet concern.
- **No rolling upgrades** — all nodes must upgrade roughly together for breaking changes. Acceptable with a small operator set.
- **MessageEnvelope not yet wired into the codec** — the envelope type exists and is tested, but the codec still sends raw `NornMessage`. Wiring it in is a future step when new message types need to coexist with old nodes.
- **No binary version negotiation** — nodes check protocol version but don't exchange capability sets. Not needed until the protocol stabilizes.

## Bottom Line

For devnet iteration, Norn is now in good shape. Breaking changes go from "mysterious crash, hunt for data dirs, coordinate on Discord" to "clear error message, run `--reset-state`, back online in seconds." The version constants (`PROTOCOL_VERSION`, `SCHEMA_VERSION`, `GENESIS_CONFIG_VERSION`) give operators and developers a single, obvious place to bump when shipping breaking changes.

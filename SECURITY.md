# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in the Norn Protocol, please report it responsibly.

**Do NOT open a public GitHub issue for security vulnerabilities.**

Instead, please email security concerns to: **security@norn.to**

### What to include

- Description of the vulnerability
- Steps to reproduce
- Potential impact assessment
- Suggested fix (if you have one)

### Response timeline

- **Acknowledgment**: Within 48 hours
- **Initial assessment**: Within 1 week
- **Fix and disclosure**: Coordinated with reporter, typically within 30 days

### Scope

The following are in scope:

- Consensus safety violations (double-spend, equivocation bypass)
- Cryptographic weaknesses (key derivation, signature verification, hash collisions)
- Fraud proof bypass or suppression
- Wallet keystore vulnerabilities (key extraction, password bypass)
- Denial of service against validator nodes
- P2P network attacks (eclipse, partition)

### Out of scope

- Testnet-only functionality (faucet endpoint, dev mode)
- Social engineering attacks
- Issues in dependencies (report upstream, but let us know)

### Recognition

We appreciate responsible disclosure. Security researchers who report valid vulnerabilities will be acknowledged in release notes (unless anonymity is preferred).

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.3.x   | Yes       |
| 0.2.x   | Yes       |
| 0.1.x   | No        |

## Security Design

Norn's security model is documented in the [Protocol Specification](docs/Norn_Protocol_Specification_v2.0.md) and [White Paper](docs/Norn_Protocol_White_Paper.md). Key properties:

- **Ed25519 signatures** (via `ed25519-dalek`) for all authentication
- **BLAKE3** for hashing and key derivation
- **Argon2id** with per-wallet random salt for keystore encryption
- **XChaCha20-Poly1305** for authenticated encryption
- **Zero `unsafe` blocks** across the entire codebase
- **Fraud proofs** as the economic security mechanism

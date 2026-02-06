//! Cryptographic primitives for the Norn Protocol.
//!
//! Provides Ed25519 signatures, BLAKE3 hashing, Merkle trees, BIP-39 mnemonic
//! generation, SLIP-0010 HD key derivation, XChaCha20-Poly1305 authenticated
//! encryption, and Shamir's Secret Sharing.

pub mod address;
pub mod encryption;
pub mod hash;
pub mod hd;
pub mod keys;
pub mod merkle;
pub mod seed;
pub mod shamir;

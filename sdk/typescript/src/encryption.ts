import { xchacha20poly1305 } from "@noble/ciphers/chacha.js";
import { randomBytes } from "@noble/ciphers/utils.js";
import { x25519 } from "@noble/curves/ed25519.js";
import { blake3Kdf } from "./crypto.js";

/**
 * Derive an X25519 static secret from an Ed25519 private key using BLAKE3 KDF.
 * Matches norn-crypto/src/encryption.rs keypair_to_x25519_secret().
 */
export function ed25519ToX25519Secret(ed25519PrivateKey: Uint8Array): Uint8Array {
  return blake3Kdf("norn-ed25519-to-x25519", ed25519PrivateKey);
}

/**
 * Derive an X25519 public key from an Ed25519 private key.
 * First derives the X25519 secret, then computes the public key via scalar base multiplication.
 */
export function ed25519ToX25519Public(ed25519PrivateKey: Uint8Array): Uint8Array {
  const secret = ed25519ToX25519Secret(ed25519PrivateKey);
  return x25519.scalarMultBase(secret);
}

/** Result of an asymmetric encryption operation. */
export interface EncryptedMessage {
  /** Ephemeral X25519 public key (32 bytes). */
  ephemeralPubkey: Uint8Array;
  /** XChaCha20-Poly1305 nonce (24 bytes). */
  nonce: Uint8Array;
  /** Encrypted ciphertext with authentication tag. */
  ciphertext: Uint8Array;
}

/**
 * Encrypt a plaintext message for a recipient using ephemeral ECDH + XChaCha20-Poly1305.
 * Matches norn-crypto/src/encryption.rs encrypt().
 */
export function encrypt(
  recipientX25519Public: Uint8Array,
  plaintext: Uint8Array,
): EncryptedMessage {
  // Generate ephemeral X25519 keypair
  const ephemeralSecret = randomBytes(32);
  const ephemeralPubkey = x25519.scalarMultBase(ephemeralSecret);

  // Diffie-Hellman shared secret
  const sharedSecret = x25519.scalarMult(ephemeralSecret, recipientX25519Public);

  // Derive encryption key from shared secret using BLAKE3 KDF
  const encryptionKey = blake3Kdf("norn-encryption-key", sharedSecret);

  // Generate random 24-byte nonce
  const nonce = randomBytes(24);

  // Encrypt with XChaCha20-Poly1305
  const cipher = xchacha20poly1305(encryptionKey, nonce);
  const ciphertext = cipher.encrypt(plaintext);

  return { ephemeralPubkey, nonce, ciphertext };
}

/**
 * Decrypt a message using the recipient's Ed25519 private key.
 * Matches norn-crypto/src/encryption.rs decrypt().
 */
export function decrypt(
  ed25519PrivateKey: Uint8Array,
  ephemeralPubkey: Uint8Array,
  nonce: Uint8Array,
  ciphertext: Uint8Array,
): Uint8Array {
  // Derive X25519 secret from Ed25519 private key
  const x25519Secret = ed25519ToX25519Secret(ed25519PrivateKey);

  // Diffie-Hellman shared secret
  const sharedSecret = x25519.scalarMult(x25519Secret, ephemeralPubkey);

  // Derive encryption key
  const encryptionKey = blake3Kdf("norn-encryption-key", sharedSecret);

  // Decrypt with XChaCha20-Poly1305
  const cipher = xchacha20poly1305(encryptionKey, nonce);
  return cipher.decrypt(ciphertext);
}

/**
 * Derive a deterministic shared secret for DMs between two parties.
 * Both parties derive the same key since X25519 DH is commutative.
 */
export function deriveSharedSecret(
  myEd25519PrivateKey: Uint8Array,
  theirX25519Public: Uint8Array,
): Uint8Array {
  const myX25519Secret = ed25519ToX25519Secret(myEd25519PrivateKey);
  const shared = x25519.scalarMult(myX25519Secret, theirX25519Public);
  return blake3Kdf("norn-chat-dm", shared);
}

/** Result of a symmetric encryption operation. */
export interface SymmetricEncryptedMessage {
  /** XChaCha20-Poly1305 nonce (24 bytes). */
  nonce: Uint8Array;
  /** Encrypted ciphertext with authentication tag. */
  ciphertext: Uint8Array;
}

/**
 * Encrypt with a symmetric key using XChaCha20-Poly1305.
 * Used for channel messages and DMs where a shared key is pre-derived.
 */
export function symmetricEncrypt(
  key: Uint8Array,
  plaintext: Uint8Array,
): SymmetricEncryptedMessage {
  const nonce = randomBytes(24);
  const cipher = xchacha20poly1305(key, nonce);
  const ciphertext = cipher.encrypt(plaintext);
  return { nonce, ciphertext };
}

/** Decrypt with a symmetric key using XChaCha20-Poly1305. */
export function symmetricDecrypt(
  key: Uint8Array,
  nonce: Uint8Array,
  ciphertext: Uint8Array,
): Uint8Array {
  const cipher = xchacha20poly1305(key, nonce);
  return cipher.decrypt(ciphertext);
}

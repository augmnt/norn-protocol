import { etc, getPublicKey, sign, verify } from "@noble/ed25519";
import { sha512 } from "@noble/hashes/sha512";
import { blake3 } from "@noble/hashes/blake3";

// ed25519 requires sha512 to be configured for sync operations.
etc.sha512Sync = (...m: Uint8Array[]) => {
  const h = sha512.create();
  for (const chunk of m) h.update(chunk);
  return h.digest();
};

/** Compute a BLAKE3 hash of the input. Returns 32 bytes. */
export function blake3Hash(data: Uint8Array): Uint8Array {
  return blake3(data);
}

/** Sign a message with an Ed25519 private key. Returns 64-byte signature. */
export function ed25519Sign(
  message: Uint8Array,
  privateKey: Uint8Array,
): Uint8Array {
  return sign(message, privateKey);
}

/** Verify an Ed25519 signature. */
export function ed25519Verify(
  signature: Uint8Array,
  message: Uint8Array,
  publicKey: Uint8Array,
): boolean {
  return verify(signature, message, publicKey);
}

/** Derive a public key from a private key. Returns 32 bytes. */
export function publicKeyFromPrivate(privateKey: Uint8Array): Uint8Array {
  return getPublicKey(privateKey);
}

/** Derive a 20-byte address from a 32-byte public key: BLAKE3(pubkey)[12..32]. */
export function publicKeyToAddress(publicKey: Uint8Array): Uint8Array {
  const hash = blake3Hash(publicKey);
  return hash.slice(12, 32);
}

/** Convert a Uint8Array to hex string. */
export function toHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

/** Convert hex string to Uint8Array. Strips optional '0x' prefix. */
export function fromHex(hex: string): Uint8Array {
  const clean = hex.startsWith("0x") ? hex.slice(2) : hex;
  if (clean.length % 2 !== 0) throw new Error("Invalid hex: odd length");
  const bytes = new Uint8Array(clean.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(clean.substring(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

/** Convert an address Uint8Array to a hex string with 0x prefix. */
export function addressToHex(address: Uint8Array): string {
  return "0x" + toHex(address);
}

/** Convert a hex address string (with or without 0x prefix) to a 20-byte Uint8Array. */
export function hexToAddress(hex: string): Uint8Array {
  const bytes = fromHex(hex);
  if (bytes.length !== 20) {
    throw new Error(`Expected 20 bytes, got ${bytes.length}`);
  }
  return bytes;
}

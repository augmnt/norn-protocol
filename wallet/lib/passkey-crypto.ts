"use client";

import {
  blake3Hash,
  publicKeyFromPrivate,
  publicKeyToAddress,
  toHex,
  addressToHex,
} from "@norn-protocol/sdk";

/**
 * Derive a deterministic salt for a given account index.
 * Salt = BLAKE3("norn-wallet-v1-ed25519-seed-{index}")
 */
export function deriveSalt(accountIndex: number): Uint8Array {
  const text = `norn-wallet-v1-ed25519-seed-${accountIndex}`;
  const encoder = new TextEncoder();
  return blake3Hash(encoder.encode(text));
}

interface DerivedKeypair {
  privateKey: Uint8Array;
  publicKey: Uint8Array;
  publicKeyHex: string;
  addressHex: string;
}

/**
 * Derive an Ed25519 keypair from PRF output (32 bytes).
 * The PRF output IS the private key seed.
 */
export function deriveKeypairFromPrf(prfOutput: Uint8Array): DerivedKeypair {
  if (prfOutput.length !== 32) {
    throw new Error(`PRF output must be 32 bytes, got ${prfOutput.length}`);
  }

  const privateKey = prfOutput;
  const publicKey = publicKeyFromPrivate(privateKey);
  const address = publicKeyToAddress(publicKey);

  return {
    privateKey,
    publicKey,
    publicKeyHex: toHex(publicKey),
    addressHex: addressToHex(address),
  };
}

/**
 * Zero out sensitive byte arrays from memory.
 * Call this after signing to minimize key exposure time.
 */
export function zeroBytes(...arrays: Uint8Array[]): void {
  for (const arr of arrays) {
    arr.fill(0);
  }
}

/**
 * Derive address from a private key (hex) without keeping the key.
 */
export function addressFromPrivateKeyHex(hex: string): string {
  const clean = hex.startsWith("0x") ? hex.slice(2) : hex;
  const bytes = new Uint8Array(clean.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(clean.substring(i * 2, i * 2 + 2), 16);
  }
  const pub = publicKeyFromPrivate(bytes);
  const addr = publicKeyToAddress(pub);
  const result = addressToHex(addr);
  zeroBytes(bytes);
  return result;
}

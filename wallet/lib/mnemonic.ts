"use client";

/**
 * BIP-39 mnemonic encoding for optional wallet backup.
 * Uses @scure/bip39 for the standard English wordlist.
 * Encodes 32 bytes of entropy into 24 words.
 *
 * Note: We use BLAKE3 for checksum instead of SHA-256 since this is
 * Norn-specific encoding. Standard BIP-39 tools won't validate these
 * mnemonics, but the wordlist is the same.
 */

import { blake3Hash } from "@norn-protocol/sdk";
import { wordlist } from "@scure/bip39/wordlists/english";

/**
 * Convert 32 bytes of entropy to a 24-word mnemonic.
 */
export async function bytesToMnemonic(entropy: Uint8Array): Promise<string> {
  if (entropy.length !== 32) {
    throw new Error(`Expected 32 bytes of entropy, got ${entropy.length}`);
  }

  // Compute checksum: first byte of BLAKE3 hash
  const hash = blake3Hash(entropy);
  const checksumByte = hash[0];

  // Convert entropy + checksum to bits
  let bits = "";
  for (const byte of entropy) {
    bits += byte.toString(2).padStart(8, "0");
  }
  bits += checksumByte.toString(2).padStart(8, "0");

  // Split into 11-bit groups (264 bits / 11 = 24 words)
  const mnemonicWords: string[] = [];
  for (let i = 0; i < 24; i++) {
    const index = parseInt(bits.slice(i * 11, (i + 1) * 11), 2);
    mnemonicWords.push(wordlist[index]);
  }

  return mnemonicWords.join(" ");
}

/**
 * Convert a 24-word mnemonic back to 32 bytes.
 */
export async function mnemonicToBytes(mnemonic: string): Promise<Uint8Array> {
  const mnemonicWords = mnemonic.trim().toLowerCase().split(/\s+/);

  if (mnemonicWords.length !== 24) {
    throw new Error(`Expected 24 words, got ${mnemonicWords.length}`);
  }

  // Convert words to bits
  let bits = "";
  for (const word of mnemonicWords) {
    const index = wordlist.indexOf(word);
    if (index === -1) {
      throw new Error(`Invalid mnemonic word: "${word}"`);
    }
    bits += index.toString(2).padStart(11, "0");
  }

  // Extract entropy (first 256 bits) and checksum (last 8 bits)
  const entropyBits = bits.slice(0, 256);
  const checksumBits = bits.slice(256, 264);

  const entropy = new Uint8Array(32);
  for (let i = 0; i < 32; i++) {
    entropy[i] = parseInt(entropyBits.slice(i * 8, (i + 1) * 8), 2);
  }

  // Verify checksum
  const hash = blake3Hash(entropy);
  const expectedChecksum = hash[0].toString(2).padStart(8, "0");
  if (checksumBits !== expectedChecksum) {
    throw new Error("Invalid mnemonic checksum");
  }

  return entropy;
}

/**
 * Validate a mnemonic without converting it.
 */
export async function validateMnemonic(mnemonic: string): Promise<boolean> {
  try {
    await mnemonicToBytes(mnemonic);
    return true;
  } catch {
    return false;
  }
}

"use client";

import {
  createChatEvent,
  encryptDmContent,
  decryptDmContent,
  ed25519ToX25519Public,
  toHex,
  fromHex,
  type ChatEvent,
} from "@norn-protocol/sdk";
import { zeroBytes } from "./passkey-crypto";
import { getWalletForSigning, getWalletForSigningWithPassword } from "./wallet-manager";
import type { StoredWalletMeta } from "@/types/passkey";
import type { Wallet } from "@norn-protocol/sdk";

async function getWallet(
  meta: StoredWalletMeta,
  accountIndex: number,
  password?: string
): Promise<Wallet> {
  if (meta.usesPrf) {
    return getWalletForSigning(meta, accountIndex);
  }
  if (!password) {
    throw new Error("Password required for non-PRF wallet");
  }
  return getWalletForSigningWithPassword(meta, password);
}

function cleanupWallet(wallet: Wallet): void {
  zeroBytes(wallet.privateKey);
}

/** Sign a chat event using the wallet's keypair. */
export async function signChatEvent(
  meta: StoredWalletMeta,
  kind: number,
  content: string,
  tags: string[][],
  accountIndex = 0,
  password?: string
): Promise<ChatEvent> {
  const wallet = await getWallet(meta, accountIndex, password);
  try {
    return createChatEvent(wallet.privateKey, kind, content, tags);
  } finally {
    cleanupWallet(wallet);
  }
}

/** Sign an encrypted DM chat event. */
export async function signEncryptedDm(
  meta: StoredWalletMeta,
  recipientX25519Public: Uint8Array,
  plaintext: string,
  recipientPubkey: string,
  accountIndex = 0,
  password?: string
): Promise<ChatEvent> {
  const wallet = await getWallet(meta, accountIndex, password);
  try {
    const { content, nonceTags } = encryptDmContent(
      wallet.privateKey,
      recipientX25519Public,
      plaintext
    );
    const tags: string[][] = [["p", recipientPubkey], ...nonceTags];
    return createChatEvent(wallet.privateKey, 30001, content, tags);
  } finally {
    cleanupWallet(wallet);
  }
}

/** Get the X25519 public key for the wallet (for sharing in profile events). */
export async function getX25519PublicKey(
  meta: StoredWalletMeta,
  accountIndex = 0,
  password?: string
): Promise<string> {
  const wallet = await getWallet(meta, accountIndex, password);
  try {
    return toHex(ed25519ToX25519Public(wallet.privateKey));
  } finally {
    cleanupWallet(wallet);
  }
}

// Cached Ed25519 private key for DM decryption (avoids repeated biometric/password prompts)
let cachedPrivateKey: Uint8Array | null = null;

/** Decrypt an incoming DM message. Caches the private key for the session. */
export async function decryptDmMessage(
  meta: StoredWalletMeta,
  peerX25519PublicHex: string,
  encryptedContent: string,
  nonceHex: string,
  accountIndex = 0,
  password?: string
): Promise<string> {
  if (!cachedPrivateKey) {
    const wallet = await getWallet(meta, accountIndex, password);
    cachedPrivateKey = new Uint8Array(wallet.privateKey);
    cleanupWallet(wallet);
  }
  return decryptDmContent(
    cachedPrivateKey,
    fromHex(peerX25519PublicHex),
    encryptedContent,
    nonceHex
  );
}

/** Clear the cached decryption key. Call on wallet lock. */
export function clearChatKeyCache(): void {
  if (cachedPrivateKey) {
    cachedPrivateKey.fill(0);
    cachedPrivateKey = null;
  }
}

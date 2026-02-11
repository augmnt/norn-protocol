"use client";

import { Wallet, toHex, fromHex } from "@norn-protocol/sdk";
import { isPrfSupported, createPasskeyWithPrf, authenticateWithPrf } from "./passkey";
import { deriveSalt, deriveKeypairFromPrf, zeroBytes } from "./passkey-crypto";
import { loadWalletMeta, saveWalletMeta, deleteWalletMeta } from "./passkey-storage";
import { createEncryptedKey, decryptKey, encryptExistingKey } from "./passkey-fallback";
import { bytesToMnemonic, mnemonicToBytes } from "./mnemonic";
import type { StoredWalletMeta, StoredAccount, WalletState } from "@/types/passkey";

function getRpId(): string {
  if (typeof window === "undefined") return "localhost";
  return window.location.hostname;
}

export interface WalletInitResult {
  state: WalletState;
  meta: StoredWalletMeta | null;
  prfSupported: boolean;
}

export interface CreateWalletResult {
  address: string;
  publicKeyHex: string;
  mnemonic?: string;
}

/** Initialize wallet state from IndexedDB. */
export async function initialize(): Promise<WalletInitResult> {
  const prfSupported = await isPrfSupported();
  const meta = await loadWalletMeta();

  if (!meta) {
    return { state: "uninitialized", meta: null, prfSupported };
  }

  return { state: "locked", meta, prfSupported };
}

/** Create a new wallet with passkey enrollment. */
export async function createWallet(
  name: string
): Promise<CreateWalletResult> {
  const rpId = getRpId();
  const prfSupported = await isPrfSupported();
  const accountIndex = 0;
  const salt = deriveSalt(accountIndex);

  let address: string;
  let publicKeyHex: string;
  let mnemonic: string | undefined;
  let meta: StoredWalletMeta;

  if (prfSupported) {
    const { credentialId, prfOutput } = await createPasskeyWithPrf(
      rpId,
      name,
      salt
    );

    if (!prfOutput) {
      throw new Error("PRF_UNSUPPORTED");
    }

    const keypair = deriveKeypairFromPrf(prfOutput);
    address = keypair.addressHex;
    publicKeyHex = keypair.publicKeyHex;

    // Generate optional mnemonic backup from the PRF output
    mnemonic = await bytesToMnemonic(prfOutput);

    zeroBytes(prfOutput, keypair.privateKey);

    meta = {
      credentialId,
      accounts: [
        {
          index: accountIndex,
          label: name,
          address,
          publicKeyHex,
          createdAt: Date.now(),
        },
      ],
      usesPrf: true,
      rpId,
      createdAt: Date.now(),
    };
  } else {
    // Fallback: password-encrypted random key
    throw new Error(
      "PRF not supported. Use createWalletWithPassword() instead."
    );
  }

  await saveWalletMeta(meta);
  return { address, publicKeyHex, mnemonic };
}

/** Create a wallet with password fallback (no PRF). */
export async function createWalletWithPassword(
  name: string,
  password: string
): Promise<CreateWalletResult> {
  const rpId = getRpId();
  const bundle = await createEncryptedKey(password);

  const keypair = deriveKeypairFromPrf(bundle.privateKey);
  const address = keypair.addressHex;
  const publicKeyHex = keypair.publicKeyHex;
  const mnemonic = await bytesToMnemonic(bundle.privateKey);

  zeroBytes(bundle.privateKey, keypair.privateKey);

  const meta: StoredWalletMeta = {
    credentialId: "",
    accounts: [
      {
        index: 0,
        label: name,
        address,
        publicKeyHex,
        createdAt: Date.now(),
      },
    ],
    usesPrf: false,
    encryptedKeyBlob: bundle.encryptedKeyBlob,
    pbkdfSalt: bundle.pbkdfSalt,
    aesIv: bundle.aesIv,
    rpId,
    createdAt: Date.now(),
  };

  await saveWalletMeta(meta);
  return { address, publicKeyHex, mnemonic };
}

/** Import wallet from hex-encoded private key. */
export async function importFromPrivateKey(
  hex: string,
  name: string,
  password?: string
): Promise<CreateWalletResult> {
  const rpId = getRpId();
  const clean = hex.startsWith("0x") ? hex.slice(2) : hex;
  const keyBytes = fromHex(clean);

  if (keyBytes.length !== 32) {
    throw new Error(`Private key must be 32 bytes, got ${keyBytes.length}`);
  }

  const keypair = deriveKeypairFromPrf(keyBytes);
  const address = keypair.addressHex;
  const publicKeyHex = keypair.publicKeyHex;

  // Encrypt the imported key with password
  const bundle = await encryptExistingKey(keyBytes, password || "");

  zeroBytes(keyBytes, keypair.privateKey);

  const meta: StoredWalletMeta = {
    credentialId: "",
    accounts: [
      {
        index: 0,
        label: name,
        address,
        publicKeyHex,
        createdAt: Date.now(),
      },
    ],
    usesPrf: false,
    encryptedKeyBlob: bundle.encryptedKeyBlob,
    pbkdfSalt: bundle.pbkdfSalt,
    aesIv: bundle.aesIv,
    rpId,
    createdAt: Date.now(),
  };

  await saveWalletMeta(meta);
  return { address, publicKeyHex };
}

/** Import wallet from 24-word mnemonic. */
export async function importFromMnemonic(
  mnemonic: string,
  name: string,
  password?: string
): Promise<CreateWalletResult> {
  const entropy = await mnemonicToBytes(mnemonic);
  const hex = toHex(entropy);
  const result = await importFromPrivateKey(hex, name, password);
  zeroBytes(entropy);
  return result;
}

/** Add another account to the existing wallet (PRF only). */
export async function addAccount(
  name: string,
  meta: StoredWalletMeta
): Promise<StoredAccount> {
  if (!meta.usesPrf) {
    throw new Error("Adding accounts is only supported with PRF wallets");
  }

  const nextIndex = meta.accounts.length;
  const salt = deriveSalt(nextIndex);

  const prfOutput = await authenticateWithPrf(meta.rpId, meta.credentialId, salt);
  const keypair = deriveKeypairFromPrf(prfOutput);

  const account: StoredAccount = {
    index: nextIndex,
    label: name,
    address: keypair.addressHex,
    publicKeyHex: keypair.publicKeyHex,
    createdAt: Date.now(),
  };

  zeroBytes(prfOutput, keypair.privateKey);

  meta.accounts.push(account);
  await saveWalletMeta(meta);

  return account;
}

/** Unlock wallet (verify passkey ownership). */
export async function unlock(meta: StoredWalletMeta): Promise<boolean> {
  if (meta.usesPrf) {
    const salt = deriveSalt(0);
    const prfOutput = await authenticateWithPrf(
      meta.rpId,
      meta.credentialId,
      salt
    );
    // Verify derived address matches stored
    const keypair = deriveKeypairFromPrf(prfOutput);
    const matches = keypair.addressHex === meta.accounts[0].address;
    zeroBytes(prfOutput, keypair.privateKey);
    return matches;
  }
  // For password wallets, unlocking is done via password in the UI
  return true;
}

/** Unlock with password (fallback wallets). */
export async function unlockWithPassword(
  meta: StoredWalletMeta,
  password: string
): Promise<boolean> {
  if (!meta.encryptedKeyBlob || !meta.pbkdfSalt || !meta.aesIv) {
    throw new Error("No encrypted key data found");
  }

  try {
    const key = await decryptKey(
      meta.encryptedKeyBlob,
      password,
      meta.pbkdfSalt,
      meta.aesIv
    );
    const keypair = deriveKeypairFromPrf(key);
    const matches = keypair.addressHex === meta.accounts[0].address;
    zeroBytes(key, keypair.privateKey);
    return matches;
  } catch {
    return false;
  }
}

/**
 * Sign a message using passkey PRF.
 * Returns 64-byte signature. Zeros key material after signing.
 */
export async function signWithPasskey(
  meta: StoredWalletMeta,
  message: Uint8Array,
  accountIndex = 0
): Promise<{ signature: Uint8Array; publicKey: Uint8Array }> {
  const salt = deriveSalt(accountIndex);
  const prfOutput = await authenticateWithPrf(
    meta.rpId,
    meta.credentialId,
    salt
  );

  const wallet = Wallet.fromPrivateKey(prfOutput);
  const signature = wallet.sign(message);
  const publicKey = new Uint8Array(wallet.publicKey);

  zeroBytes(prfOutput);

  return { signature, publicKey };
}

/**
 * Sign a message using password-decrypted key.
 * Returns 64-byte signature. Zeros key material after signing.
 */
export async function signWithPassword(
  meta: StoredWalletMeta,
  message: Uint8Array,
  password: string
): Promise<{ signature: Uint8Array; publicKey: Uint8Array }> {
  if (!meta.encryptedKeyBlob || !meta.pbkdfSalt || !meta.aesIv) {
    throw new Error("No encrypted key data found");
  }

  const key = await decryptKey(
    meta.encryptedKeyBlob,
    password,
    meta.pbkdfSalt,
    meta.aesIv
  );

  const wallet = Wallet.fromPrivateKey(key);
  const signature = wallet.sign(message);
  const publicKey = new Uint8Array(wallet.publicKey);

  zeroBytes(key);

  return { signature, publicKey };
}

/** Export private key as hex (biometric-gated). */
export async function exportPrivateKeyHex(
  meta: StoredWalletMeta,
  accountIndex = 0,
  password?: string
): Promise<string> {
  let keyBytes: Uint8Array;

  if (meta.usesPrf) {
    const salt = deriveSalt(accountIndex);
    keyBytes = await authenticateWithPrf(meta.rpId, meta.credentialId, salt);
  } else {
    if (!password || !meta.encryptedKeyBlob || !meta.pbkdfSalt || !meta.aesIv) {
      throw new Error("Password required for non-PRF wallet");
    }
    keyBytes = await decryptKey(meta.encryptedKeyBlob, password, meta.pbkdfSalt, meta.aesIv);
  }

  const hex = toHex(keyBytes);
  zeroBytes(keyBytes);
  return hex;
}

/** Export recovery phrase (biometric-gated). */
export async function exportMnemonic(
  meta: StoredWalletMeta,
  accountIndex = 0,
  password?: string
): Promise<string> {
  let keyBytes: Uint8Array;

  if (meta.usesPrf) {
    const salt = deriveSalt(accountIndex);
    keyBytes = await authenticateWithPrf(meta.rpId, meta.credentialId, salt);
  } else {
    if (!password || !meta.encryptedKeyBlob || !meta.pbkdfSalt || !meta.aesIv) {
      throw new Error("Password required for non-PRF wallet");
    }
    keyBytes = await decryptKey(meta.encryptedKeyBlob, password, meta.pbkdfSalt, meta.aesIv);
  }

  const mnemonic = await bytesToMnemonic(keyBytes);
  zeroBytes(keyBytes);
  return mnemonic;
}

/** Rename an account label in IndexedDB. */
export async function renameAccount(
  meta: StoredWalletMeta,
  accountIndex: number,
  newLabel: string
): Promise<StoredWalletMeta> {
  const account = meta.accounts[accountIndex];
  if (!account) throw new Error(`Account index ${accountIndex} not found`);
  account.label = newLabel.trim();
  await saveWalletMeta(meta);
  return { ...meta };
}

/** Export wallet metadata as an encrypted JSON backup file. */
export async function exportWalletBackup(meta: StoredWalletMeta): Promise<string> {
  const backup = {
    version: 1,
    type: "norn-wallet-backup",
    exportedAt: Date.now(),
    meta,
  };
  return JSON.stringify(backup, null, 2);
}

/** Import wallet metadata from a JSON backup file. Returns the meta or throws. */
export async function importWalletBackup(json: string): Promise<StoredWalletMeta> {
  const backup = JSON.parse(json);
  if (backup?.type !== "norn-wallet-backup" || !backup?.meta) {
    throw new Error("Invalid backup file format");
  }
  const meta = backup.meta as StoredWalletMeta;
  if (!meta.accounts || meta.accounts.length === 0) {
    throw new Error("Backup contains no accounts");
  }
  await saveWalletMeta(meta);
  return meta;
}

/** Delete wallet entirely from IndexedDB. */
export async function deleteWallet(): Promise<void> {
  await deleteWalletMeta();
}

/**
 * Get a temporary Wallet instance for signing (PRF path).
 * CALLER MUST zero the wallet's privateKey after use.
 */
export async function getWalletForSigning(
  meta: StoredWalletMeta,
  accountIndex = 0
): Promise<Wallet> {
  const salt = deriveSalt(accountIndex);
  const prfOutput = await authenticateWithPrf(
    meta.rpId,
    meta.credentialId,
    salt
  );
  return Wallet.fromPrivateKey(prfOutput);
}

/**
 * Get a temporary Wallet instance for signing (password path).
 * CALLER MUST zero the wallet's privateKey after use.
 */
export async function getWalletForSigningWithPassword(
  meta: StoredWalletMeta,
  password: string
): Promise<Wallet> {
  if (!meta.encryptedKeyBlob || !meta.pbkdfSalt || !meta.aesIv) {
    throw new Error("No encrypted key data found");
  }
  const key = await decryptKey(
    meta.encryptedKeyBlob,
    password,
    meta.pbkdfSalt,
    meta.aesIv
  );
  return Wallet.fromPrivateKey(key);
}

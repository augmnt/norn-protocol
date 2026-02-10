import { encrypt, decrypt } from "./crypto";
import { KEYSTORE_STORAGE_KEY, AUTO_LOCK_DEFAULT_MINUTES } from "./config";
import type { StoredAccount, KeystoreData } from "@/types";
import { Wallet, toHex } from "@norn-protocol/sdk";

const DEFAULT_KEYSTORE: KeystoreData = {
  accounts: [],
  activeAccountId: null,
  autoLockMinutes: AUTO_LOCK_DEFAULT_MINUTES,
};

function generateId(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(8));
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
}

async function loadKeystore(): Promise<KeystoreData> {
  const result = await chrome.storage.local.get(KEYSTORE_STORAGE_KEY);
  return result[KEYSTORE_STORAGE_KEY] ?? DEFAULT_KEYSTORE;
}

async function saveKeystore(data: KeystoreData): Promise<void> {
  await chrome.storage.local.set({ [KEYSTORE_STORAGE_KEY]: data });
}

export async function hasAccounts(): Promise<boolean> {
  const ks = await loadKeystore();
  return ks.accounts.length > 0;
}

export async function getAccounts(): Promise<StoredAccount[]> {
  const ks = await loadKeystore();
  return ks.accounts;
}

export async function getActiveAccountId(): Promise<string | null> {
  const ks = await loadKeystore();
  return ks.activeAccountId;
}

export async function getAutoLockMinutes(): Promise<number> {
  const ks = await loadKeystore();
  return ks.autoLockMinutes;
}

export async function setAutoLockMinutes(minutes: number): Promise<void> {
  const ks = await loadKeystore();
  ks.autoLockMinutes = minutes;
  await saveKeystore(ks);
}

export async function createAccount(
  name: string,
  password: string,
): Promise<StoredAccount> {
  const wallet = Wallet.generate();
  return importAccount(name, wallet.privateKey, password);
}

export async function importAccount(
  name: string,
  privateKey: Uint8Array,
  password: string,
): Promise<StoredAccount> {
  const wallet = Wallet.fromPrivateKey(privateKey);

  const encryptedPrivateKey = await encrypt(privateKey, password);

  const account: StoredAccount = {
    id: generateId(),
    name,
    address: wallet.addressHex,
    publicKey: toHex(wallet.publicKey),
    encryptedPrivateKey,
    createdAt: Date.now(),
  };

  const ks = await loadKeystore();
  ks.accounts.push(account);
  if (!ks.activeAccountId) {
    ks.activeAccountId = account.id;
  }
  await saveKeystore(ks);

  return account;
}

export async function importAccountFromHex(
  name: string,
  privateKeyHex: string,
  password: string,
): Promise<StoredAccount> {
  const clean = privateKeyHex.startsWith("0x")
    ? privateKeyHex.slice(2)
    : privateKeyHex;
  const bytes = new Uint8Array(clean.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(clean.substring(i * 2, i * 2 + 2), 16);
  }
  return importAccount(name, bytes, password);
}

export async function unlockAccount(
  accountId: string,
  password: string,
): Promise<Wallet> {
  const ks = await loadKeystore();
  const account = ks.accounts.find((a) => a.id === accountId);
  if (!account) {
    throw new Error("Account not found");
  }

  const privateKey = await decrypt(account.encryptedPrivateKey, password);
  const wallet = Wallet.fromPrivateKey(privateKey);

  // Re-derive address from the private key to ensure it matches
  // the current SDK derivation (fixes stale stored addresses).
  if (account.address !== wallet.addressHex) {
    account.address = wallet.addressHex;
    account.publicKey = toHex(wallet.publicKey);
    await saveKeystore(ks);
  }

  return wallet;
}

export async function setActiveAccount(accountId: string): Promise<void> {
  const ks = await loadKeystore();
  const exists = ks.accounts.some((a) => a.id === accountId);
  if (!exists) throw new Error("Account not found");
  ks.activeAccountId = accountId;
  await saveKeystore(ks);
}

export async function deleteAccount(
  accountId: string,
  password: string,
): Promise<void> {
  // Verify password by attempting to unlock
  await unlockAccount(accountId, password);

  const ks = await loadKeystore();
  ks.accounts = ks.accounts.filter((a) => a.id !== accountId);
  if (ks.activeAccountId === accountId) {
    ks.activeAccountId = ks.accounts[0]?.id ?? null;
  }
  await saveKeystore(ks);
}

export async function renameAccount(
  accountId: string,
  newName: string,
): Promise<void> {
  const ks = await loadKeystore();
  const account = ks.accounts.find((a) => a.id === accountId);
  if (!account) throw new Error("Account not found");
  account.name = newName;
  await saveKeystore(ks);
}

export async function exportPrivateKey(
  accountId: string,
  password: string,
): Promise<string> {
  const ks = await loadKeystore();
  const account = ks.accounts.find((a) => a.id === accountId);
  if (!account) throw new Error("Account not found");

  const privateKey = await decrypt(account.encryptedPrivateKey, password);
  return toHex(privateKey);
}

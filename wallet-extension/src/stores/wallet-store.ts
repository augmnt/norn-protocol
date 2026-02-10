import { create } from "zustand";
import type { Wallet } from "@norn-protocol/sdk";
import type { StoredAccount } from "@/types";
import {
  hasAccounts,
  getAccounts,
  getActiveAccountId,
  createAccount as ksCreate,
  importAccountFromHex,
  unlockAccount,
  setActiveAccount,
  renameAccount as ksRename,
} from "@/lib/keystore";
import { LOCKED_STORAGE_KEY } from "@/lib/config";

interface WalletState {
  isInitialized: boolean;
  isLocked: boolean;
  accounts: StoredAccount[];
  activeAccountId: string | null;
  activeWallet: Wallet | null;
  password: string | null;

  initialize: () => Promise<{ hasWallet: boolean; isLocked: boolean }>;
  createNewAccount: (name: string, password: string) => Promise<StoredAccount>;
  importExistingAccount: (
    name: string,
    hex: string,
    password: string,
  ) => Promise<StoredAccount>;
  unlock: (password: string) => Promise<void>;
  lock: () => void;
  switchAccount: (accountId: string) => Promise<void>;
  renameAccount: (accountId: string, newName: string) => Promise<void>;
  refreshAccounts: () => Promise<void>;
  getActiveAddress: () => string | null;
}

export const useWalletStore = create<WalletState>((set, get) => ({
  isInitialized: false,
  isLocked: true,
  accounts: [],
  activeAccountId: null,
  activeWallet: null,
  password: null,

  initialize: async () => {
    const has = await hasAccounts();
    const accounts = await getAccounts();
    const activeId = await getActiveAccountId();

    const lockResult = await chrome.storage.local.get(LOCKED_STORAGE_KEY);
    const storedLocked = lockResult[LOCKED_STORAGE_KEY] !== false;

    // If chrome.storage says "unlocked" but we have no activeWallet in memory
    // (popup was closed and reopened), require re-authentication.
    // The in-memory wallet/password is lost when the popup closes.
    const { activeWallet } = get();
    const isLocked = storedLocked || !activeWallet;

    set({
      isInitialized: true,
      accounts,
      activeAccountId: activeId,
      isLocked,
    });

    return { hasWallet: has, isLocked };
  },

  createNewAccount: async (name, password) => {
    const account = await ksCreate(name, password);
    const wallet = await unlockAccount(account.id, password);
    const accounts = await getAccounts();

    await chrome.storage.local.set({ [LOCKED_STORAGE_KEY]: false });
    set({
      accounts,
      activeAccountId: account.id,
      activeWallet: wallet,
      isLocked: false,
      password,
    });

    resetAutoLockAlarm();
    return account;
  },

  importExistingAccount: async (name, hex, password) => {
    const account = await importAccountFromHex(name, hex, password);
    const wallet = await unlockAccount(account.id, password);
    const accounts = await getAccounts();

    await chrome.storage.local.set({ [LOCKED_STORAGE_KEY]: false });
    set({
      accounts,
      activeAccountId: account.id,
      activeWallet: wallet,
      isLocked: false,
      password,
    });

    resetAutoLockAlarm();
    return account;
  },

  unlock: async (password) => {
    const { activeAccountId } = get();
    if (!activeAccountId) throw new Error("No active account");

    const wallet = await unlockAccount(activeAccountId, password);

    await chrome.storage.local.set({ [LOCKED_STORAGE_KEY]: false });
    set({ activeWallet: wallet, isLocked: false, password });
    resetAutoLockAlarm();
  },

  lock: () => {
    chrome.storage.local.set({ [LOCKED_STORAGE_KEY]: true });
    chrome.runtime.sendMessage({ type: "CLEAR_AUTO_LOCK" }).catch(() => {});
    set({ activeWallet: null, isLocked: true, password: null });
  },

  switchAccount: async (accountId) => {
    const { password } = get();
    if (!password) throw new Error("Wallet is locked");

    await setActiveAccount(accountId);
    const wallet = await unlockAccount(accountId, password);
    const accounts = await getAccounts();

    set({ accounts, activeAccountId: accountId, activeWallet: wallet });
  },

  renameAccount: async (accountId, newName) => {
    await ksRename(accountId, newName);
    const accounts = await getAccounts();
    set({ accounts });
  },

  refreshAccounts: async () => {
    const accounts = await getAccounts();
    const activeId = await getActiveAccountId();
    set({ accounts, activeAccountId: activeId });
  },

  getActiveAddress: () => {
    const { accounts, activeAccountId } = get();
    const account = accounts.find((a) => a.id === activeAccountId);
    return account?.address ?? null;
  },
}));

function resetAutoLockAlarm() {
  chrome.runtime
    .sendMessage({ type: "RESET_AUTO_LOCK" })
    .catch(() => {});
}

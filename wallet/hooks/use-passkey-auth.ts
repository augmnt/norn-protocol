"use client";

import { useState, useCallback } from "react";
import { useWalletStore } from "@/stores/wallet-store";
import {
  createWallet,
  createWalletWithPassword,
  importFromPrivateKey,
  importFromMnemonic,
  unlock,
  unlockWithPassword,
  deleteWallet,
  type CreateWalletResult,
} from "@/lib/wallet-manager";
import { loadWalletMeta } from "@/lib/passkey-storage";

export function usePasskeyAuth() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const store = useWalletStore;

  const create = useCallback(
    async (name: string): Promise<CreateWalletResult> => {
      setLoading(true);
      setError(null);
      try {
        const result = await createWallet(name);
        const meta = await loadWalletMeta();
        store.getState().setMeta(meta);
        store.getState().setState("unlocked");
        return result;
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Failed to create wallet";
        setError(msg);
        throw e;
      } finally {
        setLoading(false);
      }
    },
    [store]
  );

  const createWithPassword = useCallback(
    async (name: string, password: string): Promise<CreateWalletResult> => {
      setLoading(true);
      setError(null);
      try {
        const result = await createWalletWithPassword(name, password);
        const meta = await loadWalletMeta();
        store.getState().setMeta(meta);
        store.getState().setState("unlocked");
        return result;
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Failed to create wallet";
        setError(msg);
        throw e;
      } finally {
        setLoading(false);
      }
    },
    [store]
  );

  const importKey = useCallback(
    async (hex: string, name: string, password?: string): Promise<CreateWalletResult> => {
      setLoading(true);
      setError(null);
      try {
        const result = await importFromPrivateKey(hex, name, password);
        const meta = await loadWalletMeta();
        store.getState().setMeta(meta);
        store.getState().setState("unlocked");
        return result;
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Failed to import key";
        setError(msg);
        throw e;
      } finally {
        setLoading(false);
      }
    },
    [store]
  );

  const importMnemonic = useCallback(
    async (mnemonic: string, name: string, password?: string): Promise<CreateWalletResult> => {
      setLoading(true);
      setError(null);
      try {
        const result = await importFromMnemonic(mnemonic, name, password);
        const meta = await loadWalletMeta();
        store.getState().setMeta(meta);
        store.getState().setState("unlocked");
        return result;
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Failed to import mnemonic";
        setError(msg);
        throw e;
      } finally {
        setLoading(false);
      }
    },
    [store]
  );

  const unlockWallet = useCallback(async (): Promise<boolean> => {
    setLoading(true);
    setError(null);
    try {
      const meta = store.getState().meta;
      if (!meta) throw new Error("No wallet found");
      const success = await unlock(meta);
      if (success) {
        store.getState().setState("unlocked");
      } else {
        setError("Failed to verify passkey");
      }
      return success;
    } catch (e) {
      const msg = e instanceof Error ? e.message : "Unlock failed";
      setError(msg);
      return false;
    } finally {
      setLoading(false);
    }
  }, [store]);

  const unlockWithPw = useCallback(
    async (password: string): Promise<boolean> => {
      setLoading(true);
      setError(null);
      try {
        const meta = store.getState().meta;
        if (!meta) throw new Error("No wallet found");
        const success = await unlockWithPassword(meta, password);
        if (success) {
          store.getState().setState("unlocked");
        } else {
          setError("Incorrect password");
        }
        return success;
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Unlock failed";
        setError(msg);
        return false;
      } finally {
        setLoading(false);
      }
    },
    [store]
  );

  const lock = useCallback(() => {
    store.getState().setState("locked");
  }, [store]);

  const deleteAll = useCallback(async () => {
    setLoading(true);
    try {
      await deleteWallet();
      store.getState().reset();
    } finally {
      setLoading(false);
    }
  }, [store]);

  return {
    loading,
    error,
    create,
    createWithPassword,
    importKey,
    importMnemonic,
    unlock: unlockWallet,
    unlockWithPassword: unlockWithPw,
    lock,
    deleteWallet: deleteAll,
  };
}

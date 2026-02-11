"use client";

import { useWalletStore } from "@/stores/wallet-store";

export function useWallet() {
  const state = useWalletStore((s) => s.state);
  const meta = useWalletStore((s) => s.meta);
  const activeAccountIndex = useWalletStore((s) => s.activeAccountIndex);
  const prfSupported = useWalletStore((s) => s.prfSupported);

  const activeAccount = meta?.accounts[activeAccountIndex] ?? null;
  const activeAddress = activeAccount?.address ?? null;
  const accounts = meta?.accounts ?? [];
  const isLocked = state === "locked";
  const isUnlocked = state === "unlocked";
  const isUninitialized = state === "uninitialized";

  return {
    state,
    meta,
    activeAccount,
    activeAddress,
    activeAccountIndex,
    accounts,
    prfSupported,
    isLocked,
    isUnlocked,
    isUninitialized,
  };
}

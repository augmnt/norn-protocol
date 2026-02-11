"use client";

import { create } from "zustand";
import type { StoredAccount, StoredWalletMeta, WalletState } from "@/types/passkey";

interface WalletStoreState {
  state: WalletState;
  meta: StoredWalletMeta | null;
  activeAccountIndex: number;
  prfSupported: boolean;

  // Computed
  activeAccount: StoredAccount | null;
  activeAddress: string | null;

  // Actions
  setState: (state: WalletState) => void;
  setMeta: (meta: StoredWalletMeta | null) => void;
  setActiveAccountIndex: (index: number) => void;
  setPrfSupported: (supported: boolean) => void;
  reset: () => void;
}

export const useWalletStore = create<WalletStoreState>((set, get) => ({
  state: "uninitialized",
  meta: null,
  activeAccountIndex: 0,
  prfSupported: false,

  get activeAccount() {
    const { meta, activeAccountIndex } = get();
    return meta?.accounts[activeAccountIndex] ?? null;
  },

  get activeAddress() {
    const { meta, activeAccountIndex } = get();
    return meta?.accounts[activeAccountIndex]?.address ?? null;
  },

  setState: (state) => set({ state }),
  setMeta: (meta) => set({ meta }),
  setActiveAccountIndex: (index) => {
    const { meta } = get();
    const max = (meta?.accounts.length ?? 1) - 1;
    set({ activeAccountIndex: Math.max(0, Math.min(index, max)) });
  },
  setPrfSupported: (supported) => set({ prfSupported: supported }),
  reset: () =>
    set({
      state: "uninitialized",
      meta: null,
      activeAccountIndex: 0,
    }),
}));

"use client";

import { create } from "zustand";
import type { StoredAccount, StoredWalletMeta, WalletState } from "@/types/passkey";

const SESSION_PW_KEY = "norn-session-pw";
const SESSION_STATE_KEY = "norn-session-state";

function getSessionPassword(): string | null {
  try {
    return sessionStorage.getItem(SESSION_PW_KEY);
  } catch {
    return null;
  }
}

function persistSessionPassword(pw: string | null) {
  try {
    if (pw) {
      sessionStorage.setItem(SESSION_PW_KEY, pw);
    } else {
      sessionStorage.removeItem(SESSION_PW_KEY);
    }
  } catch {
    // sessionStorage unavailable (SSR, private browsing)
  }
}

function getSessionState(): WalletState | null {
  try {
    const v = sessionStorage.getItem(SESSION_STATE_KEY);
    if (v === "unlocked") return "unlocked";
    return null;
  } catch {
    return null;
  }
}

function persistSessionState(state: WalletState) {
  try {
    if (state === "unlocked") {
      sessionStorage.setItem(SESSION_STATE_KEY, "unlocked");
    } else {
      sessionStorage.removeItem(SESSION_STATE_KEY);
    }
  } catch {
    // sessionStorage unavailable
  }
}

interface WalletStoreState {
  state: WalletState;
  meta: StoredWalletMeta | null;
  activeAccountIndex: number;
  prfSupported: boolean;
  sessionPassword: string | null;

  // Computed
  activeAccount: StoredAccount | null;
  activeAddress: string | null;

  // Actions
  setState: (state: WalletState) => void;
  setMeta: (meta: StoredWalletMeta | null) => void;
  setActiveAccountIndex: (index: number) => void;
  setPrfSupported: (supported: boolean) => void;
  setSessionPassword: (password: string | null) => void;
  reset: () => void;
}

export const useWalletStore = create<WalletStoreState>((set, get) => ({
  state: "uninitialized",
  meta: null,
  activeAccountIndex: 0,
  prfSupported: false,
  sessionPassword: null,

  get activeAccount() {
    const { meta, activeAccountIndex } = get();
    return meta?.accounts[activeAccountIndex] ?? null;
  },

  get activeAddress() {
    const { meta, activeAccountIndex } = get();
    return meta?.accounts[activeAccountIndex]?.address ?? null;
  },

  setState: (state) => {
    persistSessionState(state);
    if (state !== "unlocked") {
      // Clear session password when locking or going to uninitialized
      persistSessionPassword(null);
      set({ state, sessionPassword: null });
    } else {
      set({ state });
    }
  },
  setMeta: (meta) => set({ meta }),
  setActiveAccountIndex: (index) => {
    const { meta } = get();
    const max = (meta?.accounts.length ?? 1) - 1;
    set({ activeAccountIndex: Math.max(0, Math.min(index, max)) });
  },
  setPrfSupported: (supported) => set({ prfSupported: supported }),
  setSessionPassword: (password) => {
    persistSessionPassword(password);
    set({ sessionPassword: password });
  },
  reset: () => {
    persistSessionState("uninitialized");
    persistSessionPassword(null);
    set({
      state: "uninitialized",
      meta: null,
      activeAccountIndex: 0,
      sessionPassword: null,
    });
  },
}));

/** Check if we have a saved session that can auto-unlock. */
export function getPersistedSession(): { state: WalletState; password: string | null } | null {
  const sessionState = getSessionState();
  if (sessionState === "unlocked") {
    return { state: "unlocked", password: getSessionPassword() };
  }
  return null;
}

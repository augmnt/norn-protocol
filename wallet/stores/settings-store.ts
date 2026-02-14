"use client";

import { create } from "zustand";
import { persist } from "zustand/middleware";

interface SettingsStoreState {
  autoLockTimeout: number; // milliseconds
  showTestnetWarning: boolean;

  setAutoLockTimeout: (ms: number) => void;
  setShowTestnetWarning: (show: boolean) => void;
}

export const useSettingsStore = create<SettingsStoreState>()(
  persist(
    (set) => ({
      autoLockTimeout: 5 * 60 * 1000, // 5 minutes
      showTestnetWarning: true,

      setAutoLockTimeout: (ms) => {
        // 0 = never lock; otherwise clamp to valid range: 30s–1h
        const clamped = ms === 0 ? 0 : Math.max(30_000, Math.min(ms, 3_600_000));
        set({ autoLockTimeout: clamped });
      },
      setShowTestnetWarning: (show) => set({ showTestnetWarning: show }),
    }),
    {
      name: "norn-wallet-settings",
    }
  )
);

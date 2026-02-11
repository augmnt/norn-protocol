"use client";

import { create } from "zustand";
import { persist } from "zustand/middleware";

interface SavedContract {
  loomId: string;
  label: string;
  savedAt: number;
}

interface SavedContractsState {
  contracts: SavedContract[];
  save: (loomId: string, label: string) => void;
  remove: (loomId: string) => void;
}

export const useSavedContractsStore = create<SavedContractsState>()(
  persist(
    (set) => ({
      contracts: [],
      save: (loomId, label) =>
        set((state) => {
          const normalized = loomId.toLowerCase();
          const existing = state.contracts.findIndex(
            (c) => c.loomId.toLowerCase() === normalized
          );
          if (existing >= 0) {
            const updated = [...state.contracts];
            updated[existing] = { ...updated[existing], label };
            return { contracts: updated };
          }
          return {
            contracts: [...state.contracts, { loomId, label, savedAt: Date.now() }],
          };
        }),
      remove: (loomId) =>
        set((state) => ({
          contracts: state.contracts.filter(
            (c) => c.loomId.toLowerCase() !== loomId.toLowerCase()
          ),
        })),
    }),
    { name: "norn-wallet-saved-contracts" }
  )
);

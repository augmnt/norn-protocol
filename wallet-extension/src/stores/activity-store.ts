import { create } from "zustand";
import type { TransactionHistoryEntry } from "@/types";
import { rpc } from "@/lib/rpc";

interface ActivityState {
  transactions: TransactionHistoryEntry[];
  isLoading: boolean;
  error: string | null;
  fetch: (address: string, limit?: number) => Promise<void>;
  clear: () => void;
}

export const useActivityStore = create<ActivityState>((set) => ({
  transactions: [],
  isLoading: false,
  error: null,

  fetch: async (address, limit = 50) => {
    set({ isLoading: true, error: null });
    try {
      const txs = await rpc.getTransactionHistory(address, limit);
      set({ transactions: txs, isLoading: false });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : "Failed to fetch activity",
        isLoading: false,
      });
    }
  },

  clear: () => set({ transactions: [], error: null }),
}));

"use client";

import { useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { STALE_TIMES } from "@/lib/constants";
import type { TransactionHistoryEntry } from "@/types";

/**
 * Fetches recent global transaction history across all addresses.
 */
export function useRecentHistory(limit = 10, offset = 0) {
  return useQuery({
    queryKey: ["recentHistory", limit, offset],
    queryFn: async (): Promise<TransactionHistoryEntry[]> => {
      try {
        const entries = await rpcCall<TransactionHistoryEntry[]>(
          "norn_getRecentTransfers",
          [limit]
        );
        return entries ?? [];
      } catch {
        return [];
      }
    },
    staleTime: STALE_TIMES.realtime,
    refetchInterval: 5_000,
  });
}

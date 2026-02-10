"use client";

import { useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { STALE_TIMES } from "@/lib/constants";
import type { TransactionHistoryEntry } from "@/types";

/**
 * Fetches recent global transaction history by querying known active addresses.
 * Falls back gracefully if no history exists.
 */
export function useRecentHistory(limit = 10) {
  return useQuery({
    queryKey: ["recentHistory", limit],
    queryFn: async (): Promise<TransactionHistoryEntry[]> => {
      // Try fetching recent history from the transfer log.
      // The RPC requires an address, so we query the devnet founder.
      // On a real network, this would be replaced with a global history endpoint.
      try {
        const entries = await rpcCall<TransactionHistoryEntry[]>(
          "norn_getTransactionHistory",
          // Use empty string or a well-known address; the RPC will return
          // what it has. We try without address filter first.
          ["557dede07828fc8ea66477a6056dbd446a640003", limit, 0]
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

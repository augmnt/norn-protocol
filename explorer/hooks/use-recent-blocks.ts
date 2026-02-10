"use client";

import { useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { STALE_TIMES } from "@/lib/constants";
import type { BlockInfo } from "@/types";

const RECENT_BLOCK_COUNT = 5;

export function useRecentBlocks() {
  return useQuery({
    queryKey: ["recentBlocks"],
    queryFn: async (): Promise<BlockInfo[]> => {
      const latest = await rpcCall<BlockInfo | null>(
        "norn_getLatestBlock",
        [],
      );
      if (!latest) return [];

      const height = latest.height;
      const startHeight = Math.max(0, height - RECENT_BLOCK_COUNT + 1);

      const fetches = [];
      for (let h = height; h >= startHeight; h--) {
        fetches.push(
          rpcCall<BlockInfo | null>("norn_getBlock", [h]).catch(() => null),
        );
      }

      const results = await Promise.all(fetches);
      return results.filter((b): b is BlockInfo => b !== null);
    },
    staleTime: STALE_TIMES.realtime,
    refetchInterval: 5_000,
  });
}

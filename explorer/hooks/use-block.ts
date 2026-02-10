"use client";

import { useQuery } from "@tanstack/react-query";
import { getClient } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES } from "@/lib/constants";

export function useBlock(height: number | undefined) {
  return useQuery({
    queryKey: QUERY_KEYS.block(height!),
    queryFn: () => getClient().getBlock(height!),
    staleTime: STALE_TIMES.immutable,
    enabled: height !== undefined,
  });
}

export function useLatestBlock() {
  return useQuery({
    queryKey: ["latestBlock"],
    queryFn: () => getClient().getLatestBlock(),
    staleTime: STALE_TIMES.realtime,
    refetchInterval: 5_000,
  });
}

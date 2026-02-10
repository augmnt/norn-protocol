"use client";

import { useQuery } from "@tanstack/react-query";
import { getClient } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES } from "@/lib/constants";

export function useWeaveState() {
  return useQuery({
    queryKey: QUERY_KEYS.weaveState,
    queryFn: () => getClient().getWeaveState(),
    staleTime: STALE_TIMES.dynamic,
    refetchInterval: 10_000,
  });
}

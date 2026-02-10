"use client";

import { useQuery } from "@tanstack/react-query";
import { getClient } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES } from "@/lib/constants";

export function useHealth() {
  return useQuery({
    queryKey: QUERY_KEYS.health,
    queryFn: () => getClient().health(),
    staleTime: STALE_TIMES.dynamic,
    refetchInterval: 15_000,
  });
}

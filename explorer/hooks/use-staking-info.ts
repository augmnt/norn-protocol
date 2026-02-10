"use client";

import { useQuery } from "@tanstack/react-query";
import { getClient } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES } from "@/lib/constants";

export function useStakingInfo() {
  return useQuery({
    queryKey: QUERY_KEYS.stakingInfo,
    queryFn: () => getClient().getStakingInfo(),
    staleTime: STALE_TIMES.semiStatic,
  });
}

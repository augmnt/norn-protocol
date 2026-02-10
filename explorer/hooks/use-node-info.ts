"use client";

import { useQuery } from "@tanstack/react-query";
import { getClient } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES } from "@/lib/constants";

export function useNodeInfo() {
  return useQuery({
    queryKey: QUERY_KEYS.nodeInfo,
    queryFn: () => getClient().getNodeInfo(),
    staleTime: STALE_TIMES.semiStatic,
  });
}

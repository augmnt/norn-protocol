"use client";

import { useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES } from "@/lib/constants";

export function useNameRecords(name: string | undefined) {
  return useQuery({
    queryKey: QUERY_KEYS.nameRecords(name!),
    queryFn: () =>
      rpcCall<Record<string, string>>("norn_getNameRecords", [name!]),
    staleTime: STALE_TIMES.semiStatic,
    enabled: !!name,
  });
}

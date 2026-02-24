"use client";

import { useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES } from "@/lib/constants";
import { strip0x } from "@/lib/format";

export function useReverseName(address: string | undefined) {
  return useQuery({
    queryKey: QUERY_KEYS.reverseName(address!),
    queryFn: () =>
      rpcCall<string | null>("norn_reverseName", [strip0x(address!)]),
    staleTime: STALE_TIMES.semiStatic,
    enabled: !!address,
  });
}

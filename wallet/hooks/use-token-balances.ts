"use client";

import { useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES } from "@/lib/constants";
import { strip0x } from "@/lib/format";
import type { ThreadStateInfo } from "@/types";

export function useTokenBalances(address: string | undefined) {
  return useQuery({
    queryKey: QUERY_KEYS.threadState(address!),
    queryFn: () =>
      rpcCall<ThreadStateInfo>("norn_getThreadState", [strip0x(address!)]),
    staleTime: STALE_TIMES.dynamic,
    enabled: !!address,
  });
}

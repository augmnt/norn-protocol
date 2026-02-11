"use client";

import { useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES } from "@/lib/constants";
import { strip0x } from "@/lib/format";
import type { NameInfo } from "@/types";

export function useNames(address: string | undefined) {
  return useQuery({
    queryKey: QUERY_KEYS.names(address!),
    queryFn: () =>
      rpcCall<NameInfo[]>("norn_listNames", [strip0x(address!)]),
    staleTime: STALE_TIMES.semiStatic,
    enabled: !!address,
  });
}

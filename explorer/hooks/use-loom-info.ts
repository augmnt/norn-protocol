"use client";

import { useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES } from "@/lib/constants";
import { strip0x } from "@/lib/format";
import type { LoomInfo } from "@/types";

export function useLoomInfo(loomId: string | undefined) {
  return useQuery({
    queryKey: QUERY_KEYS.loomInfo(loomId!),
    queryFn: () =>
      rpcCall<LoomInfo | null>("norn_getLoomInfo", [strip0x(loomId!)]),
    staleTime: STALE_TIMES.semiStatic,
    enabled: !!loomId,
  });
}

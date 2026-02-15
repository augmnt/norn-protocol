"use client";

import { useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES, PAGE_SIZE } from "@/lib/constants";
import type { LoomInfo } from "@/types";

export function useLoomsList(page: number = 1) {
  return useQuery({
    queryKey: QUERY_KEYS.loomsList(page),
    queryFn: () =>
      rpcCall<LoomInfo[]>("norn_listLooms", [
        PAGE_SIZE,
        (page - 1) * PAGE_SIZE,
      ]),
    staleTime: STALE_TIMES.semiStatic,
  });
}

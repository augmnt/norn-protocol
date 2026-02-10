"use client";

import { useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES, PAGE_SIZE } from "@/lib/constants";
import type { TokenInfo } from "@/types";

export function useTokensList(page: number = 1) {
  return useQuery({
    queryKey: QUERY_KEYS.tokensList(page),
    queryFn: () =>
      rpcCall<TokenInfo[]>("norn_listTokens", [
        PAGE_SIZE,
        (page - 1) * PAGE_SIZE,
      ]),
    staleTime: STALE_TIMES.semiStatic,
  });
}

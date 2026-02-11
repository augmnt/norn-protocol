"use client";

import { useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES } from "@/lib/constants";
import type { TokenInfo } from "@/types";

export function useTokenInfo(tokenId: string | undefined) {
  return useQuery({
    queryKey: QUERY_KEYS.tokenInfo(tokenId!),
    queryFn: () => rpcCall<TokenInfo>("norn_getTokenInfo", [tokenId!]),
    staleTime: STALE_TIMES.semiStatic,
    enabled: !!tokenId,
  });
}

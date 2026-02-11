"use client";

import { useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { STALE_TIMES } from "@/lib/constants";
import type { TokenInfo } from "@/types";

/**
 * Fetches all tokens from the network and filters to those created by the given address.
 * This is necessary because tokens with 0 balance don't appear in thread state,
 * but creators still need to manage (mint/burn) them.
 */
export function useCreatedTokens(address: string | undefined) {
  return useQuery({
    queryKey: ["createdTokens", address],
    queryFn: async () => {
      const all: TokenInfo[] = [];
      let offset = 0;
      const limit = 100;

      // Paginate through all tokens
      while (true) {
        const page = await rpcCall<TokenInfo[]>("norn_listTokens", [limit, offset]);
        if (!page || page.length === 0) break;
        all.push(...page);
        if (page.length < limit) break;
        offset += limit;
      }

      return all.filter(
        (t) => t.creator?.toLowerCase() === address?.toLowerCase()
      );
    },
    staleTime: STALE_TIMES.semiStatic,
    enabled: !!address,
  });
}

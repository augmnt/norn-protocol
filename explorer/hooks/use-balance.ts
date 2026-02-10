"use client";

import { useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES } from "@/lib/constants";
import { config } from "@/lib/config";
import { strip0x } from "@/lib/format";

interface BalanceResult {
  balance: string;
  human_readable: string;
}

export function useBalance(address: string | undefined, tokenId?: string) {
  const effectiveTokenId = tokenId || config.nativeTokenId;

  return useQuery({
    queryKey: QUERY_KEYS.balance(address!, effectiveTokenId),
    queryFn: async (): Promise<BalanceResult> => {
      // RPC returns a plain string (raw amount), not an object
      const raw = await rpcCall<string>("norn_getBalance", [
        strip0x(address!),
        strip0x(effectiveTokenId),
      ]);
      return {
        balance: raw,
        human_readable: raw,
      };
    },
    staleTime: STALE_TIMES.dynamic,
    enabled: !!address,
  });
}

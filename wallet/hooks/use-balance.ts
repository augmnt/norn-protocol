"use client";

import { useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES, NATIVE_TOKEN_ID } from "@/lib/constants";
import { strip0x } from "@/lib/format";

export function useBalance(address: string | undefined, tokenId?: string) {
  const effectiveTokenId = tokenId || NATIVE_TOKEN_ID;

  return useQuery({
    queryKey: QUERY_KEYS.balance(address!, effectiveTokenId),
    queryFn: async () => {
      const raw = await rpcCall<string>("norn_getBalance", [
        strip0x(address!),
        strip0x(effectiveTokenId),
      ]);
      return { balance: raw, human_readable: raw };
    },
    staleTime: STALE_TIMES.realtime,
    refetchInterval: 30_000,
    enabled: !!address,
  });
}

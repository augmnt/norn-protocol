"use client";

import { useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES, PAGE_SIZE } from "@/lib/constants";
import { strip0x } from "@/lib/format";
import type { TransactionHistoryEntry } from "@/types";

export function useTxHistory(address: string | undefined, page: number = 1) {
  return useQuery({
    queryKey: QUERY_KEYS.txHistory(address!, page),
    queryFn: () =>
      rpcCall<TransactionHistoryEntry[]>("norn_getTransactionHistory", [
        strip0x(address!),
        PAGE_SIZE,
        (page - 1) * PAGE_SIZE,
      ]),
    staleTime: STALE_TIMES.dynamic,
    enabled: !!address,
  });
}

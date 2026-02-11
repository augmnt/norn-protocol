"use client";

import { useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES } from "@/lib/constants";
import type { TransactionHistoryEntry } from "@/types";

export function useTransaction(knotId: string) {
  return useQuery({
    queryKey: QUERY_KEYS.transaction(knotId),
    queryFn: () =>
      rpcCall<TransactionHistoryEntry | null>("norn_getTransaction", [knotId]),
    staleTime: STALE_TIMES.immutable,
    enabled: knotId.length > 0,
  });
}

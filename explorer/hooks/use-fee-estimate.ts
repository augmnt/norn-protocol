"use client";

import { useQuery } from "@tanstack/react-query";
import { getClient } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES } from "@/lib/constants";

export function useFeeEstimate() {
  return useQuery({
    queryKey: QUERY_KEYS.feeEstimate,
    queryFn: () => getClient().getFeeEstimate(),
    staleTime: STALE_TIMES.dynamic,
  });
}

"use client";

import { useQuery } from "@tanstack/react-query";
import { getClient } from "@/lib/rpc";
import { QUERY_KEYS, STALE_TIMES } from "@/lib/constants";

export function useValidatorSet() {
  return useQuery({
    queryKey: QUERY_KEYS.validatorSet,
    queryFn: () => getClient().getValidatorSet(),
    staleTime: STALE_TIMES.semiStatic,
  });
}

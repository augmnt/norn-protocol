"use client";

import { useQuery } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { getAppTypeForCodeHash } from "@/lib/code-hash-registry";
import { STALE_TIMES } from "@/lib/constants";
import type { LoomInfo } from "@/types";

/**
 * Fetch all on-chain loom instances that match a given app type.
 * Uses the code_hash field from norn_listLooms to filter by contract type.
 */
export function useAppInstances(appType: string) {
  return useQuery({
    queryKey: ["appInstances", appType],
    queryFn: async () => {
      // Fetch a large batch of looms (up to 200)
      const looms = await rpcCall<LoomInfo[]>("norn_listLooms", [200, 0]);
      return looms.filter(
        (loom) =>
          loom.code_hash && getAppTypeForCodeHash(loom.code_hash) === appType
      );
    },
    staleTime: STALE_TIMES.semiStatic,
  });
}

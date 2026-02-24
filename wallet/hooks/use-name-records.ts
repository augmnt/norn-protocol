"use client";

import { useState, useCallback } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { strip0x } from "@/lib/format";
import { QUERY_KEYS, STALE_TIMES } from "@/lib/constants";
import { useWallet } from "./use-wallet";
import { useSignTransaction } from "./use-sign-transaction";
import type { SubmitResult } from "@/types";

export function useNameRecords(name: string | undefined) {
  return useQuery({
    queryKey: QUERY_KEYS.nameRecords(name!),
    queryFn: () => rpcCall<Record<string, string>>("norn_getNameRecords", [name!]),
    staleTime: STALE_TIMES.semiStatic,
    enabled: !!name,
  });
}

export function useSetNameRecord() {
  const { activeAddress } = useWallet();
  const { signNameRecordUpdate, signing } = useSignTransaction();
  const queryClient = useQueryClient();
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const setNameRecord = useCallback(
    async (name: string, key: string, value: string) => {
      if (!activeAddress) throw new Error("No active address");
      setSubmitting(true);
      setError(null);
      try {
        const hex = await signNameRecordUpdate({ name, key, value });
        const result = await rpcCall<SubmitResult>("norn_setNameRecord", [
          name, key, value,
          strip0x(activeAddress),
          hex,
        ]);
        if (!result.success) throw new Error(result.reason || "Update failed");
        queryClient.invalidateQueries({ queryKey: QUERY_KEYS.nameRecords(name) });
        queryClient.invalidateQueries({ queryKey: QUERY_KEYS.resolveName(name) });
        return result;
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Update failed";
        setError(msg);
        throw e;
      } finally {
        setSubmitting(false);
      }
    },
    [signNameRecordUpdate, activeAddress, queryClient]
  );

  return { setNameRecord, loading: signing || submitting, error };
}

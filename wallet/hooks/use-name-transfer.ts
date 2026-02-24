"use client";

import { useState, useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { strip0x } from "@/lib/format";
import { QUERY_KEYS } from "@/lib/constants";
import { useWallet } from "./use-wallet";
import { useSignTransaction } from "./use-sign-transaction";
import type { SubmitResult } from "@/types";

export function useNameTransfer() {
  const { activeAddress } = useWallet();
  const { signNameTransfer, signing } = useSignTransaction();
  const queryClient = useQueryClient();
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const transferName = useCallback(
    async (name: string, to: string) => {
      if (!activeAddress) throw new Error("No active address");
      setSubmitting(true);
      setError(null);
      try {
        const hex = await signNameTransfer({ name, to });
        const result = await rpcCall<SubmitResult>("norn_transferName", [
          name,
          strip0x(activeAddress),
          hex,
        ]);
        if (!result.success) throw new Error(result.reason || "Transfer failed");
        queryClient.invalidateQueries({ queryKey: QUERY_KEYS.names(activeAddress) });
        queryClient.invalidateQueries({ queryKey: QUERY_KEYS.resolveName(name) });
        queryClient.invalidateQueries({ queryKey: ["txHistory", activeAddress] });
        return result;
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Transfer failed";
        setError(msg);
        throw e;
      } finally {
        setSubmitting(false);
      }
    },
    [signNameTransfer, activeAddress, queryClient]
  );

  return { transferName, loading: signing || submitting, error };
}

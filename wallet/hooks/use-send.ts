"use client";

import { useState, useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { QUERY_KEYS } from "@/lib/constants";
import { useWallet } from "./use-wallet";
import { useSignTransaction } from "./use-sign-transaction";
import type { SubmitResult } from "@/types";

export function useSend() {
  const { activeAddress } = useWallet();
  const { signTransfer, signing } = useSignTransaction();
  const queryClient = useQueryClient();
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const send = useCallback(
    async (params: { to: string; amount: string; tokenId?: string; memo?: string }) => {
      setSubmitting(true);
      setError(null);
      try {
        const knotHex = await signTransfer(params);
        const result = await rpcCall<SubmitResult>("norn_submitKnot", [knotHex]);
        if (!result.success) {
          throw new Error(result.reason || "Transaction rejected");
        }
        if (activeAddress) {
          queryClient.invalidateQueries({ queryKey: QUERY_KEYS.balance(activeAddress) });
          queryClient.invalidateQueries({ queryKey: QUERY_KEYS.threadState(activeAddress) });
          queryClient.invalidateQueries({ queryKey: ["txHistory", activeAddress] });
          if (params.tokenId) {
            queryClient.invalidateQueries({ queryKey: QUERY_KEYS.balance(activeAddress, params.tokenId) });
          }
        }
        return result;
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Send failed";
        setError(msg);
        throw e;
      } finally {
        setSubmitting(false);
      }
    },
    [signTransfer, activeAddress, queryClient]
  );

  return { send, sending: signing || submitting, error };
}

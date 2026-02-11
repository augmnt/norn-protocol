"use client";

import { useState, useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { strip0x } from "@/lib/format";
import { QUERY_KEYS } from "@/lib/constants";
import { useWallet } from "./use-wallet";
import type { SubmitResult } from "@/types";

export function useFaucet() {
  const { activeAddress } = useWallet();
  const queryClient = useQueryClient();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const requestFaucet = useCallback(async () => {
    if (!activeAddress) throw new Error("No active address");
    setLoading(true);
    setError(null);
    try {
      const result = await rpcCall<SubmitResult>("norn_faucet", [
        strip0x(activeAddress),
      ]);
      if (!result.success) {
        throw new Error(result.reason || "Faucet request failed");
      }
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.balance(activeAddress) });
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.threadState(activeAddress) });
      queryClient.invalidateQueries({ queryKey: ["txHistory", activeAddress] });
      return result;
    } catch (e) {
      const msg = e instanceof Error ? e.message : "Faucet request failed";
      setError(msg);
      throw e;
    } finally {
      setLoading(false);
    }
  }, [activeAddress, queryClient]);

  return { requestFaucet, loading, error };
}

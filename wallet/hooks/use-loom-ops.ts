"use client";

import { useState, useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { strip0x } from "@/lib/format";
import { QUERY_KEYS } from "@/lib/constants";
import { useWallet } from "./use-wallet";
import type { ExecutionResult, QueryResult } from "@/types";

export function useLoomOps() {
  const { activeAddress } = useWallet();
  const queryClient = useQueryClient();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const queryLoom = useCallback(
    async (loomId: string, inputHex: string): Promise<QueryResult> => {
      setLoading(true);
      setError(null);
      try {
        return await rpcCall<QueryResult>("norn_queryLoom", [
          strip0x(loomId),
          inputHex,
        ]);
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Query failed";
        setError(msg);
        throw e;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const executeLoom = useCallback(
    async (loomId: string, inputHex: string): Promise<ExecutionResult> => {
      if (!activeAddress) throw new Error("No active address");
      setLoading(true);
      setError(null);
      try {
        const result = await rpcCall<ExecutionResult>("norn_executeLoom", [
          strip0x(loomId),
          inputHex,
          strip0x(activeAddress),
        ]);
        queryClient.invalidateQueries({ queryKey: QUERY_KEYS.balance(activeAddress) });
        queryClient.invalidateQueries({ queryKey: QUERY_KEYS.threadState(activeAddress) });
        queryClient.invalidateQueries({ queryKey: ["txHistory", activeAddress] });
        return result;
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Execution failed";
        setError(msg);
        throw e;
      } finally {
        setLoading(false);
      }
    },
    [activeAddress, queryClient]
  );

  return { queryLoom, executeLoom, loading, error };
}

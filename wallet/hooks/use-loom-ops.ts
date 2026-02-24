"use client";

import { useState, useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { strip0x } from "@/lib/format";
import * as signer from "@/lib/secure-signer";

import { useWallet } from "./use-wallet";
import { useWalletStore } from "@/stores/wallet-store";
import type { ExecutionResult, QueryResult } from "@/types";

export function useLoomOps() {
  const { meta, activeAddress, activeAccountIndex } = useWallet();
  const queryClient = useQueryClient();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const queryLoom = useCallback(
    async (loomId: string, inputHex: string): Promise<QueryResult> => {
      try {
        return await rpcCall<QueryResult>("norn_queryLoom", [
          strip0x(loomId),
          inputHex,
        ]);
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Query failed";
        setError(msg);
        throw e;
      }
    },
    []
  );

  const executeLoom = useCallback(
    async (loomId: string, inputHex: string): Promise<ExecutionResult> => {
      if (!activeAddress || !meta) throw new Error("No active wallet");
      setLoading(true);
      setError(null);
      try {
        const pw = useWalletStore.getState().sessionPassword ?? undefined;
        const { signatureHex, pubkeyHex, senderHex } =
          await signer.signExecuteLoom(
            meta,
            strip0x(loomId),
            inputHex,
            activeAccountIndex,
            pw
          );
        const result = await rpcCall<ExecutionResult>("norn_executeLoom", [
          strip0x(loomId),
          inputHex,
          senderHex,
          signatureHex,
          pubkeyHex,
        ]);
        if (!result.success) {
          throw new Error(result.reason || "Execution failed");
        }
        queryClient.invalidateQueries({ queryKey: ["balance", activeAddress] });
        queryClient.invalidateQueries({ queryKey: ["threadState", activeAddress] });
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
    [meta, activeAddress, activeAccountIndex, queryClient]
  );

  return { queryLoom, executeLoom, loading, error };
}

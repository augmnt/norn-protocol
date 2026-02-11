"use client";

import { useState, useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { QUERY_KEYS } from "@/lib/constants";
import { useWallet } from "./use-wallet";
import { useSignTransaction } from "./use-sign-transaction";
import type { SubmitResult } from "@/types";

export function useTokenOps() {
  const { activeAddress } = useWallet();
  const { signTokenDefinition, signTokenMint, signTokenBurn, signing } = useSignTransaction();
  const queryClient = useQueryClient();
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const invalidateTokenState = useCallback((tokenId?: string) => {
    if (activeAddress) {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.balance(activeAddress) });
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.threadState(activeAddress) });
      queryClient.invalidateQueries({ queryKey: ["createdTokens", activeAddress] });
      queryClient.invalidateQueries({ queryKey: ["txHistory", activeAddress] });
      if (tokenId) {
        queryClient.invalidateQueries({ queryKey: QUERY_KEYS.balance(activeAddress, tokenId) });
      }
    }
    queryClient.invalidateQueries({ queryKey: ["tokensList"] });
    if (tokenId) {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.tokenInfo(tokenId) });
    }
  }, [activeAddress, queryClient]);

  const createToken = useCallback(
    async (params: { name: string; symbol: string; decimals: number; maxSupply: string; initialSupply?: string }) => {
      setSubmitting(true);
      setError(null);
      try {
        const hex = await signTokenDefinition(params);
        const result = await rpcCall<SubmitResult>("norn_createToken", [hex]);
        if (!result.success) throw new Error(result.reason || "Token creation failed");
        invalidateTokenState();
        return result;
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Token creation failed";
        setError(msg);
        throw e;
      } finally {
        setSubmitting(false);
      }
    },
    [signTokenDefinition, invalidateTokenState]
  );

  const mintToken = useCallback(
    async (params: { tokenId: string; to: string; amount: string; decimals?: number }) => {
      setSubmitting(true);
      setError(null);
      try {
        const hex = await signTokenMint(params);
        const result = await rpcCall<SubmitResult>("norn_mintToken", [hex]);
        if (!result.success) throw new Error(result.reason || "Mint failed");
        invalidateTokenState(params.tokenId);
        return result;
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Mint failed";
        setError(msg);
        throw e;
      } finally {
        setSubmitting(false);
      }
    },
    [signTokenMint, invalidateTokenState]
  );

  const burnToken = useCallback(
    async (params: { tokenId: string; amount: string; decimals?: number }) => {
      setSubmitting(true);
      setError(null);
      try {
        const hex = await signTokenBurn(params);
        const result = await rpcCall<SubmitResult>("norn_burnToken", [hex]);
        if (!result.success) throw new Error(result.reason || "Burn failed");
        invalidateTokenState(params.tokenId);
        return result;
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Burn failed";
        setError(msg);
        throw e;
      } finally {
        setSubmitting(false);
      }
    },
    [signTokenBurn, invalidateTokenState]
  );

  return { createToken, mintToken, burnToken, loading: signing || submitting, error };
}

"use client";

import { useCallback } from "react";
import { useLoomOps } from "./use-loom-ops";
import {
  encodeCreatePool,
  encodeAddLiquidity,
  encodeRemoveLiquidity,
  encodeSwapNornForToken,
  encodeSwapTokenForNorn,
  encodeGetPool,
  encodeGetPoolByToken,
  encodeGetPoolCount,
  encodeGetLpBalance,
  encodeGetQuote,
  encodeGetConfig,
  decodePool,
  decodeLpBalance,
  decodeQuote,
  decodeAmmConfig,
  decodeU64,
} from "@/lib/borsh-amm";
import type { AmmPool, AmmConfig } from "@/lib/borsh-amm";
import { strip0x } from "@/lib/format";

export function useAmm(loomId: string) {
  const { queryLoom, executeLoom, loading, error } = useLoomOps();

  // ── Execute operations ──────────────────────────────────────────

  const createPool = useCallback(
    async (token: string, nornAmount: bigint, tokenAmount: bigint) => {
      const input = encodeCreatePool(strip0x(token), nornAmount, tokenAmount);
      return executeLoom(loomId, input);
    },
    [loomId, executeLoom]
  );

  const addLiquidity = useCallback(
    async (poolId: bigint, nornAmount: bigint, tokenAmount: bigint) => {
      return executeLoom(
        loomId,
        encodeAddLiquidity(poolId, nornAmount, tokenAmount)
      );
    },
    [loomId, executeLoom]
  );

  const removeLiquidity = useCallback(
    async (poolId: bigint, lpAmount: bigint) => {
      return executeLoom(loomId, encodeRemoveLiquidity(poolId, lpAmount));
    },
    [loomId, executeLoom]
  );

  const swapNornForToken = useCallback(
    async (poolId: bigint, nornAmount: bigint, minTokenOut: bigint) => {
      return executeLoom(
        loomId,
        encodeSwapNornForToken(poolId, nornAmount, minTokenOut)
      );
    },
    [loomId, executeLoom]
  );

  const swapTokenForNorn = useCallback(
    async (poolId: bigint, tokenAmount: bigint, minNornOut: bigint) => {
      return executeLoom(
        loomId,
        encodeSwapTokenForNorn(poolId, tokenAmount, minNornOut)
      );
    },
    [loomId, executeLoom]
  );

  // ── Query operations ────────────────────────────────────────────

  const getPool = useCallback(
    async (poolId: bigint): Promise<AmmPool | null> => {
      try {
        const result = await queryLoom(loomId, encodeGetPool(poolId));
        if (!result?.output_hex) return null;
        return decodePool(result.output_hex);
      } catch {
        return null;
      }
    },
    [loomId, queryLoom]
  );

  const getPoolByToken = useCallback(
    async (tokenHex: string): Promise<AmmPool | null> => {
      try {
        const result = await queryLoom(
          loomId,
          encodeGetPoolByToken(strip0x(tokenHex))
        );
        if (!result?.output_hex) return null;
        return decodePool(result.output_hex);
      } catch {
        return null;
      }
    },
    [loomId, queryLoom]
  );

  const getPoolCount = useCallback(async (): Promise<bigint> => {
    try {
      const result = await queryLoom(loomId, encodeGetPoolCount());
      if (!result?.output_hex) return 0n;
      return decodeU64(result.output_hex);
    } catch {
      return 0n;
    }
  }, [loomId, queryLoom]);

  const getLpBalance = useCallback(
    async (poolId: bigint, address: string): Promise<bigint> => {
      try {
        const result = await queryLoom(
          loomId,
          encodeGetLpBalance(poolId, strip0x(address))
        );
        if (!result?.output_hex) return 0n;
        return decodeLpBalance(result.output_hex);
      } catch {
        return 0n;
      }
    },
    [loomId, queryLoom]
  );

  const getQuote = useCallback(
    async (
      poolId: bigint,
      inputIsNorn: boolean,
      amountIn: bigint
    ): Promise<bigint> => {
      try {
        const result = await queryLoom(
          loomId,
          encodeGetQuote(poolId, inputIsNorn, amountIn)
        );
        if (!result?.output_hex) return 0n;
        return decodeQuote(result.output_hex);
      } catch {
        return 0n;
      }
    },
    [loomId, queryLoom]
  );

  const getConfig = useCallback(async (): Promise<AmmConfig | null> => {
    try {
      const result = await queryLoom(loomId, encodeGetConfig());
      if (!result?.output_hex) return null;
      return decodeAmmConfig(result.output_hex);
    } catch {
      return null;
    }
  }, [loomId, queryLoom]);

  return {
    // Execute
    createPool,
    addLiquidity,
    removeLiquidity,
    swapNornForToken,
    swapTokenForNorn,
    // Query
    getPool,
    getPoolByToken,
    getPoolCount,
    getLpBalance,
    getQuote,
    getConfig,
    // State
    loading,
    error,
  };
}

"use client";

import { useCallback } from "react";
import { useLoomOps } from "./use-loom-ops";
import {
  encodeInitialize,
  encodeContribute,
  encodeClaimTokens,
  encodeFinalize,
  encodeRefund,
  encodeGetConfig,
  encodeGetContribution,
  encodeGetTotalRaised,
  decodeLaunchConfig,
  decodeU128,
} from "@/lib/borsh-launchpad";
import type { LaunchConfig } from "@/lib/borsh-launchpad";
import { strip0x } from "@/lib/format";

export function useLaunchpad(loomId: string) {
  const { queryLoom, executeLoom, loading, error } = useLoomOps();

  const initialize = useCallback(
    async (
      tokenId: string,
      price: bigint,
      hardCap: bigint,
      maxPerWallet: bigint,
      startTime: bigint,
      endTime: bigint,
      totalTokens: bigint
    ) => {
      const input = encodeInitialize(
        strip0x(tokenId),
        price,
        hardCap,
        maxPerWallet,
        startTime,
        endTime,
        totalTokens
      );
      return executeLoom(loomId, input);
    },
    [loomId, executeLoom]
  );

  const contribute = useCallback(
    async (amount: bigint) => {
      return executeLoom(loomId, encodeContribute(amount));
    },
    [loomId, executeLoom]
  );

  const claimTokens = useCallback(async () => {
    return executeLoom(loomId, encodeClaimTokens());
  }, [loomId, executeLoom]);

  const finalize = useCallback(async () => {
    return executeLoom(loomId, encodeFinalize());
  }, [loomId, executeLoom]);

  const refund = useCallback(async () => {
    return executeLoom(loomId, encodeRefund());
  }, [loomId, executeLoom]);

  const getConfig = useCallback(async (): Promise<LaunchConfig | null> => {
    try {
      const result = await queryLoom(loomId, encodeGetConfig());
      if (!result?.output_hex) return null;
      return decodeLaunchConfig(result.output_hex);
    } catch {
      return null;
    }
  }, [loomId, queryLoom]);

  const getContribution = useCallback(
    async (addr: string): Promise<bigint> => {
      try {
        const result = await queryLoom(
          loomId,
          encodeGetContribution(strip0x(addr))
        );
        if (!result?.output_hex) return 0n;
        return decodeU128(result.output_hex);
      } catch {
        return 0n;
      }
    },
    [loomId, queryLoom]
  );

  const getTotalRaised = useCallback(async (): Promise<bigint> => {
    try {
      const result = await queryLoom(loomId, encodeGetTotalRaised());
      if (!result?.output_hex) return 0n;
      return decodeU128(result.output_hex);
    } catch {
      return 0n;
    }
  }, [loomId, queryLoom]);

  return {
    initialize,
    contribute,
    claimTokens,
    finalize,
    refund,
    getConfig,
    getContribution,
    getTotalRaised,
    loading,
    error,
  };
}

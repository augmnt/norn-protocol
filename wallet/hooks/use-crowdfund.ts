"use client";

import { useCallback } from "react";
import { useLoomOps } from "./use-loom-ops";
import {
  encodeInitialize,
  encodeContribute,
  encodeFinalize,
  encodeRefund,
  encodeGetConfig,
  encodeGetContribution,
  encodeGetTotalRaised,
  encodeGetContributorCount,
  decodeCrowdfundConfig,
  decodeU128,
  decodeU64,
} from "@/lib/borsh-crowdfund";
import type { CrowdfundConfig } from "@/lib/borsh-crowdfund";
import { strip0x } from "@/lib/format";

export function useCrowdfund(loomId: string) {
  const { queryLoom, executeLoom, loading, error } = useLoomOps();

  const initialize = useCallback(
    async (
      title: string,
      description: string,
      tokenId: string,
      goal: bigint,
      deadline: bigint
    ) => {
      const input = encodeInitialize(
        title,
        description,
        strip0x(tokenId),
        goal,
        deadline
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

  const finalize = useCallback(async () => {
    return executeLoom(loomId, encodeFinalize());
  }, [loomId, executeLoom]);

  const refund = useCallback(async () => {
    return executeLoom(loomId, encodeRefund());
  }, [loomId, executeLoom]);

  const getConfig = useCallback(async (): Promise<CrowdfundConfig | null> => {
    try {
      const result = await queryLoom(loomId, encodeGetConfig());
      if (!result?.output_hex) return null;
      return decodeCrowdfundConfig(result.output_hex);
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

  const getContributorCount = useCallback(async (): Promise<bigint> => {
    try {
      const result = await queryLoom(loomId, encodeGetContributorCount());
      if (!result?.output_hex) return 0n;
      return decodeU64(result.output_hex);
    } catch {
      return 0n;
    }
  }, [loomId, queryLoom]);

  return {
    initialize,
    contribute,
    finalize,
    refund,
    getConfig,
    getContribution,
    getTotalRaised,
    getContributorCount,
    loading,
    error,
  };
}

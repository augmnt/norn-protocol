"use client";

import { useCallback } from "react";
import { useLoomOps } from "./use-loom-ops";
import {
  encodeCreateDeal,
  encodeFundDeal,
  encodeMarkDelivered,
  encodeConfirmReceived,
  encodeDispute,
  encodeCancelDeal,
  encodeRefundExpired,
  encodeGetDeal,
  encodeGetDealCount,
  decodeDeal,
  decodeU64,
} from "@/lib/borsh-escrow";
import type { Deal } from "@/lib/borsh-escrow";
import { strip0x } from "@/lib/format";

export function useEscrow(loomId: string) {
  const { queryLoom, executeLoom, loading, error } = useLoomOps();

  const createDeal = useCallback(
    async (
      seller: string,
      tokenId: string,
      amount: bigint,
      description: string,
      deadline: bigint
    ) => {
      const input = encodeCreateDeal(
        strip0x(seller),
        strip0x(tokenId),
        amount,
        description,
        deadline
      );
      return executeLoom(loomId, input);
    },
    [loomId, executeLoom]
  );

  const fundDeal = useCallback(
    async (dealId: bigint) => {
      return executeLoom(loomId, encodeFundDeal(dealId));
    },
    [loomId, executeLoom]
  );

  const markDelivered = useCallback(
    async (dealId: bigint) => {
      return executeLoom(loomId, encodeMarkDelivered(dealId));
    },
    [loomId, executeLoom]
  );

  const confirmReceived = useCallback(
    async (dealId: bigint) => {
      return executeLoom(loomId, encodeConfirmReceived(dealId));
    },
    [loomId, executeLoom]
  );

  const dispute = useCallback(
    async (dealId: bigint) => {
      return executeLoom(loomId, encodeDispute(dealId));
    },
    [loomId, executeLoom]
  );

  const cancelDeal = useCallback(
    async (dealId: bigint) => {
      return executeLoom(loomId, encodeCancelDeal(dealId));
    },
    [loomId, executeLoom]
  );

  const refundExpired = useCallback(
    async (dealId: bigint) => {
      return executeLoom(loomId, encodeRefundExpired(dealId));
    },
    [loomId, executeLoom]
  );

  const getDeal = useCallback(
    async (dealId: bigint): Promise<Deal | null> => {
      try {
        const result = await queryLoom(loomId, encodeGetDeal(dealId));
        if (!result?.output_hex) return null;
        return decodeDeal(result.output_hex);
      } catch {
        return null;
      }
    },
    [loomId, queryLoom]
  );

  const getDealCount = useCallback(async (): Promise<bigint> => {
    try {
      const result = await queryLoom(loomId, encodeGetDealCount());
      if (!result?.output_hex) return 0n;
      return decodeU64(result.output_hex);
    } catch {
      return 0n;
    }
  }, [loomId, queryLoom]);

  return {
    createDeal,
    fundDeal,
    markDelivered,
    confirmReceived,
    dispute,
    cancelDeal,
    refundExpired,
    getDeal,
    getDealCount,
    loading,
    error,
  };
}

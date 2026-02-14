"use client";

import { useCallback } from "react";
import { useLoomOps } from "./use-loom-ops";
import {
  encodeCreateOrder,
  encodeFillOrder,
  encodeCancelOrder,
  encodeGetOrder,
  encodeGetOrderCount,
  decodeSwapOrder,
  decodeU64,
} from "@/lib/borsh-swap";
import type { SwapOrder } from "@/lib/borsh-swap";
import { strip0x } from "@/lib/format";

export function useSwap(loomId: string) {
  const { queryLoom, executeLoom, loading, error } = useLoomOps();

  const createOrder = useCallback(
    async (
      sellToken: string,
      sellAmount: bigint,
      buyToken: string,
      buyAmount: bigint
    ) => {
      const input = encodeCreateOrder(
        strip0x(sellToken),
        sellAmount,
        strip0x(buyToken),
        buyAmount
      );
      return executeLoom(loomId, input);
    },
    [loomId, executeLoom]
  );

  const fillOrder = useCallback(
    async (orderId: bigint) => {
      return executeLoom(loomId, encodeFillOrder(orderId));
    },
    [loomId, executeLoom]
  );

  const cancelOrder = useCallback(
    async (orderId: bigint) => {
      return executeLoom(loomId, encodeCancelOrder(orderId));
    },
    [loomId, executeLoom]
  );

  const getOrder = useCallback(
    async (orderId: bigint): Promise<SwapOrder | null> => {
      try {
        const result = await queryLoom(loomId, encodeGetOrder(orderId));
        if (!result?.output_hex) return null;
        return decodeSwapOrder(result.output_hex);
      } catch {
        return null;
      }
    },
    [loomId, queryLoom]
  );

  const getOrderCount = useCallback(async (): Promise<bigint> => {
    try {
      const result = await queryLoom(loomId, encodeGetOrderCount());
      if (!result?.output_hex) return 0n;
      return decodeU64(result.output_hex);
    } catch {
      return 0n;
    }
  }, [loomId, queryLoom]);

  return {
    createOrder,
    fillOrder,
    cancelOrder,
    getOrder,
    getOrderCount,
    loading,
    error,
  };
}

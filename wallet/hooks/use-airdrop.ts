"use client";

import { useCallback } from "react";
import { useLoomOps } from "./use-loom-ops";
import {
  encodeInitialize,
  encodeAddRecipients,
  encodeFinalize,
  encodeClaim,
  encodeReclaimRemaining,
  encodeGetConfig,
  encodeGetAllocation,
  encodeIsClaimed,
  decodeAirdropConfig,
  decodeU128,
  decodeBool,
} from "@/lib/borsh-airdrop";
import type { AirdropConfig } from "@/lib/borsh-airdrop";
import { strip0x } from "@/lib/format";

export function useAirdrop(loomId: string) {
  const { queryLoom, executeLoom, loading, error } = useLoomOps();

  const initialize = useCallback(
    async (tokenId: string, totalAmount: bigint) => {
      const input = encodeInitialize(strip0x(tokenId), totalAmount);
      return executeLoom(loomId, input);
    },
    [loomId, executeLoom]
  );

  const addRecipients = useCallback(
    async (recipients: { address: string; amount: bigint }[]) => {
      const stripped = recipients.map((r) => ({
        address: strip0x(r.address),
        amount: r.amount,
      }));
      const input = encodeAddRecipients(stripped);
      return executeLoom(loomId, input);
    },
    [loomId, executeLoom]
  );

  const finalize = useCallback(async () => {
    return executeLoom(loomId, encodeFinalize());
  }, [loomId, executeLoom]);

  const claim = useCallback(async () => {
    return executeLoom(loomId, encodeClaim());
  }, [loomId, executeLoom]);

  const reclaimRemaining = useCallback(async () => {
    return executeLoom(loomId, encodeReclaimRemaining());
  }, [loomId, executeLoom]);

  const getConfig = useCallback(async (): Promise<AirdropConfig | null> => {
    try {
      const result = await queryLoom(loomId, encodeGetConfig());
      if (!result?.output_hex) return null;
      return decodeAirdropConfig(result.output_hex);
    } catch {
      return null;
    }
  }, [loomId, queryLoom]);

  const getAllocation = useCallback(
    async (addr: string): Promise<bigint> => {
      try {
        const result = await queryLoom(
          loomId,
          encodeGetAllocation(strip0x(addr))
        );
        if (!result?.output_hex) return 0n;
        return decodeU128(result.output_hex);
      } catch {
        return 0n;
      }
    },
    [loomId, queryLoom]
  );

  const isClaimed = useCallback(
    async (addr: string): Promise<boolean> => {
      try {
        const result = await queryLoom(
          loomId,
          encodeIsClaimed(strip0x(addr))
        );
        if (!result?.output_hex) return false;
        return decodeBool(result.output_hex);
      } catch {
        return false;
      }
    },
    [loomId, queryLoom]
  );

  return {
    initialize,
    addRecipients,
    finalize,
    claim,
    reclaimRemaining,
    getConfig,
    getAllocation,
    isClaimed,
    loading,
    error,
  };
}

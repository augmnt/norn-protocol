"use client";

import { useCallback } from "react";
import { useLoomOps } from "./use-loom-ops";
import {
  encodeInitialize,
  encodeSplit,
  encodeGetConfig,
  decodeSplitterConfig,
} from "@/lib/borsh-splitter";
import type { SplitterConfig } from "@/lib/borsh-splitter";
import { strip0x } from "@/lib/format";

export function useSplitter(loomId: string) {
  const { queryLoom, executeLoom, loading, error } = useLoomOps();

  const initialize = useCallback(
    async (
      name: string,
      recipients: { address: string; shareBps: bigint }[]
    ) => {
      const strippedRecipients = recipients.map((r) => ({
        address: strip0x(r.address),
        shareBps: r.shareBps,
      }));
      const input = encodeInitialize(name, strippedRecipients);
      return executeLoom(loomId, input);
    },
    [loomId, executeLoom]
  );

  const split = useCallback(
    async (tokenId: string, amount: bigint) => {
      return executeLoom(loomId, encodeSplit(strip0x(tokenId), amount));
    },
    [loomId, executeLoom]
  );

  const getConfig = useCallback(async (): Promise<SplitterConfig | null> => {
    try {
      const result = await queryLoom(loomId, encodeGetConfig());
      if (!result?.output_hex) return null;
      return decodeSplitterConfig(result.output_hex);
    } catch {
      return null;
    }
  }, [loomId, queryLoom]);

  return {
    initialize,
    split,
    getConfig,
    loading,
    error,
  };
}

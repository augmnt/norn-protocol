"use client";

import { useCallback } from "react";
import { useLoomOps } from "./use-loom-ops";
import {
  encodeInitialize,
  encodePropose,
  encodeVote,
  encodeFinalize,
  encodeGetConfig,
  encodeGetProposal,
  encodeGetProposalCount,
  encodeGetVote,
  decodeGovConfig,
  decodeGovProposal,
  decodeU64,
  decodeBool,
} from "@/lib/borsh-governance";
import type { GovConfig, GovProposal } from "@/lib/borsh-governance";
import { strip0x } from "@/lib/format";

export function useGovernance(loomId: string) {
  const { queryLoom, executeLoom, loading, error } = useLoomOps();

  const initialize = useCallback(
    async (name: string, votingPeriod: bigint, quorum: bigint) => {
      const input = encodeInitialize(name, votingPeriod, quorum);
      return executeLoom(loomId, input);
    },
    [loomId, executeLoom]
  );

  const propose = useCallback(
    async (title: string, description: string) => {
      const input = encodePropose(title, description);
      return executeLoom(loomId, input);
    },
    [loomId, executeLoom]
  );

  const vote = useCallback(
    async (proposalId: bigint, support: boolean) => {
      return executeLoom(loomId, encodeVote(proposalId, support));
    },
    [loomId, executeLoom]
  );

  const finalize = useCallback(
    async (proposalId: bigint) => {
      return executeLoom(loomId, encodeFinalize(proposalId));
    },
    [loomId, executeLoom]
  );

  const getConfig = useCallback(async (): Promise<GovConfig | null> => {
    try {
      const result = await queryLoom(loomId, encodeGetConfig());
      if (!result?.output_hex) return null;
      return decodeGovConfig(result.output_hex);
    } catch {
      return null;
    }
  }, [loomId, queryLoom]);

  const getProposal = useCallback(
    async (proposalId: bigint): Promise<GovProposal | null> => {
      try {
        const result = await queryLoom(
          loomId,
          encodeGetProposal(proposalId)
        );
        if (!result?.output_hex) return null;
        return decodeGovProposal(result.output_hex);
      } catch {
        return null;
      }
    },
    [loomId, queryLoom]
  );

  const getProposalCount = useCallback(async (): Promise<bigint> => {
    try {
      const result = await queryLoom(loomId, encodeGetProposalCount());
      if (!result?.output_hex) return 0n;
      return decodeU64(result.output_hex);
    } catch {
      return 0n;
    }
  }, [loomId, queryLoom]);

  const getVote = useCallback(
    async (proposalId: bigint, voter: string): Promise<boolean | null> => {
      try {
        const result = await queryLoom(
          loomId,
          encodeGetVote(proposalId, strip0x(voter))
        );
        if (!result?.output_hex) return null;
        return decodeBool(result.output_hex);
      } catch {
        return null;
      }
    },
    [loomId, queryLoom]
  );

  return {
    initialize,
    propose,
    vote,
    finalize,
    getConfig,
    getProposal,
    getProposalCount,
    getVote,
    loading,
    error,
  };
}

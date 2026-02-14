"use client";

import { useCallback } from "react";
import { useLoomOps } from "./use-loom-ops";
import {
  encodeInitialize,
  encodePropose,
  encodeApprove,
  encodeReject,
  encodeDeposit,
  encodeRevokeApproval,
  encodeExpireProposal,
  encodeGetConfig,
  encodeGetProposal,
  encodeGetProposalCount,
  decodeTreasuryConfig,
  decodeProposal,
  decodeU64,
} from "@/lib/borsh-treasury";
import type { TreasuryConfig, Proposal } from "@/lib/borsh-treasury";
import { strip0x } from "@/lib/format";

export function useTreasury(loomId: string) {
  const { queryLoom, executeLoom, loading, error } = useLoomOps();

  const initialize = useCallback(
    async (owners: string[], requiredApprovals: bigint, name: string) => {
      const input = encodeInitialize(
        owners.map(strip0x),
        requiredApprovals,
        name
      );
      return executeLoom(loomId, input);
    },
    [loomId, executeLoom]
  );

  const propose = useCallback(
    async (
      to: string,
      tokenId: string,
      amount: bigint,
      description: string,
      deadline: bigint
    ) => {
      const input = encodePropose(
        strip0x(to),
        strip0x(tokenId),
        amount,
        description,
        deadline
      );
      return executeLoom(loomId, input);
    },
    [loomId, executeLoom]
  );

  const approve = useCallback(
    async (proposalId: bigint) => {
      return executeLoom(loomId, encodeApprove(proposalId));
    },
    [loomId, executeLoom]
  );

  const reject = useCallback(
    async (proposalId: bigint) => {
      return executeLoom(loomId, encodeReject(proposalId));
    },
    [loomId, executeLoom]
  );

  const deposit = useCallback(
    async (tokenId: string, amount: bigint) => {
      return executeLoom(loomId, encodeDeposit(strip0x(tokenId), amount));
    },
    [loomId, executeLoom]
  );

  const revokeApproval = useCallback(
    async (proposalId: bigint) => {
      return executeLoom(loomId, encodeRevokeApproval(proposalId));
    },
    [loomId, executeLoom]
  );

  const expireProposal = useCallback(
    async (proposalId: bigint) => {
      return executeLoom(loomId, encodeExpireProposal(proposalId));
    },
    [loomId, executeLoom]
  );

  const getConfig = useCallback(async (): Promise<TreasuryConfig | null> => {
    try {
      const result = await queryLoom(loomId, encodeGetConfig());
      if (!result?.output_hex) return null;
      return decodeTreasuryConfig(result.output_hex);
    } catch {
      return null;
    }
  }, [loomId, queryLoom]);

  const getProposal = useCallback(
    async (proposalId: bigint): Promise<Proposal | null> => {
      try {
        const result = await queryLoom(loomId, encodeGetProposal(proposalId));
        if (!result?.output_hex) return null;
        return decodeProposal(result.output_hex);
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

  return {
    initialize,
    propose,
    approve,
    reject,
    deposit,
    revokeApproval,
    expireProposal,
    getConfig,
    getProposal,
    getProposalCount,
    loading,
    error,
  };
}

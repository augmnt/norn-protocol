"use client";

import { useCallback } from "react";
import { useLoomOps } from "./use-loom-ops";
import {
  encodeInitialize,
  encodeStake,
  encodeUnstake,
  encodeClaimRewards,
  encodeFundRewards,
  encodeGetConfig,
  encodeGetStake,
  encodeGetPendingRewards,
  encodeGetTotalStaked,
  encodeGetRewardPool,
  decodeStakingConfig,
  decodeStakeInfo,
  decodeU128,
} from "@/lib/borsh-staking";
import type { StakingConfig, StakeInfo } from "@/lib/borsh-staking";
import { strip0x } from "@/lib/format";

export function useStaking(loomId: string) {
  const { queryLoom, executeLoom, loading, error } = useLoomOps();

  const initialize = useCallback(
    async (tokenId: string, rewardRate: bigint, minLockPeriod: bigint) => {
      const input = encodeInitialize(
        strip0x(tokenId),
        rewardRate,
        minLockPeriod
      );
      return executeLoom(loomId, input);
    },
    [loomId, executeLoom]
  );

  const stake = useCallback(
    async (amount: bigint) => {
      return executeLoom(loomId, encodeStake(amount));
    },
    [loomId, executeLoom]
  );

  const unstake = useCallback(
    async (amount: bigint) => {
      return executeLoom(loomId, encodeUnstake(amount));
    },
    [loomId, executeLoom]
  );

  const claimRewards = useCallback(async () => {
    return executeLoom(loomId, encodeClaimRewards());
  }, [loomId, executeLoom]);

  const fundRewards = useCallback(
    async (amount: bigint) => {
      return executeLoom(loomId, encodeFundRewards(amount));
    },
    [loomId, executeLoom]
  );

  const getConfig = useCallback(async (): Promise<StakingConfig | null> => {
    try {
      const result = await queryLoom(loomId, encodeGetConfig());
      if (!result?.output_hex) return null;
      return decodeStakingConfig(result.output_hex);
    } catch {
      return null;
    }
  }, [loomId, queryLoom]);

  const getStake = useCallback(
    async (addr: string): Promise<StakeInfo | null> => {
      try {
        const result = await queryLoom(
          loomId,
          encodeGetStake(strip0x(addr))
        );
        if (!result?.output_hex) return null;
        return decodeStakeInfo(result.output_hex);
      } catch {
        return null;
      }
    },
    [loomId, queryLoom]
  );

  const getPendingRewards = useCallback(
    async (addr: string): Promise<bigint> => {
      try {
        const result = await queryLoom(
          loomId,
          encodeGetPendingRewards(strip0x(addr))
        );
        if (!result?.output_hex) return 0n;
        return decodeU128(result.output_hex);
      } catch {
        return 0n;
      }
    },
    [loomId, queryLoom]
  );

  const getTotalStaked = useCallback(async (): Promise<bigint> => {
    try {
      const result = await queryLoom(loomId, encodeGetTotalStaked());
      if (!result?.output_hex) return 0n;
      return decodeU128(result.output_hex);
    } catch {
      return 0n;
    }
  }, [loomId, queryLoom]);

  const getRewardPool = useCallback(async (): Promise<bigint> => {
    try {
      const result = await queryLoom(loomId, encodeGetRewardPool());
      if (!result?.output_hex) return 0n;
      return decodeU128(result.output_hex);
    } catch {
      return 0n;
    }
  }, [loomId, queryLoom]);

  return {
    initialize,
    stake,
    unstake,
    claimRewards,
    fundRewards,
    getConfig,
    getStake,
    getPendingRewards,
    getTotalStaked,
    getRewardPool,
    loading,
    error,
  };
}

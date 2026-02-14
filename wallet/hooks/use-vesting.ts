"use client";

import { useCallback } from "react";
import { useLoomOps } from "./use-loom-ops";
import {
  encodeCreateSchedule,
  encodeClaim,
  encodeRevoke,
  encodeGetSchedule,
  encodeGetScheduleCount,
  encodeGetClaimable,
  decodeVestingSchedule,
  decodeU64,
  decodeU128,
} from "@/lib/borsh-vesting";
import type { VestingSchedule } from "@/lib/borsh-vesting";
import { strip0x } from "@/lib/format";

export function useVesting(loomId: string) {
  const { queryLoom, executeLoom, loading, error } = useLoomOps();

  const createSchedule = useCallback(
    async (
      beneficiary: string,
      tokenId: string,
      amount: bigint,
      startTime: bigint,
      cliffDuration: bigint,
      totalDuration: bigint,
      revocable: boolean
    ) => {
      const input = encodeCreateSchedule(
        strip0x(beneficiary),
        strip0x(tokenId),
        amount,
        startTime,
        cliffDuration,
        totalDuration,
        revocable
      );
      return executeLoom(loomId, input);
    },
    [loomId, executeLoom]
  );

  const claim = useCallback(
    async (scheduleId: bigint) => {
      return executeLoom(loomId, encodeClaim(scheduleId));
    },
    [loomId, executeLoom]
  );

  const revoke = useCallback(
    async (scheduleId: bigint) => {
      return executeLoom(loomId, encodeRevoke(scheduleId));
    },
    [loomId, executeLoom]
  );

  const getSchedule = useCallback(
    async (scheduleId: bigint): Promise<VestingSchedule | null> => {
      try {
        const result = await queryLoom(
          loomId,
          encodeGetSchedule(scheduleId)
        );
        if (!result?.output_hex) return null;
        return decodeVestingSchedule(result.output_hex);
      } catch {
        return null;
      }
    },
    [loomId, queryLoom]
  );

  const getScheduleCount = useCallback(async (): Promise<bigint> => {
    try {
      const result = await queryLoom(loomId, encodeGetScheduleCount());
      if (!result?.output_hex) return 0n;
      return decodeU64(result.output_hex);
    } catch {
      return 0n;
    }
  }, [loomId, queryLoom]);

  const getClaimable = useCallback(
    async (scheduleId: bigint): Promise<bigint> => {
      try {
        const result = await queryLoom(
          loomId,
          encodeGetClaimable(scheduleId)
        );
        if (!result?.output_hex) return 0n;
        return decodeU128(result.output_hex);
      } catch {
        return 0n;
      }
    },
    [loomId, queryLoom]
  );

  return {
    createSchedule,
    claim,
    revoke,
    getSchedule,
    getScheduleCount,
    getClaimable,
    loading,
    error,
  };
}

"use client";

import { useCallback } from "react";
import { useLoomOps } from "./use-loom-ops";
import {
  encodeLock,
  encodeWithdraw,
  encodeGetLock,
  encodeGetLockCount,
  decodeLockInfo,
  decodeU64,
} from "@/lib/borsh-timelock";
import type { LockInfo } from "@/lib/borsh-timelock";
import { strip0x } from "@/lib/format";

export function useTimelock(loomId: string) {
  const { queryLoom, executeLoom, loading, error } = useLoomOps();

  const lock = useCallback(
    async (tokenId: string, amount: bigint, unlockTime: bigint) => {
      const input = encodeLock(strip0x(tokenId), amount, unlockTime);
      return executeLoom(loomId, input);
    },
    [loomId, executeLoom]
  );

  const withdraw = useCallback(
    async (lockId: bigint) => {
      return executeLoom(loomId, encodeWithdraw(lockId));
    },
    [loomId, executeLoom]
  );

  const getLock = useCallback(
    async (lockId: bigint): Promise<LockInfo | null> => {
      try {
        const result = await queryLoom(loomId, encodeGetLock(lockId));
        if (!result?.output_hex) return null;
        return decodeLockInfo(result.output_hex);
      } catch {
        return null;
      }
    },
    [loomId, queryLoom]
  );

  const getLockCount = useCallback(async (): Promise<bigint> => {
    try {
      const result = await queryLoom(loomId, encodeGetLockCount());
      if (!result?.output_hex) return 0n;
      return decodeU64(result.output_hex);
    } catch {
      return 0n;
    }
  }, [loomId, queryLoom]);

  return {
    lock,
    withdraw,
    getLock,
    getLockCount,
    loading,
    error,
  };
}

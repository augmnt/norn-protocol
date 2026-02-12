"use client";

import { useState, useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";

import { useWallet } from "./use-wallet";
import { useSignTransaction } from "./use-sign-transaction";
import type { SubmitResult } from "@/types";

export function useSend() {
  const { activeAddress } = useWallet();
  const { signTransfer, signing } = useSignTransaction();
  const queryClient = useQueryClient();
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const send = useCallback(
    async (params: { to: string; amount: string; tokenId?: string; memo?: string; decimals?: number }) => {
      setSubmitting(true);
      setError(null);
      try {
        // Fetch the sender's current thread state for before-state verification.
        let beforeState: { version: bigint; stateHash: string } | undefined;
        if (activeAddress) {
          try {
            const ts = await rpcCall<{ version: number; state_hash: string }>(
              "norn_getThreadState",
              [activeAddress],
            );
            if (ts?.state_hash && ts.state_hash !== "0".repeat(64)) {
              beforeState = {
                version: BigInt(ts.version),
                stateHash: ts.state_hash,
              };
            }
          } catch {
            // Fall back to no before-state if the thread doesn't exist yet.
          }
        }

        const knotHex = await signTransfer({ ...params, beforeState });
        const result = await rpcCall<SubmitResult>("norn_submitKnot", [knotHex]);
        if (!result.success) {
          throw new Error(result.reason || "Transaction rejected");
        }
        if (activeAddress) {
          queryClient.invalidateQueries({ queryKey: ["balance", activeAddress] });
          queryClient.invalidateQueries({ queryKey: ["threadState", activeAddress] });
          queryClient.invalidateQueries({ queryKey: ["txHistory", activeAddress] });

          // Advisory post-transfer state verification: confirm state was updated.
          setTimeout(async () => {
            try {
              const newTs = await rpcCall<{ state_hash: string }>(
                "norn_getThreadState",
                [activeAddress],
              );
              if (beforeState && newTs?.state_hash === beforeState.stateHash) {
                console.warn("[norn] state_hash unchanged after transfer — state may not have been applied yet");
              }
            } catch {
              // Non-critical — don't surface to user.
            }
          }, 500);
        }
        return result;
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Send failed";
        setError(msg);
        throw e;
      } finally {
        setSubmitting(false);
      }
    },
    [signTransfer, activeAddress, queryClient]
  );

  return { send, sending: signing || submitting, error };
}

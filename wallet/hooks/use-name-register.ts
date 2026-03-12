"use client";

import { useState, useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { strip0x } from "@/lib/format";
import { QUERY_KEYS, NATIVE_TOKEN_ID, NORN_DECIMALS } from "@/lib/constants";
import { useWallet } from "./use-wallet";
import { useSignTransaction } from "./use-sign-transaction";
import type { SubmitResult } from "@/types";

/** Name registration fee: 1 NORN (in base units). */
const NAME_REGISTRATION_FEE = 10n ** BigInt(NORN_DECIMALS);

export function useNameRegister() {
  const { activeAddress } = useWallet();
  const { signNameRegistration, signing } = useSignTransaction();
  const queryClient = useQueryClient();
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const registerName = useCallback(
    async (name: string) => {
      if (!activeAddress) throw new Error("No active address");
      setSubmitting(true);
      setError(null);
      try {
        // Pre-flight balance check — fail fast before signing or submitting.
        const raw = await rpcCall<string>("norn_getBalance", [
          strip0x(activeAddress),
          strip0x(NATIVE_TOKEN_ID),
        ]);
        if (BigInt(raw || "0") < NAME_REGISTRATION_FEE) {
          throw new Error(
            "Insufficient balance for name registration fee (requires 1 NORN)"
          );
        }

        const hex = await signNameRegistration(name);
        const result = await rpcCall<SubmitResult>("norn_registerName", [
          name,
          strip0x(activeAddress),
          hex,
        ]);
        if (!result.success) throw new Error(result.reason || "Registration failed");
        queryClient.invalidateQueries({ queryKey: QUERY_KEYS.names(activeAddress) });
        queryClient.invalidateQueries({ queryKey: ["balance", activeAddress] });
        queryClient.invalidateQueries({ queryKey: ["threadState", activeAddress] });
        queryClient.invalidateQueries({ queryKey: ["txHistory", activeAddress] });
        return result;
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Registration failed";
        setError(msg);
        throw e;
      } finally {
        setSubmitting(false);
      }
    },
    [signNameRegistration, activeAddress, queryClient]
  );

  return { registerName, loading: signing || submitting, error };
}

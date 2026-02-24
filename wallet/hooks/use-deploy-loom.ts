"use client";

import { useState, useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { rpcCall } from "@/lib/rpc";
import { useWallet } from "./use-wallet";
import { useWalletStore } from "@/stores/wallet-store";
import * as signer from "@/lib/secure-signer";
import type { LoomInfo, SubmitResult } from "@/types";

/** Poll until the loom appears in state (i.e. a block has included the deploy). */
async function waitForLoom(loomId: string, maxWaitMs = 30_000): Promise<void> {
  const start = Date.now();
  const interval = 1_000;
  while (Date.now() - start < maxWaitMs) {
    const info = await rpcCall<LoomInfo | null>("norn_getLoomInfo", [loomId]);
    if (info) return;
    await new Promise((r) => setTimeout(r, interval));
  }
  throw new Error("Timed out waiting for loom to be confirmed on-chain");
}

/**
 * Hook for deploying a new loom instance:
 * 1. Sign & submit a LoomRegistration (costs 50 NORN)
 * 2. Wait for the deployment to be included in a block
 * 3. Upload WASM bytecode + optional init message
 *
 * Returns the computed loom ID on success.
 */
export function useDeployLoom() {
  const { meta, activeAccountIndex } = useWallet();
  const queryClient = useQueryClient();
  const [deploying, setDeploying] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const deploy = useCallback(
    async (params: {
      name: string;
      wasmBytes: Uint8Array;
      initMsgHex?: string;
    }): Promise<string> => {
      if (!meta) throw new Error("No wallet");
      setDeploying(true);
      setError(null);

      try {
        const pw = useWalletStore.getState().sessionPassword ?? undefined;

        // Step 1: Build and sign the loom registration
        const registrationHex = await signer.signLoomRegistration(
          meta,
          params.name,
          activeAccountIndex,
          pw
        );

        // Step 2: Submit the deployment
        const deployResult = await rpcCall<SubmitResult>("norn_deployLoom", [
          registrationHex,
        ]);
        if (!deployResult.success) {
          throw new Error(deployResult.reason || "Loom deployment failed");
        }

        // Extract the loom ID from the response message
        // Format: "loom deployed (id: <hex>, will be included in next block)"
        const idMatch = deployResult.reason?.match(/id:\s*([0-9a-f]{64})/i);
        if (!idMatch) {
          throw new Error("Could not extract loom ID from deployment response");
        }
        const loomId = idMatch[1];

        // Step 3: Wait for the loom to appear in state (block confirmation)
        await waitForLoom(loomId);

        // Step 4: Upload bytecode + init (signed by operator)
        const bytecodeHex = Array.from(params.wasmBytes)
          .map((b) => b.toString(16).padStart(2, "0"))
          .join("");

        const { signatureHex: opSig, pubkeyHex: opPubkey } =
          await signer.signBytecodeUpload(
            meta,
            loomId,
            params.wasmBytes,
            activeAccountIndex,
            pw
          );

        const uploadResult = await rpcCall<SubmitResult>(
          "norn_uploadLoomBytecode",
          [loomId, bytecodeHex, params.initMsgHex ?? null, opSig, opPubkey]
        );
        if (!uploadResult.success) {
          throw new Error(uploadResult.reason || "Bytecode upload failed");
        }

        // Invalidate loom-related queries
        queryClient.invalidateQueries({ queryKey: ["appInstances"] });
        queryClient.invalidateQueries({ queryKey: ["loomsList"] });
        queryClient.invalidateQueries({ queryKey: ["balance"] });

        return loomId;
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Deployment failed";
        setError(msg);
        throw e;
      } finally {
        setDeploying(false);
      }
    },
    [meta, activeAccountIndex, queryClient]
  );

  return { deploy, deploying, error };
}

"use client";

import { useState, useCallback } from "react";
import { useWallet } from "./use-wallet";
import { useWalletStore } from "@/stores/wallet-store";
import * as signer from "@/lib/secure-signer";

export function useSignTransaction() {
  const { meta, activeAccountIndex } = useWallet();
  const [signing, setSigning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const signTransfer = useCallback(
    async (params: { to: string; amount: string; tokenId?: string; memo?: string; decimals?: number }) => {
      if (!meta) throw new Error("No wallet");
      setSigning(true);
      setError(null);
      try {
        const pw = useWalletStore.getState().sessionPassword ?? undefined;
        return await signer.signTransfer(meta, params, activeAccountIndex, pw);
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Signing failed";
        setError(msg);
        throw e;
      } finally {
        setSigning(false);
      }
    },
    [meta, activeAccountIndex]
  );

  const signNameRegistration = useCallback(
    async (name: string) => {
      if (!meta) throw new Error("No wallet");
      setSigning(true);
      setError(null);
      try {
        const pw = useWalletStore.getState().sessionPassword ?? undefined;
        return await signer.signNameRegistration(meta, name, activeAccountIndex, pw);
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Signing failed";
        setError(msg);
        throw e;
      } finally {
        setSigning(false);
      }
    },
    [meta, activeAccountIndex]
  );

  const signTokenDefinition = useCallback(
    async (params: { name: string; symbol: string; decimals: number; maxSupply: string; initialSupply?: string }) => {
      if (!meta) throw new Error("No wallet");
      setSigning(true);
      setError(null);
      try {
        const pw = useWalletStore.getState().sessionPassword ?? undefined;
        return await signer.signTokenDefinition(meta, params, activeAccountIndex, pw);
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Signing failed";
        setError(msg);
        throw e;
      } finally {
        setSigning(false);
      }
    },
    [meta, activeAccountIndex]
  );

  const signTokenMint = useCallback(
    async (params: { tokenId: string; to: string; amount: string; decimals?: number }) => {
      if (!meta) throw new Error("No wallet");
      setSigning(true);
      setError(null);
      try {
        const pw = useWalletStore.getState().sessionPassword ?? undefined;
        return await signer.signTokenMint(meta, params, activeAccountIndex, pw);
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Signing failed";
        setError(msg);
        throw e;
      } finally {
        setSigning(false);
      }
    },
    [meta, activeAccountIndex]
  );

  const signTokenBurn = useCallback(
    async (params: { tokenId: string; amount: string; decimals?: number }) => {
      if (!meta) throw new Error("No wallet");
      setSigning(true);
      setError(null);
      try {
        const pw = useWalletStore.getState().sessionPassword ?? undefined;
        return await signer.signTokenBurn(meta, params, activeAccountIndex, pw);
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Signing failed";
        setError(msg);
        throw e;
      } finally {
        setSigning(false);
      }
    },
    [meta, activeAccountIndex]
  );

  return {
    signing,
    error,
    signTransfer,
    signNameRegistration,
    signTokenDefinition,
    signTokenMint,
    signTokenBurn,
  };
}

"use client";

import { useEffect, useRef } from "react";
import { useWallet } from "@/hooks/use-wallet";
import { signChatEvent, getX25519PublicKey } from "@/lib/chat-signer";
import { getChatProfile, saveChatProfile } from "@/lib/chat-storage";
import { rpcCall } from "@/lib/rpc";
import type { SubmitResult } from "@norn-protocol/sdk";

/**
 * Automatically publishes a kind 30000 chat profile event on first chat use.
 * Contains the user's display name and X25519 public key for DM encryption.
 */
export function ChatProfilePublisher() {
  const { meta, activeAccount, activeAccountIndex } = useWallet();
  const publishedRef = useRef(false);

  useEffect(() => {
    if (!meta || !activeAccount || publishedRef.current) return;

    const pubkey = activeAccount.publicKeyHex;

    async function maybePublish() {
      // Check if we already published a profile
      const existing = await getChatProfile(pubkey);
      if (existing?.x25519PublicKey) {
        publishedRef.current = true;
        return;
      }

      try {
        const x25519PublicKey = await getX25519PublicKey(meta!, activeAccountIndex);

        const content = JSON.stringify({
          name: activeAccount!.label || undefined,
          x25519PublicKey,
          address: activeAccount!.address,
        });

        const event = await signChatEvent(meta!, 30000, content, [], activeAccountIndex);
        const result = await rpcCall<SubmitResult>("norn_publishChatEvent", [event]);

        if (result.success) {
          // Cache our own profile locally
          await saveChatProfile(pubkey, {
            pubkey,
            displayName: activeAccount!.label || undefined,
            x25519PublicKey,
            address: activeAccount!.address,
            updatedAt: Math.floor(Date.now() / 1000),
          });
          publishedRef.current = true;
        }
      } catch {
        // Non-fatal — will retry on next mount
      }
    }

    maybePublish();
  }, [meta, activeAccount, activeAccountIndex]);

  return null;
}

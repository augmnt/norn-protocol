"use client";

import { useEffect, useRef, useCallback } from "react";
import {
  subscribeChatEvents,
  verifyChatEvent,
  decryptDmContent,
  fromHex,
  type Subscription,
  type ChatEvent,
} from "@norn-protocol/sdk";
import { toast } from "sonner";
import { useNetworkStore } from "@/stores/network-store";
import { useChatStore } from "@/stores/chat-store";
import { useWallet } from "@/hooks/use-wallet";
import {
  appendChatMessage,
  getChatProfile,
  saveChatProfile,
  saveChannels,
  getChannels,
  type StoredMessage,
  type StoredChannel,
} from "@/lib/chat-storage";
import { truncateAddress } from "@/lib/format";

const MAX_RECONNECT_DELAY = 30_000;
const BASE_RECONNECT_DELAY = 1_000;

export function useChatSubscription() {
  const { activeAccount } = useWallet();
  const pubkeyHex = activeAccount?.publicKeyHex ?? null;
  const subsRef = useRef<Subscription | null>(null);
  const reconnectRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const attemptRef = useRef(0);
  const activeNetworkId = useNetworkStore((s) => s.activeNetworkId);

  const handleEvent = useCallback(
    async (event: ChatEvent) => {
      // Verify signature
      if (!verifyChatEvent(event)) return;

      const store = useChatStore.getState();

      if (event.kind === 30000) {
        // Profile event: extract display name and X25519 pubkey
        try {
          const profile = JSON.parse(event.content);
          await saveChatProfile(event.pubkey, {
            pubkey: event.pubkey,
            displayName: profile.name,
            x25519PublicKey: profile.x25519PublicKey,
            address: profile.address ?? "",
            nornName: profile.nornName,
            updatedAt: event.created_at,
          });
        } catch {
          // Ignore malformed profile events
        }
        return;
      }

      if (event.kind === 30002) {
        // Channel create event
        try {
          const channel = JSON.parse(event.content);
          const channelId = event.id;
          const newChannel: StoredChannel = {
            id: channelId,
            name: channel.name ?? "unnamed",
            description: channel.description ?? "",
            creator: event.pubkey,
            created_at: event.created_at,
          };
          const existing = await getChannels();
          if (!existing.some((c) => c.id === channelId)) {
            await saveChannels([...existing, newChannel]);
            store.addConversation({
              id: channelId,
              type: "channel",
              name: channel.name ?? "unnamed",
            });
          }
        } catch {
          // Ignore malformed channel events
        }
        return;
      }

      if (event.kind === 30003) {
        // Channel message
        const channelTag = event.tags.find((t) => t[0] === "c");
        if (!channelTag) return;
        const channelId = channelTag[1];

        const msg: StoredMessage = {
          id: event.id,
          pubkey: event.pubkey,
          created_at: event.created_at,
          kind: event.kind,
          tags: event.tags,
          content: event.content,
          sig: event.sig,
        };
        await appendChatMessage(channelId, msg);
        store.updateLastMessage(channelId, event.content.slice(0, 50), event.created_at);
        store.incrementUnread(channelId);
        return;
      }

      if (event.kind === 30001) {
        // DM
        const pTag = event.tags.find((t) => t[0] === "p");
        if (!pTag) return;
        const recipientPubkey = pTag[1];

        // Determine peer pubkey (the other party)
        const isIncoming = recipientPubkey === pubkeyHex;
        const peerPubkey = isIncoming ? event.pubkey : recipientPubkey;
        const dmConversationId = `dm:${peerPubkey}`;

        // Try to decrypt
        let decryptedContent: string | undefined;
        if (pubkeyHex) {
          try {
            const senderProfile = await getChatProfile(peerPubkey);
            if (senderProfile?.x25519PublicKey) {
              const nonceTag = event.tags.find((t) => t[0] === "nonce");
              if (nonceTag) {
                // We need the wallet private key to decrypt — store encrypted for now
                // Decryption will happen in the message display component
                decryptedContent = undefined;
              }
            }
          } catch {
            // Decryption deferred to display
          }
        }

        const msg: StoredMessage = {
          id: event.id,
          pubkey: event.pubkey,
          created_at: event.created_at,
          kind: event.kind,
          tags: event.tags,
          content: event.content,
          sig: event.sig,
          decryptedContent,
        };
        await appendChatMessage(dmConversationId, msg);

        // Ensure conversation exists
        const profile = await getChatProfile(peerPubkey);
        store.addConversation({
          id: dmConversationId,
          type: "dm",
          name: profile?.displayName ?? truncateAddress(`0x${peerPubkey.slice(0, 40)}`),
          peerPubkey,
        });
        store.updateLastMessage(
          dmConversationId,
          decryptedContent ?? "(encrypted)",
          event.created_at
        );
        store.incrementUnread(dmConversationId);

        // Toast for incoming DMs
        if (isIncoming && store.activeConversationId !== dmConversationId) {
          const senderName = profile?.displayName ?? truncateAddress(`0x${event.pubkey.slice(0, 40)}`);
          toast("New Message", {
            description: `${senderName} sent you a message`,
            duration: 4000,
          });
        }
      }
    },
    [pubkeyHex]
  );

  useEffect(() => {
    if (!pubkeyHex) return;

    let mounted = true;

    function connect() {
      subsRef.current?.unsubscribe();
      subsRef.current = null;

      const wsUrl =
        useNetworkStore.getState().wsUrl ??
        useNetworkStore.getState().customWsUrl ??
        "wss://seed.norn.network";

      const sub = subscribeChatEvents(
        {
          url: wsUrl,
          onOpen: () => {
            if (!mounted) return;
            attemptRef.current = 0;
          },
          onClose: () => {
            if (!mounted) return;
            scheduleReconnect();
          },
          onError: () => {},
        },
        (event) => {
          handleEvent(event);
        },
        pubkeyHex ?? undefined
      );

      subsRef.current = sub;
    }

    function scheduleReconnect() {
      if (!mounted) return;
      if (reconnectRef.current) clearTimeout(reconnectRef.current);
      const delay = Math.min(
        BASE_RECONNECT_DELAY * Math.pow(2, attemptRef.current),
        MAX_RECONNECT_DELAY
      );
      attemptRef.current++;
      reconnectRef.current = setTimeout(() => {
        if (mounted) connect();
      }, delay);
    }

    connect();

    return () => {
      mounted = false;
      if (reconnectRef.current) clearTimeout(reconnectRef.current);
      subsRef.current?.unsubscribe();
      subsRef.current = null;
    };
  }, [pubkeyHex, activeNetworkId, handleEvent]);
}

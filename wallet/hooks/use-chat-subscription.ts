"use client";

import { useEffect, useRef, useCallback } from "react";
import {
  subscribeChatEvents,
  verifyChatEvent,
  type Subscription,
  type ChatEvent,
} from "@norn-protocol/sdk";
import { toast } from "sonner";
import { useNetworkStore } from "@/stores/network-store";
import { useChatStore } from "@/stores/chat-store";
import { useWalletStore } from "@/stores/wallet-store";
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
import { decryptDmMessage } from "@/lib/chat-signer";
import { rpcCall } from "@/lib/rpc";
import { truncateAddress } from "@/lib/format";

const MAX_RECONNECT_DELAY = 30_000;
const BASE_RECONNECT_DELAY = 1_000;

export function useChatSubscription() {
  const { activeAccount } = useWallet();
  const pubkeyHex = activeAccount?.publicKeyHex ?? null;
  const subsRef = useRef<Subscription | null>(null);
  const reconnectRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const attemptRef = useRef(0);
  const hydratedRef = useRef(false);
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
        store.bumpMessageVersion();
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

        // Fetch profile early for decryption key
        const profile = await getChatProfile(peerPubkey);
        const nonceTag = event.tags.find((t) => t[0] === "nonce");

        const msg: StoredMessage = {
          id: event.id,
          pubkey: event.pubkey,
          created_at: event.created_at,
          kind: event.kind,
          tags: event.tags,
          content: event.content,
          sig: event.sig,
        };

        // Attempt decryption
        let preview = "(encrypted)";
        if (profile?.x25519PublicKey && nonceTag?.[1]) {
          try {
            const walletStore = useWalletStore.getState();
            const meta = walletStore.meta;
            const pw = walletStore.sessionPassword ?? undefined;
            if (meta) {
              const decrypted = await decryptDmMessage(
                meta,
                profile.x25519PublicKey,
                event.content,
                nonceTag[1],
                0,
                pw
              );
              msg.decryptedContent = decrypted;
              preview = decrypted.slice(0, 50);
            }
          } catch {
            // Decryption failed — show as encrypted
          }
        }

        await appendChatMessage(dmConversationId, msg);

        // Ensure conversation exists
        store.addConversation({
          id: dmConversationId,
          type: "dm",
          name: profile?.displayName ?? truncateAddress(`0x${peerPubkey.slice(0, 40)}`),
          peerPubkey,
        });
        store.updateLastMessage(dmConversationId, preview, event.created_at);
        store.incrementUnread(dmConversationId);
        store.bumpMessageVersion();

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

  // Hydrate: fetch existing channels, profiles, and recent messages from server
  const hydrate = useCallback(async () => {
    if (hydratedRef.current) return;
    hydratedRef.current = true;

    try {
      // Fetch all channel creation events
      const channels = await rpcCall<ChatEvent[]>("norn_getChatHistory", [
        { kinds: [30002], limit: 100 },
      ]);
      for (const event of channels) {
        await handleEvent(event);
      }

      // Fetch all profile events
      const profiles = await rpcCall<ChatEvent[]>("norn_getChatHistory", [
        { kinds: [30000], limit: 200 },
      ]);
      for (const event of profiles) {
        await handleEvent(event);
      }

      // Fetch recent channel messages for each known channel
      const store = useChatStore.getState();
      for (const conv of store.conversations) {
        if (conv.type === "channel") {
          const msgs = await rpcCall<ChatEvent[]>("norn_getChatHistory", [
            { kinds: [30003], channel_id: conv.id, limit: 100 },
          ]);
          for (const event of msgs) {
            await handleEvent(event);
          }
        }
      }

      // Fetch recent DMs for this pubkey
      if (pubkeyHex) {
        const dms = await rpcCall<ChatEvent[]>("norn_getChatHistory", [
          { kinds: [30001], pubkey: pubkeyHex, limit: 100 },
        ]);
        for (const event of dms) {
          await handleEvent(event);
        }
      }
    } catch {
      // Non-fatal — hydration is best-effort
    }
  }, [handleEvent, pubkeyHex]);

  useEffect(() => {
    if (!pubkeyHex) return;

    let mounted = true;
    hydratedRef.current = false;

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
            // Hydrate history on first connect
            hydrate();
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
  }, [pubkeyHex, activeNetworkId, handleEvent, hydrate]);
}

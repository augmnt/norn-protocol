"use client";

import { useEffect, useRef, useState, useCallback } from "react";
import { MessageBubble } from "./message-bubble";
import { getChatMessages, getChatProfile, type StoredMessage, type ChatProfile } from "@/lib/chat-storage";
import { useWallet } from "@/hooks/use-wallet";
import { useChatStore } from "@/stores/chat-store";

interface MessageListProps {
  conversationId: string;
}

export function MessageList({ conversationId }: MessageListProps) {
  const { activeAccount } = useWallet();
  const myPubkey = activeAccount?.publicKeyHex ?? "";
  const [messages, setMessages] = useState<StoredMessage[]>([]);
  const [profiles, setProfiles] = useState<Record<string, ChatProfile>>({});
  const scrollRef = useRef<HTMLDivElement>(null);
  const unreadCounts = useChatStore((s) => s.unreadCounts);

  const loadMessages = useCallback(async () => {
    const msgs = await getChatMessages(conversationId);
    setMessages(msgs);

    // Load profiles for senders we haven't seen
    const newProfiles: Record<string, ChatProfile> = {};
    for (const msg of msgs) {
      if (!profiles[msg.pubkey] && !newProfiles[msg.pubkey]) {
        const profile = await getChatProfile(msg.pubkey);
        if (profile) newProfiles[msg.pubkey] = profile;
      }
    }
    if (Object.keys(newProfiles).length > 0) {
      setProfiles((prev) => ({ ...prev, ...newProfiles }));
    }
  }, [conversationId]);

  // Load messages on mount and when unread changes (new message arrived)
  useEffect(() => {
    loadMessages();
  }, [loadMessages, unreadCounts[conversationId]]);

  // Auto-scroll to bottom when messages change
  useEffect(() => {
    const el = scrollRef.current;
    if (el) {
      el.scrollTop = el.scrollHeight;
    }
  }, [messages.length]);

  return (
    <div ref={scrollRef} className="flex-1 overflow-y-auto p-4 space-y-2">
      {messages.length === 0 && (
        <div className="flex items-center justify-center h-full">
          <p className="text-sm text-muted-foreground">No messages yet</p>
        </div>
      )}
      {messages.map((msg) => {
        const isOwn = msg.pubkey === myPubkey;
        const profile = profiles[msg.pubkey];
        const senderName = profile?.displayName ?? profile?.nornName ?? undefined;
        return (
          <MessageBubble
            key={msg.id}
            message={msg}
            isOwn={isOwn}
            senderName={senderName}
          />
        );
      })}
    </div>
  );
}

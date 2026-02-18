"use client";

import { useEffect, useRef, useState, useCallback } from "react";
import { ArrowDown } from "lucide-react";
import { MessageBubble } from "./message-bubble";
import { getChatMessages, getChatProfile, type StoredMessage, type ChatProfile } from "@/lib/chat-storage";
import { useWallet } from "@/hooks/use-wallet";
import { useChatStore } from "@/stores/chat-store";
import { Button } from "@/components/ui/button";

interface MessageListProps {
  conversationId: string;
}

/** Max seconds between messages from same sender to group them. */
const GROUP_THRESHOLD = 120;

function isSameDay(a: number, b: number): boolean {
  const da = new Date(a * 1000);
  const db = new Date(b * 1000);
  return da.toDateString() === db.toDateString();
}

function formatDateSeparator(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);

  if (date.toDateString() === today.toDateString()) return "Today";
  if (date.toDateString() === yesterday.toDateString()) return "Yesterday";
  return date.toLocaleDateString([], { weekday: "long", month: "long", day: "numeric" });
}

export function MessageList({ conversationId }: MessageListProps) {
  const { activeAccount } = useWallet();
  const myPubkey = activeAccount?.publicKeyHex ?? "";
  const [messages, setMessages] = useState<StoredMessage[]>([]);
  const [profiles, setProfiles] = useState<Record<string, ChatProfile>>({});
  const scrollRef = useRef<HTMLDivElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const [showScrollBtn, setShowScrollBtn] = useState(false);
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

  // Auto-scroll to bottom when messages change (if near bottom)
  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    const isNearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 100;
    if (isNearBottom || messages.length <= 1) {
      bottomRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  }, [messages.length]);

  // Track scroll position for "scroll to bottom" button
  const handleScroll = useCallback(() => {
    const el = scrollRef.current;
    if (!el) return;
    const distFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
    setShowScrollBtn(distFromBottom > 200);
  }, []);

  const scrollToBottom = useCallback(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, []);

  return (
    <div className="relative flex-1 overflow-hidden">
      <div
        ref={scrollRef}
        className="h-full overflow-y-auto px-4 py-2"
        onScroll={handleScroll}
      >
        {messages.length === 0 && (
          <div className="flex items-center justify-center h-full">
            <p className="text-sm text-muted-foreground">No messages yet. Say something!</p>
          </div>
        )}
        {messages.map((msg, i) => {
          const prev = i > 0 ? messages[i - 1] : null;
          const isOwn = msg.pubkey === myPubkey;
          const profile = profiles[msg.pubkey];
          const senderName = profile?.displayName ?? profile?.nornName ?? undefined;

          // Date separator
          const showDateSep = !prev || !isSameDay(prev.created_at, msg.created_at);

          // Message grouping: same sender within threshold
          const isGrouped =
            !showDateSep &&
            prev !== null &&
            prev.pubkey === msg.pubkey &&
            msg.created_at - prev.created_at < GROUP_THRESHOLD;

          return (
            <div key={msg.id}>
              {showDateSep && (
                <div className="flex items-center gap-3 my-4">
                  <div className="flex-1 border-t border-border" />
                  <span className="text-[10px] uppercase tracking-wider text-muted-foreground font-medium">
                    {formatDateSeparator(msg.created_at)}
                  </span>
                  <div className="flex-1 border-t border-border" />
                </div>
              )}
              <MessageBubble
                message={msg}
                isOwn={isOwn}
                senderName={senderName}
                isGrouped={isGrouped}
              />
            </div>
          );
        })}
        <div ref={bottomRef} />
      </div>

      {/* Scroll to bottom button */}
      {showScrollBtn && (
        <div className="absolute bottom-3 left-1/2 -translate-x-1/2">
          <Button
            variant="secondary"
            size="sm"
            className="rounded-full shadow-lg gap-1.5 h-8 px-3"
            onClick={scrollToBottom}
          >
            <ArrowDown className="h-3.5 w-3.5" />
            New messages
          </Button>
        </div>
      )}
    </div>
  );
}

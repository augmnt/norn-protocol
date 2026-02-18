"use client";

import { cn } from "@/lib/utils";
import { Hash } from "lucide-react";
import { Identicon } from "@/components/ui/identicon";
import type { ConversationSummary } from "@/stores/chat-store";

interface ConversationItemProps {
  conversation: ConversationSummary;
  active: boolean;
  unreadCount: number;
  onClick: () => void;
}

function formatTime(timestamp?: number): string {
  if (!timestamp) return "";
  const now = Math.floor(Date.now() / 1000);
  const diff = now - timestamp;

  if (diff < 60) return "now";
  if (diff < 3600) return `${Math.floor(diff / 60)}m`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h`;

  const date = new Date(timestamp * 1000);
  return date.toLocaleDateString([], { month: "short", day: "numeric" });
}

export function ConversationItem({ conversation, active, unreadCount, onClick }: ConversationItemProps) {
  const dmAddress = conversation.peerPubkey
    ? `0x${conversation.peerPubkey.slice(0, 40)}`
    : "0x0000000000000000000000000000000000000000";

  return (
    <button
      onClick={onClick}
      className={cn(
        "flex items-center gap-2.5 w-full rounded-md px-3 py-2 text-left transition-colors",
        active
          ? "bg-accent text-accent-foreground"
          : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
      )}
    >
      <div className="flex items-center justify-center h-8 w-8 rounded-full shrink-0 overflow-hidden">
        {conversation.type === "channel" ? (
          <div className="flex items-center justify-center h-8 w-8 rounded-full bg-secondary">
            <Hash className="h-3.5 w-3.5" />
          </div>
        ) : (
          <Identicon address={dmAddress} size={32} />
        )}
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center justify-between gap-1">
          <span className={cn(
            "text-sm truncate",
            unreadCount > 0 && "font-medium text-foreground",
            conversation.type === "channel" && "font-mono"
          )}>
            {conversation.type === "channel" ? `# ${conversation.name}` : conversation.name}
          </span>
          {conversation.lastMessageAt && (
            <span className="text-[10px] text-muted-foreground shrink-0">
              {formatTime(conversation.lastMessageAt)}
            </span>
          )}
        </div>
        {conversation.lastMessage && (
          <p className={cn(
            "text-xs truncate mt-0.5",
            unreadCount > 0 ? "text-foreground/70" : "text-muted-foreground"
          )}>
            {conversation.lastMessage}
          </p>
        )}
      </div>
      {unreadCount > 0 && (
        <span className="bg-norn text-white text-[10px] rounded-full h-5 min-w-5 flex items-center justify-center px-1.5 shrink-0 font-medium">
          {unreadCount > 99 ? "99+" : unreadCount}
        </span>
      )}
    </button>
  );
}

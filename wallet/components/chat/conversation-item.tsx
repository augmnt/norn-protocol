"use client";

import { cn } from "@/lib/utils";
import { Hash, User } from "lucide-react";
import type { ConversationSummary } from "@/stores/chat-store";

interface ConversationItemProps {
  conversation: ConversationSummary;
  active: boolean;
  unreadCount: number;
  onClick: () => void;
}

function formatTime(timestamp?: number): string {
  if (!timestamp) return "";
  const date = new Date(timestamp * 1000);
  const now = new Date();
  const isToday = date.toDateString() === now.toDateString();
  if (isToday) {
    return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }
  return date.toLocaleDateString([], { month: "short", day: "numeric" });
}

export function ConversationItem({ conversation, active, unreadCount, onClick }: ConversationItemProps) {
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
      <div className="flex items-center justify-center h-8 w-8 rounded-full bg-secondary shrink-0">
        {conversation.type === "channel" ? (
          <Hash className="h-3.5 w-3.5" />
        ) : (
          <User className="h-3.5 w-3.5" />
        )}
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center justify-between gap-1">
          <span className={cn(
            "text-sm truncate",
            conversation.type === "channel" && "font-mono"
          )}>
            {conversation.type === "channel" ? `# ${conversation.name}` : conversation.name}
          </span>
          {conversation.lastMessageAt && (
            <span className="text-xs text-muted-foreground shrink-0">
              {formatTime(conversation.lastMessageAt)}
            </span>
          )}
        </div>
        {conversation.lastMessage && (
          <p className="text-xs text-muted-foreground truncate mt-0.5">
            {conversation.lastMessage}
          </p>
        )}
      </div>
      {unreadCount > 0 && (
        <span className="bg-norn text-white text-xs rounded-full h-5 min-w-5 flex items-center justify-center px-1.5 shrink-0">
          {unreadCount > 99 ? "99+" : unreadCount}
        </span>
      )}
    </button>
  );
}

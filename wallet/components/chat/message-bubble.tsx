"use client";

import { cn } from "@/lib/utils";
import { Identicon } from "@/components/ui/identicon";
import type { StoredMessage } from "@/lib/chat-storage";

interface MessageBubbleProps {
  message: StoredMessage;
  isOwn: boolean;
  senderName?: string;
  /** Whether this message continues a group from the same sender. */
  isGrouped?: boolean;
}

function formatRelativeTime(timestamp: number): string {
  const now = Math.floor(Date.now() / 1000);
  const diff = now - timestamp;

  if (diff < 60) return "just now";
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;

  const date = new Date(timestamp * 1000);
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);

  if (date.toDateString() === yesterday.toDateString()) return "yesterday";

  return date.toLocaleDateString([], { month: "short", day: "numeric" });
}

export function MessageBubble({ message, isOwn, senderName, isGrouped }: MessageBubbleProps) {
  const displayContent = message.decryptedContent ?? message.content;
  const isEncrypted = message.kind === 30001 && !message.decryptedContent;
  // Use first 40 chars of pubkey as a pseudo-address for identicon
  const identiconAddress = `0x${message.pubkey.slice(0, 40)}`;

  if (isOwn) {
    return (
      <div className={cn("flex justify-end", isGrouped ? "mt-0.5" : "mt-3")}>
        <div className="flex flex-col items-end max-w-[85%] sm:max-w-[70%]">
          <div className="rounded-lg p-3 text-sm break-words bg-norn/10">
            {isEncrypted ? (
              <span className="text-muted-foreground italic">Encrypted message</span>
            ) : (
              <p className="whitespace-pre-wrap">{displayContent}</p>
            )}
          </div>
          {!isGrouped && (
            <span className="text-[10px] text-muted-foreground mt-1 px-1">
              {formatRelativeTime(message.created_at)}
            </span>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className={cn("flex gap-2.5", isGrouped ? "mt-0.5 pl-[38px]" : "mt-3")}>
      {!isGrouped && (
        <div className="shrink-0 mt-0.5">
          <Identicon address={identiconAddress} size={28} className="rounded-full" />
        </div>
      )}
      <div className="flex flex-col max-w-[85%] sm:max-w-[70%]">
        {!isGrouped && (
          <div className="flex items-baseline gap-2 mb-0.5 px-1">
            <span className="text-xs font-medium">
              {senderName ?? `${message.pubkey.slice(0, 8)}...`}
            </span>
            <span className="text-[10px] text-muted-foreground">
              {formatRelativeTime(message.created_at)}
            </span>
          </div>
        )}
        <div className="rounded-lg p-3 text-sm break-words bg-secondary">
          {isEncrypted ? (
            <span className="text-muted-foreground italic">Encrypted message</span>
          ) : (
            <p className="whitespace-pre-wrap">{displayContent}</p>
          )}
        </div>
      </div>
    </div>
  );
}

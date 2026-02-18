"use client";

import { cn } from "@/lib/utils";
import type { StoredMessage } from "@/lib/chat-storage";

interface MessageBubbleProps {
  message: StoredMessage;
  isOwn: boolean;
  senderName?: string;
}

function formatTime(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function MessageBubble({ message, isOwn, senderName }: MessageBubbleProps) {
  const displayContent = message.decryptedContent ?? message.content;
  const isEncrypted = message.kind === 30001 && !message.decryptedContent;

  return (
    <div
      className={cn(
        "flex flex-col max-w-[85%] sm:max-w-[70%] animate-slide-in",
        isOwn ? "ml-auto items-end" : "mr-auto items-start"
      )}
    >
      {!isOwn && senderName && (
        <span className="text-xs text-muted-foreground font-mono mb-0.5 px-1">
          {senderName}
        </span>
      )}
      <div
        className={cn(
          "rounded-lg p-3 text-sm break-words",
          isOwn ? "bg-norn/10" : "bg-secondary"
        )}
      >
        {isEncrypted ? (
          <span className="text-muted-foreground italic">Encrypted message</span>
        ) : (
          <p className="whitespace-pre-wrap">{displayContent}</p>
        )}
      </div>
      <span className="text-xs text-muted-foreground mt-0.5 px-1">
        {formatTime(message.created_at)}
      </span>
    </div>
  );
}

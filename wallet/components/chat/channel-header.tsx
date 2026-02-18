"use client";

import { ArrowLeft, Hash } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Identicon } from "@/components/ui/identicon";
import type { ConversationSummary } from "@/stores/chat-store";

interface ChannelHeaderProps {
  conversation: ConversationSummary;
  onBack?: () => void;
}

export function ChannelHeader({ conversation, onBack }: ChannelHeaderProps) {
  const dmAddress = conversation.peerPubkey
    ? `0x${conversation.peerPubkey.slice(0, 40)}`
    : "";

  return (
    <div className="flex items-center gap-2 px-4 py-3 border-b shrink-0">
      {onBack && (
        <Button variant="ghost" size="icon" className="h-8 w-8 md:hidden" onClick={onBack}>
          <ArrowLeft className="h-4 w-4" />
        </Button>
      )}
      <div className="flex items-center justify-center h-8 w-8 rounded-full shrink-0 overflow-hidden">
        {conversation.type === "channel" ? (
          <div className="flex items-center justify-center h-8 w-8 rounded-full bg-secondary">
            <Hash className="h-4 w-4 text-muted-foreground" />
          </div>
        ) : (
          <Identicon address={dmAddress} size={32} />
        )}
      </div>
      <div className="flex-1 min-w-0">
        <h2 className="text-sm font-medium truncate">
          {conversation.type === "channel"
            ? `# ${conversation.name}`
            : conversation.name}
        </h2>
        {conversation.type === "dm" && conversation.peerPubkey && (
          <p className="text-[10px] text-muted-foreground font-mono truncate">
            {conversation.peerPubkey.slice(0, 8)}...{conversation.peerPubkey.slice(-8)}
          </p>
        )}
      </div>
    </div>
  );
}

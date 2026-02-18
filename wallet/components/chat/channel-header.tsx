"use client";

import { ArrowLeft, Hash, User } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useChatStore, type ConversationSummary } from "@/stores/chat-store";

interface ChannelHeaderProps {
  conversation: ConversationSummary;
  onBack?: () => void;
}

export function ChannelHeader({ conversation, onBack }: ChannelHeaderProps) {
  return (
    <div className="flex items-center gap-2 px-4 py-3 border-b shrink-0">
      {onBack && (
        <Button variant="ghost" size="icon" className="h-8 w-8 md:hidden" onClick={onBack}>
          <ArrowLeft className="h-4 w-4" />
        </Button>
      )}
      <div className="flex items-center justify-center h-8 w-8 rounded-full bg-secondary shrink-0">
        {conversation.type === "channel" ? (
          <Hash className="h-4 w-4 text-muted-foreground" />
        ) : (
          <User className="h-4 w-4 text-muted-foreground" />
        )}
      </div>
      <div className="flex-1 min-w-0">
        <h2 className="text-sm font-medium truncate">
          {conversation.type === "channel"
            ? `# ${conversation.name}`
            : conversation.name}
        </h2>
        {conversation.type === "dm" && conversation.peerPubkey && (
          <p className="text-xs text-muted-foreground font-mono truncate">
            {conversation.peerPubkey.slice(0, 8)}...{conversation.peerPubkey.slice(-8)}
          </p>
        )}
      </div>
    </div>
  );
}

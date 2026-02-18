"use client";

import { useState, useCallback } from "react";
import { useChatStore } from "@/stores/chat-store";
import { useChatSubscription } from "@/hooks/use-chat-subscription";
import { ConversationList } from "@/components/chat/conversation-list";
import { ChannelHeader } from "@/components/chat/channel-header";
import { MessageList } from "@/components/chat/message-list";
import { MessageInput } from "@/components/chat/message-input";
import { ChatEmptyState } from "@/components/chat/chat-empty-state";
import { ChatProfilePublisher } from "@/components/chat/chat-profile-publisher";

export default function ChatPage() {
  useChatSubscription();

  const activeId = useChatStore((s) => s.activeConversationId);
  const activeType = useChatStore((s) => s.activeConversationType);
  const conversations = useChatStore((s) => s.conversations);
  const setActive = useChatStore((s) => s.setActiveConversation);

  const activeConversation = activeId
    ? conversations.find((c) => c.id === activeId)
    : null;

  // Mobile: toggle between list and detail views
  const [showDetail, setShowDetail] = useState(false);

  const handleSelectConversation = useCallback(() => {
    setShowDetail(true);
  }, []);

  const handleBack = useCallback(() => {
    setShowDetail(false);
    setActive(null, null);
  }, [setActive]);

  return (
    <>
      <ChatProfilePublisher />
      <div className="flex h-full">
        {/* Sidebar: conversation list */}
        <div
          className={
            showDetail && activeConversation
              ? "hidden md:flex md:w-[280px] md:border-r md:flex-col md:shrink-0"
              : "flex flex-col w-full md:w-[280px] md:border-r md:shrink-0"
          }
        >
          <ConversationList onSelect={handleSelectConversation} />
        </div>

        {/* Main area: messages */}
        <div
          className={
            showDetail && activeConversation
              ? "flex flex-col flex-1 min-w-0"
              : "hidden md:flex md:flex-col md:flex-1 md:min-w-0"
          }
        >
          {activeConversation ? (
            <>
              <ChannelHeader
                conversation={activeConversation}
                onBack={handleBack}
              />
              <MessageList conversationId={activeConversation.id} />
              <MessageInput
                conversationId={activeConversation.id}
                conversationType={activeConversation.type}
                peerPubkey={activeConversation.peerPubkey}
              />
            </>
          ) : (
            <ChatEmptyState />
          )}
        </div>
      </div>
    </>
  );
}

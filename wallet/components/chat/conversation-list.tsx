"use client";

import { useState } from "react";
import { Plus, Search } from "lucide-react";
import { useChatStore } from "@/stores/chat-store";
import { ConversationItem } from "./conversation-item";
import { CreateChannelDialog } from "./create-channel-dialog";
import { NewDmDialog } from "./new-dm-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

interface ConversationListProps {
  onSelect?: () => void;
}

export function ConversationList({ onSelect }: ConversationListProps) {
  const conversations = useChatStore((s) => s.conversations);
  const activeId = useChatStore((s) => s.activeConversationId);
  const unreadCounts = useChatStore((s) => s.unreadCounts);
  const setActive = useChatStore((s) => s.setActiveConversation);
  const [search, setSearch] = useState("");
  const [showCreateChannel, setShowCreateChannel] = useState(false);
  const [showNewDm, setShowNewDm] = useState(false);

  const filtered = search
    ? conversations.filter((c) =>
        c.name.toLowerCase().includes(search.toLowerCase())
      )
    : conversations;

  const channels = filtered.filter((c) => c.type === "channel");
  const dms = filtered.filter((c) => c.type === "dm");

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 p-3 border-b">
        <div className="relative flex-1">
          <Search className="absolute left-2.5 top-2.5 h-3.5 w-3.5 text-muted-foreground" />
          <Input
            placeholder="Search..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-8 h-8 text-sm"
          />
        </div>
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 shrink-0"
          onClick={() => setShowCreateChannel(true)}
          title="New channel"
        >
          <Plus className="h-4 w-4" />
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto p-2 space-y-4">
        {channels.length > 0 && (
          <div>
            <div className="flex items-center justify-between px-2 mb-1">
              <span className="text-xs uppercase tracking-wider text-muted-foreground font-medium">
                Channels
              </span>
            </div>
            <div className="space-y-0.5">
              {channels.map((c) => (
                <ConversationItem
                  key={c.id}
                  conversation={c}
                  active={activeId === c.id}
                  unreadCount={unreadCounts[c.id] ?? 0}
                  onClick={() => {
                    setActive(c.id, "channel");
                    onSelect?.();
                  }}
                />
              ))}
            </div>
          </div>
        )}

        {dms.length > 0 && (
          <div>
            <div className="flex items-center justify-between px-2 mb-1">
              <span className="text-xs uppercase tracking-wider text-muted-foreground font-medium">
                Direct Messages
              </span>
            </div>
            <div className="space-y-0.5">
              {dms.map((c) => (
                <ConversationItem
                  key={c.id}
                  conversation={c}
                  active={activeId === c.id}
                  unreadCount={unreadCounts[c.id] ?? 0}
                  onClick={() => {
                    setActive(c.id, "dm");
                    onSelect?.();
                  }}
                />
              ))}
            </div>
          </div>
        )}

        {filtered.length === 0 && (
          <div className="flex flex-col items-center justify-center py-8 text-center">
            <p className="text-sm text-muted-foreground">
              {search ? "No conversations match" : "No conversations yet"}
            </p>
            {!search && (
              <div className="flex gap-2 mt-3">
                <Button variant="outline" size="sm" onClick={() => setShowCreateChannel(true)}>
                  New Channel
                </Button>
                <Button variant="outline" size="sm" onClick={() => setShowNewDm(true)}>
                  New DM
                </Button>
              </div>
            )}
          </div>
        )}
      </div>

      <div className="p-2 border-t">
        <Button
          variant="ghost"
          size="sm"
          className="w-full justify-start text-muted-foreground"
          onClick={() => setShowNewDm(true)}
        >
          <Plus className="h-3.5 w-3.5 mr-2" />
          New Message
        </Button>
      </div>

      <CreateChannelDialog open={showCreateChannel} onOpenChange={setShowCreateChannel} />
      <NewDmDialog open={showNewDm} onOpenChange={setShowNewDm} />
    </div>
  );
}

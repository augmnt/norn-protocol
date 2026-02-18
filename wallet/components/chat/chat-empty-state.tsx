"use client";

import { MessageSquare } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";

export function ChatEmptyState() {
  return (
    <EmptyState
      icon={MessageSquare}
      title="No conversation selected"
      description="Select a channel or start a DM to begin chatting"
      className="h-full"
    />
  );
}

"use client";

import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { useWallet } from "@/hooks/use-wallet";
import { useChatStore } from "@/stores/chat-store";
import { signChatEvent } from "@/lib/chat-signer";
import { rpcCall } from "@/lib/rpc";
import { toast } from "sonner";
import type { SubmitResult } from "@norn-protocol/sdk";

interface CreateChannelDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function CreateChannelDialog({ open, onOpenChange }: CreateChannelDialogProps) {
  const { meta, activeAccountIndex } = useWallet();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [creating, setCreating] = useState(false);

  const handleCreate = async () => {
    const trimmed = name.trim();
    if (!trimmed || !meta) return;

    setCreating(true);
    try {
      const content = JSON.stringify({
        name: trimmed,
        description: description.trim(),
      });
      const event = await signChatEvent(meta, 30002, content, [], activeAccountIndex);

      const result = await rpcCall<SubmitResult>("norn_publishChatEvent", [event]);
      if (!result.success) {
        toast.error("Failed to create channel", { description: result.reason });
        return;
      }

      // Add to local store immediately
      useChatStore.getState().addConversation({
        id: event.id,
        type: "channel",
        name: trimmed,
      });

      setName("");
      setDescription("");
      onOpenChange(false);

      // Select the new channel
      useChatStore.getState().setActiveConversation(event.id, "channel");
    } catch (err) {
      toast.error("Failed to create channel", {
        description: err instanceof Error ? err.message : "Unknown error",
      });
    } finally {
      setCreating(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create Channel</DialogTitle>
          <DialogDescription>Create a public channel for group messaging.</DialogDescription>
        </DialogHeader>
        <div className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="channel-name">Channel Name</Label>
            <Input
              id="channel-name"
              placeholder="general"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="font-mono"
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="channel-desc">Description (optional)</Label>
            <Input
              id="channel-desc"
              placeholder="What's this channel about?"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleCreate} disabled={!name.trim() || creating}>
            {creating ? "Creating..." : "Create"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

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
import { useChatStore } from "@/stores/chat-store";
import { getChatProfile } from "@/lib/chat-storage";
import { rpcCall } from "@/lib/rpc";
import { toast } from "sonner";
import type { NameResolution } from "@norn-protocol/sdk";
import { strip0x } from "@/lib/format";

interface NewDmDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function NewDmDialog({ open, onOpenChange }: NewDmDialogProps) {
  const [input, setInput] = useState("");
  const [resolving, setResolving] = useState(false);

  const handleStart = async () => {
    const trimmed = input.trim();
    if (!trimmed) return;

    setResolving(true);
    try {
      let pubkey = trimmed;
      let displayName = trimmed;

      // Check if it's a Norn name (contains .norn or no hex chars)
      if (trimmed.includes(".") || !/^[0-9a-fA-F]+$/.test(strip0x(trimmed))) {
        // Try to resolve as a Norn name
        const resolution = await rpcCall<NameResolution | null>("norn_resolveName", [trimmed]);
        if (!resolution) {
          toast.error("Name not found", { description: `Could not resolve "${trimmed}"` });
          return;
        }
        displayName = trimmed;
        // We need the pubkey, not the address — check profile cache
        // For now, use the address owner as a display hint
      }

      // Validate hex pubkey (64 chars = 32 bytes)
      const cleanPubkey = strip0x(pubkey);
      if (!/^[0-9a-fA-F]{64}$/.test(cleanPubkey)) {
        toast.error("Invalid pubkey", {
          description: "Enter a 32-byte hex pubkey or a Norn name",
        });
        return;
      }

      const dmConversationId = `dm:${cleanPubkey}`;
      const profile = await getChatProfile(cleanPubkey);

      useChatStore.getState().addConversation({
        id: dmConversationId,
        type: "dm",
        name: profile?.displayName ?? displayName,
        peerPubkey: cleanPubkey,
      });
      useChatStore.getState().setActiveConversation(dmConversationId, "dm");

      if (!profile?.x25519PublicKey) {
        toast("Note", {
          description: "Waiting for recipient's chat profile to enable encryption",
          duration: 5000,
        });
      }

      setInput("");
      onOpenChange(false);
    } catch (err) {
      toast.error("Failed to start DM", {
        description: err instanceof Error ? err.message : "Unknown error",
      });
    } finally {
      setResolving(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>New Direct Message</DialogTitle>
          <DialogDescription>
            Enter a public key (hex) to start an encrypted conversation.
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-2">
          <Label htmlFor="dm-pubkey">Recipient Public Key</Label>
          <Input
            id="dm-pubkey"
            placeholder="64-character hex public key"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            className="font-mono text-xs"
          />
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleStart} disabled={!input.trim() || resolving}>
            {resolving ? "Looking up..." : "Start Chat"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

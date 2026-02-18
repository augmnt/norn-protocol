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
import { strip0x, truncateAddress } from "@/lib/format";
import type { NameResolution, ThreadInfo } from "@norn-protocol/sdk";

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
      let pubkey: string | null = null;
      let displayName: string = trimmed;

      const cleaned = strip0x(trimmed);
      const isHex64 = /^[0-9a-fA-F]{64}$/.test(cleaned);
      const isAddress = /^[0-9a-fA-F]{40}$/.test(cleaned);

      if (isHex64) {
        // Direct 32-byte pubkey
        pubkey = cleaned;
      } else if (isAddress) {
        // Address — look up the thread to get the owner pubkey
        pubkey = await resolveAddressToPubkey(trimmed);
        if (!pubkey) {
          toast.error("Account not found", {
            description: `No account registered for address ${truncateAddress(trimmed)}`,
          });
          return;
        }
        displayName = truncateAddress(trimmed);
      } else {
        // Assume it's a Norn name
        const resolution = await rpcCall<NameResolution | null>("norn_resolveName", [trimmed]);
        if (!resolution) {
          toast.error("Name not found", {
            description: `Could not resolve "${trimmed}"`,
          });
          return;
        }
        displayName = trimmed;
        pubkey = await resolveAddressToPubkey(resolution.owner);
        if (!pubkey) {
          toast.error("Account not found", {
            description: `Resolved "${trimmed}" to ${truncateAddress(resolution.owner)} but no pubkey found`,
          });
          return;
        }
      }

      const dmConversationId = `dm:${pubkey}`;
      const profile = await getChatProfile(pubkey);

      useChatStore.getState().addConversation({
        id: dmConversationId,
        type: "dm",
        name: profile?.displayName ?? profile?.nornName ?? displayName,
        peerPubkey: pubkey,
      });
      useChatStore.getState().setActiveConversation(dmConversationId, "dm");

      if (!profile?.x25519PublicKey) {
        toast("Waiting for encryption key", {
          description: "The recipient hasn't published their chat profile yet. Messages will be available once they open Chat.",
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
            Enter a Norn name, address, or public key to start an encrypted conversation.
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-2">
          <Label htmlFor="dm-recipient">Recipient</Label>
          <Input
            id="dm-recipient"
            placeholder="alice.norn, 0x557d..., or pubkey hex"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                handleStart();
              }
            }}
            className="font-mono text-xs"
          />
          <p className="text-xs text-muted-foreground">
            Accepts a Norn name, 0x address, or 64-character public key
          </p>
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

async function resolveAddressToPubkey(address: string): Promise<string | null> {
  const hex = address.startsWith("0x") ? address : `0x${address}`;
  const thread = await rpcCall<ThreadInfo | null>("norn_getThread", [hex]);
  if (thread?.owner && thread.owner.length === 64) {
    return thread.owner;
  }
  return null;
}

"use client";

import { useState, useRef, useCallback } from "react";
import { Send } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { useWallet } from "@/hooks/use-wallet";
import { useChatStore } from "@/stores/chat-store";
import { signChatEvent, signEncryptedDm } from "@/lib/chat-signer";
import { getChatProfile } from "@/lib/chat-storage";
import { rpcCall } from "@/lib/rpc";
import { fromHex, type ChatEvent, type SubmitResult } from "@norn-protocol/sdk";
import { toast } from "sonner";

interface MessageInputProps {
  conversationId: string;
  conversationType: "channel" | "dm";
  peerPubkey?: string;
}

export function MessageInput({ conversationId, conversationType, peerPubkey }: MessageInputProps) {
  const { meta, activeAccountIndex } = useWallet();
  const [text, setText] = useState("");
  const [sending, setSending] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleSend = useCallback(async () => {
    const message = text.trim();
    if (!message || !meta || sending) return;

    setSending(true);
    try {
      let event: ChatEvent;

      if (conversationType === "channel") {
        const tags: string[][] = [["c", conversationId]];
        event = await signChatEvent(meta, 30003, message, tags, activeAccountIndex);
      } else if (conversationType === "dm" && peerPubkey) {
        const profile = await getChatProfile(peerPubkey);
        if (!profile?.x25519PublicKey) {
          toast.error("Cannot send encrypted message", {
            description: "Recipient hasn't published their chat profile yet",
          });
          return;
        }
        event = await signEncryptedDm(
          meta,
          fromHex(profile.x25519PublicKey),
          message,
          peerPubkey,
          activeAccountIndex
        );
      } else {
        return;
      }

      const result = await rpcCall<SubmitResult>("norn_publishChatEvent", [event]);
      if (!result.success) {
        toast.error("Failed to send", { description: result.reason });
        return;
      }

      setText("");
      textareaRef.current?.focus();
    } catch (err) {
      toast.error("Failed to send message", {
        description: err instanceof Error ? err.message : "Unknown error",
      });
    } finally {
      setSending(false);
    }
  }, [text, meta, sending, conversationId, conversationType, peerPubkey, activeAccountIndex]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="p-3 border-t shrink-0">
      <div className="flex items-end gap-2 bg-secondary rounded-lg border border-border p-2">
        <Textarea
          ref={textareaRef}
          value={text}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={
            conversationType === "dm" ? "Send encrypted message..." : "Type a message..."
          }
          className="min-h-[36px] max-h-[120px] resize-none border-0 bg-transparent p-1 text-sm focus-visible:ring-0 focus-visible:ring-offset-0"
          rows={1}
        />
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 shrink-0"
          onClick={handleSend}
          disabled={!text.trim() || sending}
        >
          <Send className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}

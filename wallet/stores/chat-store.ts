"use client";

import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface ConversationSummary {
  id: string;
  type: "channel" | "dm";
  name: string;
  lastMessage?: string;
  lastMessageAt?: number;
  /** For DMs: the other party's pubkey */
  peerPubkey?: string;
}

interface ChatState {
  activeConversationId: string | null;
  activeConversationType: "channel" | "dm" | null;
  conversations: ConversationSummary[];
  unreadCounts: Record<string, number>;

  setActiveConversation: (id: string | null, type: "channel" | "dm" | null) => void;
  addConversation: (summary: ConversationSummary) => void;
  updateLastMessage: (conversationId: string, preview: string, timestamp: number) => void;
  incrementUnread: (conversationId: string) => void;
  clearUnread: (conversationId: string) => void;
  removeConversation: (conversationId: string) => void;
}

export const useChatStore = create<ChatState>()(
  persist(
    (set, get) => ({
      activeConversationId: null,
      activeConversationType: null,
      conversations: [],
      unreadCounts: {},

      setActiveConversation: (id, type) => {
        set({ activeConversationId: id, activeConversationType: type });
        if (id) {
          // Clear unread when opening a conversation
          set((state) => ({
            unreadCounts: { ...state.unreadCounts, [id]: 0 },
          }));
        }
      },

      addConversation: (summary) =>
        set((state) => {
          const exists = state.conversations.some((c) => c.id === summary.id);
          if (exists) return state;
          return {
            conversations: [summary, ...state.conversations],
          };
        }),

      updateLastMessage: (conversationId, preview, timestamp) =>
        set((state) => ({
          conversations: state.conversations
            .map((c) =>
              c.id === conversationId
                ? { ...c, lastMessage: preview, lastMessageAt: timestamp }
                : c
            )
            .sort((a, b) => (b.lastMessageAt ?? 0) - (a.lastMessageAt ?? 0)),
        })),

      incrementUnread: (conversationId) =>
        set((state) => {
          // Don't increment if this is the active conversation
          if (state.activeConversationId === conversationId) return state;
          return {
            unreadCounts: {
              ...state.unreadCounts,
              [conversationId]: (state.unreadCounts[conversationId] ?? 0) + 1,
            },
          };
        }),

      clearUnread: (conversationId) =>
        set((state) => ({
          unreadCounts: { ...state.unreadCounts, [conversationId]: 0 },
        })),

      removeConversation: (conversationId) =>
        set((state) => ({
          conversations: state.conversations.filter((c) => c.id !== conversationId),
          unreadCounts: Object.fromEntries(
            Object.entries(state.unreadCounts).filter(([k]) => k !== conversationId)
          ),
        })),
    }),
    {
      name: "norn-wallet-chat",
      partialize: (state) => ({
        conversations: state.conversations,
        unreadCounts: state.unreadCounts,
      }),
    }
  )
);

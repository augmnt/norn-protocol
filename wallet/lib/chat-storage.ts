"use client";

import { get, set } from "idb-keyval";

export interface StoredMessage {
  id: string;
  pubkey: string;
  created_at: number;
  kind: number;
  tags: string[][];
  content: string;
  sig: string;
  decryptedContent?: string;
}

export interface StoredChannel {
  id: string;
  name: string;
  description: string;
  creator: string;
  created_at: number;
  lastMessageAt?: number;
}

export interface ChatProfile {
  pubkey: string;
  displayName?: string;
  x25519PublicKey: string;
  address: string;
  nornName?: string;
  updatedAt: number;
}

const MESSAGES_PREFIX = "norn-chat-msgs:";
const CHANNELS_KEY = "norn-chat-channels";
const PROFILE_PREFIX = "norn-chat-profile:";
const LAST_READ_PREFIX = "norn-chat-read:";

export async function getChatMessages(conversationId: string): Promise<StoredMessage[]> {
  try {
    return (await get<StoredMessage[]>(MESSAGES_PREFIX + conversationId)) ?? [];
  } catch {
    return [];
  }
}

export async function saveChatMessages(conversationId: string, messages: StoredMessage[]): Promise<void> {
  await set(MESSAGES_PREFIX + conversationId, messages);
}

export async function appendChatMessage(conversationId: string, message: StoredMessage): Promise<void> {
  const existing = await getChatMessages(conversationId);
  // Dedup by ID
  if (existing.some((m) => m.id === message.id)) return;
  existing.push(message);
  // Keep last 500 messages per conversation
  const trimmed = existing.length > 500 ? existing.slice(-500) : existing;
  await saveChatMessages(conversationId, trimmed);
}

export async function getChannels(): Promise<StoredChannel[]> {
  try {
    return (await get<StoredChannel[]>(CHANNELS_KEY)) ?? [];
  } catch {
    return [];
  }
}

export async function saveChannels(channels: StoredChannel[]): Promise<void> {
  await set(CHANNELS_KEY, channels);
}

export async function saveChatProfile(pubkey: string, profile: ChatProfile): Promise<void> {
  await set(PROFILE_PREFIX + pubkey, profile);
}

export async function getChatProfile(pubkey: string): Promise<ChatProfile | null> {
  try {
    return (await get<ChatProfile>(PROFILE_PREFIX + pubkey)) ?? null;
  } catch {
    return null;
  }
}

export async function getLastRead(conversationId: string): Promise<string | null> {
  try {
    return (await get<string>(LAST_READ_PREFIX + conversationId)) ?? null;
  } catch {
    return null;
  }
}

export async function setLastRead(conversationId: string, eventId: string): Promise<void> {
  await set(LAST_READ_PREFIX + conversationId, eventId);
}

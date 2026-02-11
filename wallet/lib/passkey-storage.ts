"use client";

import { get, set, del } from "idb-keyval";
import type { StoredWalletMeta } from "@/types/passkey";

const WALLET_META_KEY = "norn-wallet-meta";

/** Load wallet metadata from IndexedDB. Returns null if not found. */
export async function loadWalletMeta(): Promise<StoredWalletMeta | null> {
  try {
    const meta = await get<StoredWalletMeta>(WALLET_META_KEY);
    return meta ?? null;
  } catch {
    return null;
  }
}

/** Save wallet metadata to IndexedDB. */
export async function saveWalletMeta(meta: StoredWalletMeta): Promise<void> {
  await set(WALLET_META_KEY, meta);
}

/** Delete wallet metadata from IndexedDB. */
export async function deleteWalletMeta(): Promise<void> {
  await del(WALLET_META_KEY);
}

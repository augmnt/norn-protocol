"use client";

import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface Contact {
  address: string;
  label: string;
  nornName?: string;
  createdAt: number;
}

interface ContactsState {
  contacts: Contact[];
  recentRecipients: string[];
  addContact: (address: string, label: string, nornName?: string) => void;
  removeContact: (address: string) => void;
  getContactLabel: (address: string) => string | undefined;
  isContact: (address: string) => boolean;
  addRecentRecipient: (address: string) => void;
}

export const useContactsStore = create<ContactsState>()(
  persist(
    (set, get) => ({
      contacts: [],
      recentRecipients: [],
      addContact: (address, label, nornName?) =>
        set((state) => {
          const normalized = address.toLowerCase();
          const existing = state.contacts.findIndex(
            (c) => c.address.toLowerCase() === normalized
          );
          if (existing >= 0) {
            const updated = [...state.contacts];
            updated[existing] = { ...updated[existing], label, ...(nornName ? { nornName } : {}) };
            return { contacts: updated };
          }
          return {
            contacts: [...state.contacts, { address, label, ...(nornName ? { nornName } : {}), createdAt: Date.now() }],
          };
        }),
      removeContact: (address) =>
        set((state) => ({
          contacts: state.contacts.filter(
            (c) => c.address.toLowerCase() !== address.toLowerCase()
          ),
        })),
      getContactLabel: (address) => {
        const normalized = address.toLowerCase();
        return get().contacts.find(
          (c) => c.address.toLowerCase() === normalized
        )?.label;
      },
      isContact: (address) => {
        const normalized = address.toLowerCase();
        return get().contacts.some(
          (c) => c.address.toLowerCase() === normalized
        );
      },
      addRecentRecipient: (address) =>
        set((state) => {
          const normalized = address.toLowerCase();
          const filtered = state.recentRecipients.filter(
            (r) => r.toLowerCase() !== normalized
          );
          return {
            recentRecipients: [address, ...filtered].slice(0, 10),
          };
        }),
    }),
    { name: "norn-wallet-contacts" }
  )
);

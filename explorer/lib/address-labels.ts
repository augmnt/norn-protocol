"use client";

const KNOWN_LABELS: Record<string, string> = {
  "0x0000000000000000000000000000000000000000": "System / Burn",
  "0x557dede07828fc8ea66477a6056dbd446a640003": "Devnet Founder",
};

const STORAGE_KEY = "norn-address-labels";

function getUserLabels(): Record<string, string> {
  if (typeof window === "undefined") return {};
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
}

function saveUserLabels(labels: Record<string, string>) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(labels));
}

export function getAddressLabel(address: string): string | null {
  const lower = address.toLowerCase();
  const user = getUserLabels();
  return user[lower] ?? KNOWN_LABELS[lower] ?? null;
}

export function setAddressLabel(address: string, label: string) {
  const user = getUserLabels();
  user[address.toLowerCase()] = label;
  saveUserLabels(user);
}

export function removeAddressLabel(address: string) {
  const user = getUserLabels();
  delete user[address.toLowerCase()];
  saveUserLabels(user);
}

export function getAllLabels(): Record<string, string> {
  return { ...KNOWN_LABELS, ...getUserLabels() };
}

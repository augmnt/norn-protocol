"use client";

import { create } from "zustand";

const STORAGE_KEY = "norn-favorites";

interface FavoriteAddress {
  address: string;
  addedAt: number;
}

interface FavoritesState {
  favorites: FavoriteAddress[];
  addFavorite: (address: string) => void;
  removeFavorite: (address: string) => void;
  isFavorite: (address: string) => boolean;
}

function loadFavorites(): FavoriteAddress[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function saveFavorites(favorites: FavoriteAddress[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(favorites));
}

export const useFavoritesStore = create<FavoritesState>((set, get) => ({
  favorites: loadFavorites(),

  addFavorite: (address: string) => {
    const lower = address.toLowerCase();
    const current = get().favorites;
    if (current.some((f) => f.address.toLowerCase() === lower)) return;
    const updated = [{ address, addedAt: Date.now() }, ...current];
    saveFavorites(updated);
    set({ favorites: updated });
  },

  removeFavorite: (address: string) => {
    const lower = address.toLowerCase();
    const updated = get().favorites.filter(
      (f) => f.address.toLowerCase() !== lower
    );
    saveFavorites(updated);
    set({ favorites: updated });
  },

  isFavorite: (address: string) => {
    const lower = address.toLowerCase();
    return get().favorites.some((f) => f.address.toLowerCase() === lower);
  },
}));

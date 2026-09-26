import { create } from "zustand";
import type { ConversionResult } from "@/types/conversion";

interface FavoriteConversion {
  id: string;
  fromFormat: string;
  toFormat: string;
  label: string;
}

interface HistoryState {
  history: ConversionResult[];
  favorites: FavoriteConversion[];
  recentPairs: Array<{ from: string; to: string }>;

  setHistory: (history: ConversionResult[]) => void;
  addToHistory: (result: ConversionResult) => void;
  clearHistory: () => void;
  setFavorites: (favorites: FavoriteConversion[]) => void;
  addFavorite: (fav: FavoriteConversion) => void;
  removeFavorite: (id: string) => void;
  addRecentPair: (from: string, to: string) => void;
}

const MAX_RECENT_PAIRS = 10;

export const useHistoryStore = create<HistoryState>((set) => ({
  history: [],
  favorites: [],
  recentPairs: [],

  setHistory: (history) => set({ history }),
  addToHistory: (result) =>
    set((state) => ({ history: [result, ...state.history].slice(0, 500) })),
  clearHistory: () => set({ history: [] }),
  setFavorites: (favorites) => set({ favorites }),
  addFavorite: (fav) =>
    set((state) => ({ favorites: [...state.favorites, fav] })),
  removeFavorite: (id) =>
    set((state) => ({ favorites: state.favorites.filter((f) => f.id !== id) })),
  addRecentPair: (from, to) =>
    set((state) => {
      const filtered = state.recentPairs.filter(
        (p) => !(p.from === from && p.to === to)
      );
      return { recentPairs: [{ from, to }, ...filtered].slice(0, MAX_RECENT_PAIRS) };
    }),
}));

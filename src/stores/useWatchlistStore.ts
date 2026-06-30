import { create } from "zustand";
import {
  WatchlistItem,
  getWatchlist,
  addToWatchlist,
  removeFromWatchlist,
} from "../lib/tauri-bridge";

interface WatchlistState {
  items: WatchlistItem[];
  loading: boolean;
  error: string | null;
  fetchWatchlist: () => Promise<void>;
  addTicker: (ticker: string) => Promise<void>;
  removeTicker: (ticker: string) => Promise<void>;
}

export const useWatchlistStore = create<WatchlistState>((set) => ({
  items: [],
  loading: false,
  error: null,

  fetchWatchlist: async () => {
    set({ loading: true, error: null });
    try {
      const items = await getWatchlist();
      set({ items, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  addTicker: async (ticker: string) => {
    try {
      await addToWatchlist(ticker.toUpperCase());
      const items = await getWatchlist();
      set({ items });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  removeTicker: async (ticker: string) => {
    try {
      await removeFromWatchlist(ticker);
      set((state) => ({
        items: state.items.filter((i) => i.ticker !== ticker),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));

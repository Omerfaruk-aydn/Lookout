import { useState, useEffect } from "react";
import { useWatchlistStore } from "../stores/useWatchlistStore";

export function Watchlist() {
  const { items, loading, error, addTicker, removeTicker, fetchWatchlist } =
    useWatchlistStore();
  const [newTicker, setNewTicker] = useState("");

  useEffect(() => {
    fetchWatchlist();
  }, [fetchWatchlist]);

  const handleAdd = async () => {
    if (!newTicker.trim()) return;
    await addTicker(newTicker.trim());
    setNewTicker("");
  };

  return (
    <div className="flex flex-col gap-4">
      <div className="flex gap-2">
        <input
          type="text"
          value={newTicker}
          onChange={(e) => setNewTicker(e.target.value.toUpperCase())}
          onKeyDown={(e) => e.key === "Enter" && handleAdd()}
          placeholder="Add ticker..."
          className="flex-1 bg-bg-tertiary text-fg-primary border border-border rounded px-3 py-1.5 text-sm font-mono placeholder:text-fg-muted"
        />
        <button
          onClick={handleAdd}
          disabled={!newTicker.trim()}
          className="px-3 py-1.5 bg-accent-green text-bg-primary text-sm font-semibold rounded hover:bg-accent-green/80 disabled:opacity-50 transition-colors"
        >
          Add
        </button>
      </div>

      {error && (
        <div className="bg-accent-red/10 border border-accent-red/30 rounded-lg p-3">
          <p className="text-accent-red text-sm">{error}</p>
        </div>
      )}

      {loading && (
        <div className="flex justify-center py-4">
          <div className="w-6 h-6 border-2 border-accent-blue border-t-transparent rounded-full animate-spin" />
        </div>
      )}

      {!loading && items.length === 0 && (
        <p className="text-fg-muted text-sm text-center py-8">
          No tickers in watchlist. Add one above.
        </p>
      )}

      <div className="flex flex-col gap-2">
        {items.map((item) => (
          <div
            key={item.ticker}
            className="flex items-center justify-between bg-bg-secondary rounded-lg p-3 border border-border"
          >
            <div>
              <span className="text-fg-primary font-mono font-semibold text-sm">
                {item.ticker}
              </span>
              <p className="text-fg-muted text-xs mt-0.5">
                Added {new Date(item.added_at * 1000).toLocaleDateString()}
              </p>
            </div>
            <button
              onClick={() => removeTicker(item.ticker)}
              className="text-accent-red hover:text-accent-red/80 text-xs font-semibold transition-colors"
            >
              Remove
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}

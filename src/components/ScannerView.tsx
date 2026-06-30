import { useEffect, useState } from "react";
import {
  getMarketIndices,
  addIndexToWatchlist,
  runBatchScan,
  startAutoScanner,
  stopAutoScanner,
  isScannerRunning,
  MarketIndex,
} from "../lib/tauri-bridge";
import { useWatchlistStore } from "../stores/useWatchlistStore";

const regionMap: Record<string, { label: string; emoji: string }> = {
  US: { label: "United States", emoji: "🇺🇸" },
  UK: { label: "United Kingdom", emoji: "🇬🇧" },
  DE: { label: "Germany", emoji: "🇩🇪" },
  FR: { label: "France", emoji: "🇫🇷" },
  EU: { label: "Europe", emoji: "🇪🇺" },
  JP: { label: "Japan", emoji: "🇯🇵" },
  HK: { label: "Hong Kong", emoji: "🇭🇰" },
  CN: { label: "China", emoji: "🇨🇳" },
  KR: { label: "South Korea", emoji: "🇰🇷" },
  TW: { label: "Taiwan", emoji: "🇹🇼" },
  IN: { label: "India", emoji: "🇮🇳" },
  SG: { label: "Singapore", emoji: "🇸🇬" },
  MY: { label: "Malaysia", emoji: "🇲🇾" },
  TH: { label: "Thailand", emoji: "🇹🇭" },
  ID: { label: "Indonesia", emoji: "🇮🇩" },
  PH: { label: "Philippines", emoji: "🇵🇭" },
  VN: { label: "Vietnam", emoji: "🇻🇳" },
  CA: { label: "Canada", emoji: "🇨🇦" },
  BR: { label: "Brazil", emoji: "🇧🇷" },
  MX: { label: "Mexico", emoji: "🇲🇽" },
  CL: { label: "Chile", emoji: "🇨🇱" },
  PE: { label: "Peru", emoji: "🇵🇪" },
  CO: { label: "Colombia", emoji: "🇨🇴" },
  AR: { label: "Argentina", emoji: "🇦🇷" },
  SA: { label: "Saudi Arabia", emoji: "🇸🇦" },
  AE: { label: "UAE", emoji: "🇦🇪" },
  QA: { label: "Qatar", emoji: "🇶🇦" },
  KW: { label: "Kuwait", emoji: "🇰🇼" },
  BH: { label: "Bahrain", emoji: "🇧🇭" },
  OM: { label: "Oman", emoji: "🇴🇲" },
  IL: { label: "Israel", emoji: "🇮🇱" },
  TR: { label: "Turkey", emoji: "🇹🇷" },
  ZA: { label: "South Africa", emoji: "🇿🇦" },
  EG: { label: "Egypt", emoji: "🇪🇬" },
  NG: { label: "Nigeria", emoji: "🇳🇬" },
  KE: { label: "Kenya", emoji: "🇰🇪" },
  MA: { label: "Morocco", emoji: "🇲🇦" },
  RU: { label: "Russia", emoji: "🇷🇺" },
  PL: { label: "Poland", emoji: "🇵🇱" },
  CZ: { label: "Czech Republic", emoji: "🇨🇿" },
  GR: { label: "Greece", emoji: "🇬🇷" },
  ES: { label: "Spain", emoji: "🇪🇸" },
  CH: { label: "Switzerland", emoji: "🇨🇭" },
  AU: { label: "Australia", emoji: "🇦🇺" },
  GLOBAL: { label: "Global ETFs", emoji: "🌐" },
};

function getRegionKey(key: string): string {
  const upper = key.toUpperCase();
  for (const code of Object.keys(regionMap)) {
    if (upper.startsWith(code)) return code;
  }
  if (upper.includes("EURO")) return "EU";
  if (upper.includes("GLOBAL")) return "GLOBAL";
  if (upper.includes("FTSE")) return "UK";
  if (upper.includes("DAX")) return "DE";
  if (upper.includes("CAC")) return "FR";
  if (upper.includes("NIKKEI") || upper.includes("TOPIX")) return "JP";
  if (upper.includes("HANG") || upper.includes("HSI")) return "HK";
  if (upper.includes("SSE") || upper.includes("SZSE") || upper.includes("SHANGHAI") || upper.includes("SHENZHEN")) return "CN";
  if (upper.includes("KOSPI") || upper.includes("KOSDAQ")) return "KR";
  if (upper.includes("TWSE") || upper.includes("TPEX") || upper.includes("TAIEX")) return "TW";
  if (upper.includes("BSE") || upper.includes("NSE")) return "IN";
  if (upper.includes("SGX")) return "SG";
  if (upper.includes("BURSA") || upper.includes("KLCI")) return "MY";
  if (upper.includes("SET")) return "TH";
  if (upper.includes("IDX") || upper.includes("JAKARTA")) return "ID";
  if (upper.includes("PSE")) return "PH";
  if (upper.includes("HOSE") || upper.includes("VN")) return "VN";
  if (upper.includes("TSX")) return "CA";
  if (upper.includes("BOVESPA") || upper.includes("B3")) return "BR";
  if (upper.includes("IPC") || upper.includes("MEXICO")) return "MX";
  if (upper.includes("TADAWUL")) return "SA";
  if (upper.includes("BIST")) return "TR";
  if (upper.includes("ASX")) return "AU";
  if (upper.includes("MOEX") || upper.includes("MOSS")) return "RU";
  if (upper.includes("GPW") || upper.includes("WARSAW")) return "PL";
  if (upper.includes("SIX")) return "CH";
  if (upper.includes("BME") || upper.includes("MADRID")) return "ES";
  if (upper.includes("JSE")) return "ZA";
  if (upper.includes("TASE") || upper.includes("TEL_AVIV")) return "IL";
  return "GLOBAL";
}

export function ScannerView() {
  const { items, fetchWatchlist } = useWatchlistStore();
  const [indices, setIndices] = useState<MarketIndex[]>([]);
  const [selectedIndex, setSelectedIndex] = useState("");
  const [scanning, setScanning] = useState(false);
  const [scanResults, setScanResults] = useState<
    { ticker: string; success: boolean; error: string | null }[]
  >([]);
  const [scannerRunning, setScannerRunning] = useState(false);
  const [intervalMinutes, setIntervalMinutes] = useState(30);
  const [useWebSearch, setUseWebSearch] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getMarketIndices().then(setIndices);
    isScannerRunning().then(setScannerRunning);
  }, []);

  const handleAddIndex = async () => {
    if (!selectedIndex) return;
    setError(null);
    try {
      const count = await addIndexToWatchlist(selectedIndex);
      await fetchWatchlist();
      setError(`${count} tickers added to watchlist`);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleScanWatchlist = async () => {
    if (items.length === 0) return;
    setScanning(true);
    setScanResults([]);
    setError(null);
    try {
      const tickers = items.map((i) => i.ticker);
      const results = await runBatchScan(tickers, useWebSearch);
      setScanResults(results);
    } catch (e) {
      setError(String(e));
    } finally {
      setScanning(false);
    }
  };

  const handleStartAuto = async () => {
    setError(null);
    try {
      await startAutoScanner(intervalMinutes, useWebSearch);
      setScannerRunning(true);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleStopAuto = async () => {
    try {
      await stopAutoScanner();
      setScannerRunning(false);
    } catch (e) {
      setError(String(e));
    }
  };

  const grouped = indices.reduce<Record<string, MarketIndex[]>>((acc, idx) => {
    const regionKey = getRegionKey(idx.key);
    if (!acc[regionKey]) acc[regionKey] = [];
    acc[regionKey].push(idx);
    return acc;
  }, {});

  const regionOrder = Object.keys(grouped).sort((a, b) => {
    const order = ["US", "CA", "BR", "MX", "CL", "PE", "CO", "AR", "UK", "DE", "FR", "EU", "ES", "CH", "PL", "CZ", "GR", "JP", "CN", "KR", "TW", "IN", "SG", "MY", "TH", "ID", "PH", "VN", "SA", "AE", "QA", "KW", "BH", "OM", "IL", "TR", "ZA", "EG", "NG", "KE", "MA", "RU", "AU", "GLOBAL"];
    return order.indexOf(a) - order.indexOf(b);
  });

  return (
    <div className="flex flex-col gap-4">
      <h2 className="text-sm font-semibold text-accent-blue">
        Market Scanner & Auto-Analysis
      </h2>

      {error && (
        <div
          className={`p-3 rounded-lg text-sm ${
            error.includes("added")
              ? "bg-accent-green/10 border border-accent-green/30 text-accent-green"
              : "bg-accent-red/10 border border-accent-red/30 text-accent-red"
          }`}
        >
          {error}
        </div>
      )}

      <div className="bg-bg-secondary rounded-lg p-3 border border-border">
        <h3 className="text-accent-purple text-xs font-semibold mb-2 uppercase tracking-wide">
          Add Market Index
        </h3>
        <p className="text-fg-muted text-xs mb-2">
          {indices.length} indices available
        </p>
        <div className="flex gap-2">
          <select
            value={selectedIndex}
            onChange={(e) => setSelectedIndex(e.target.value)}
            className="flex-1 bg-bg-tertiary text-fg-primary border border-border rounded px-3 py-1.5 text-sm"
          >
            <option value="">Select index...</option>
            {regionOrder.map((regionKey) => {
              const region = regionMap[regionKey] || { label: regionKey, emoji: "📊" };
              return (
                <optgroup key={regionKey} label={`${region.emoji} ${region.label}`}>
                  {(grouped[regionKey] || []).map((idx) => (
                    <option key={idx.key} value={idx.key}>
                      {idx.label}
                    </option>
                  ))}
                </optgroup>
              );
            })}
          </select>
          <button
            onClick={handleAddIndex}
            disabled={!selectedIndex}
            className="px-3 py-1.5 bg-accent-blue text-bg-primary text-sm font-semibold rounded hover:bg-accent-blue/80 disabled:opacity-50 transition-colors"
          >
            Add
          </button>
        </div>
      </div>

      <div className="bg-bg-secondary rounded-lg p-3 border border-border">
        <h3 className="text-accent-purple text-xs font-semibold mb-2 uppercase tracking-wide">
          Batch Scan
        </h3>
        <p className="text-fg-secondary text-xs mb-3">
          Scan all {items.length} tickers in your watchlist.
        </p>
        <label className="flex items-center gap-2 mb-3 text-sm text-fg-primary">
          <input
            type="checkbox"
            checked={useWebSearch}
            onChange={(e) => setUseWebSearch(e.target.checked)}
            className="rounded border-border bg-bg-tertiary text-accent-blue"
          />
          Use AI web search
        </label>
        <button
          onClick={handleScanWatchlist}
          disabled={scanning || items.length === 0}
          className="w-full px-3 py-2 bg-accent-green text-bg-primary text-sm font-semibold rounded hover:bg-accent-green/80 disabled:opacity-50 transition-colors"
        >
          {scanning ? `Scanning 0/${items.length}...` : "Scan Watchlist Now"}
        </button>
      </div>

      <div className="bg-bg-secondary rounded-lg p-3 border border-border">
        <h3 className="text-accent-purple text-xs font-semibold mb-2 uppercase tracking-wide">
          Auto Scanner
        </h3>
        <div className="flex items-center gap-2 mb-3">
          <span className="text-fg-secondary text-xs">Interval (min):</span>
          <input
            type="number"
            min={5}
            max={1440}
            value={intervalMinutes}
            onChange={(e) => setIntervalMinutes(parseInt(e.target.value) || 30)}
            className="w-20 bg-bg-tertiary text-fg-primary border border-border rounded px-2 py-1 text-sm font-mono"
          />
        </div>
        {scannerRunning ? (
          <button
            onClick={handleStopAuto}
            className="w-full px-3 py-2 bg-accent-red text-bg-primary text-sm font-semibold rounded hover:bg-accent-red/80 transition-colors"
          >
            Stop Auto Scanner
          </button>
        ) : (
          <button
            onClick={handleStartAuto}
            disabled={items.length === 0}
            className="w-full px-3 py-2 bg-accent-yellow text-bg-primary text-sm font-semibold rounded hover:bg-accent-yellow/80 disabled:opacity-50 transition-colors"
          >
            Start Auto Scanner
          </button>
        )}
      </div>

      {scanResults.length > 0 && (
        <div className="flex flex-col gap-2">
          <h3 className="text-fg-secondary text-xs font-semibold">Scan Results</h3>
          {scanResults.map((r) => (
            <div
              key={r.ticker}
              className={`flex items-center justify-between rounded-lg p-2 border ${
                r.success
                  ? "bg-accent-green/10 border-accent-green/30"
                  : "bg-accent-red/10 border-accent-red/30"
              }`}
            >
              <span className="text-fg-primary text-sm font-mono">{r.ticker}</span>
              <span
                className={`text-xs font-semibold ${
                  r.success ? "text-accent-green" : "text-accent-red"
                }`}
              >
                {r.success ? "OK" : r.error || "Failed"}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

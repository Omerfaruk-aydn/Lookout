import { useState, useCallback } from "react";
import { useReportStore } from "../stores/useReportStore";
import { useSettingsStore } from "../stores/useSettingsStore";
import { findWebull, captureScreen, CaptureRect } from "../lib/tauri-bridge";
import { ReportView } from "./ReportView";
import { Watchlist } from "./Watchlist";
import { ChartView } from "./ChartView";
import { HistoryView } from "./HistoryView";

import { ScannerView } from "./ScannerView";

type Tab = "analysis" | "watchlist" | "chart" | "history" | "scanner";

export function OverlayPanel() {
  const [activeTab, setActiveTab] = useState<Tab>("analysis");
  const [tickerInput, setTickerInput] = useState("");
  const [useWebSearch, setUseWebSearch] = useState(true);
  const { currentReport, analyzing, error, analyzeTicker, clearCurrent } =
    useReportStore();
  const { regionConfig } = useSettingsStore();

  const handleAnalyze = useCallback(async () => {
    if (!tickerInput.trim()) return;
    clearCurrent();

    let imageBase64: string | undefined;
    if (regionConfig) {
      try {
        const hwnd = await findWebull();
        const rect: CaptureRect = {
          x: Math.round(
            regionConfig.chart_area.x_pct * 1920
          ),
          y: Math.round(
            regionConfig.chart_area.y_pct * 1080
          ),
          width: Math.round(
            regionConfig.chart_area.width_pct * 1920
          ),
          height: Math.round(
            regionConfig.chart_area.height_pct * 1080
          ),
        };
        imageBase64 = await captureScreen(hwnd, rect);
      } catch {
        imageBase64 = undefined;
      }
    }

    await analyzeTicker(tickerInput.toUpperCase().trim(), imageBase64, useWebSearch);
  }, [tickerInput, regionConfig, analyzeTicker, clearCurrent, useWebSearch]);

  const tabs: { key: Tab; label: string }[] = [
    { key: "analysis", label: "Analysis" },
    { key: "watchlist", label: "Watchlist" },
    { key: "scanner", label: "Scanner" },
    { key: "chart", label: "Chart" },
    { key: "history", label: "History" },
  ];

  return (
    <div className="flex flex-col h-screen bg-bg-primary">
      <header className="flex items-center justify-between px-4 py-3 bg-bg-secondary border-b border-border">
        <h1 className="text-lg font-bold text-accent-blue">Lookout</h1>
        <span className="text-xs text-fg-muted font-mono">v0.1.0</span>
      </header>

      <div className="flex items-center gap-2 px-4 py-2 bg-bg-secondary border-b border-border">
        <input
          type="text"
          value={tickerInput}
          onChange={(e) => setTickerInput(e.target.value.toUpperCase())}
          onKeyDown={(e) => e.key === "Enter" && handleAnalyze()}
          placeholder="Enter ticker (e.g. AAPL)"
          className="flex-1 bg-bg-tertiary text-fg-primary border border-border rounded px-3 py-1.5 text-sm font-mono placeholder:text-fg-muted"
        />
        <button
          onClick={handleAnalyze}
          disabled={analyzing || !tickerInput.trim()}
          className="px-3 py-1.5 bg-accent-blue text-bg-primary text-sm font-semibold rounded hover:bg-accent-blue/80 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {analyzing ? "Analyzing..." : "Analyze"}
        </button>
      </div>

      <div className="flex items-center gap-3 px-4 py-1 bg-bg-secondary border-b border-border">
        <label className="flex items-center gap-2 text-xs text-fg-primary">
          <input
            type="checkbox"
            checked={useWebSearch}
            onChange={(e) => setUseWebSearch(e.target.checked)}
            className="rounded border-border bg-bg-tertiary text-accent-blue"
          />
          AI Web Search
        </label>
      </div>

      <nav className="flex border-b border-border bg-bg-secondary">
        {tabs.map((tab) => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key)}
            className={`flex-1 px-3 py-2 text-xs font-medium transition-colors ${
              activeTab === tab.key
                ? "text-accent-blue border-b-2 border-accent-blue"
                : "text-fg-muted hover:text-fg-secondary"
            }`}
          >
            {tab.label}
          </button>
        ))}
      </nav>

      <main className="flex-1 overflow-y-auto p-4">
        {error && (
          <div className="bg-accent-red/10 border border-accent-red/30 rounded-lg p-3 mb-4">
            <p className="text-accent-red text-sm">{error}</p>
          </div>
        )}

        {analyzing && (
          <div className="flex flex-col items-center justify-center py-12 gap-3">
            <div className="w-8 h-8 border-2 border-accent-purple border-t-transparent rounded-full animate-spin" />
            <p className="text-fg-secondary text-sm">
              Analyzing {tickerInput}...
            </p>
            <p className="text-fg-muted text-xs">
              This may take 3-8 seconds
            </p>
          </div>
        )}

        {activeTab === "analysis" && !analyzing && currentReport && (
          <ReportView report={currentReport} />
        )}

        {activeTab === "analysis" && !analyzing && !currentReport && !error && (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <p className="text-fg-muted text-sm">
              Enter a ticker symbol and click Analyze to get started.
            </p>
          </div>
        )}

        {activeTab === "watchlist" && <Watchlist />}
        {activeTab === "scanner" && <ScannerView />}
        {activeTab === "chart" && <ChartView />}
        {activeTab === "history" && <HistoryView />}
      </main>
    </div>
  );
}

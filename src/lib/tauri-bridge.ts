import { invoke } from "@tauri-apps/api/core";

export interface RegionRect {
  x_pct: number;
  y_pct: number;
  width_pct: number;
  height_pct: number;
}

export interface RegionConfig {
  chart_area: RegionRect;
  ticker_area: RegionRect;
  price_area: RegionRect;
}

export interface WatchlistItem {
  ticker: string;
  added_at: number;
  auto_scan_enabled: boolean;
}

export interface ReportRecord {
  id: string;
  ticker: string;
  created_at: number;
  vision_result_json: string | null;
  technical_snapshot_json: string | null;
  sentiment_result_json: string | null;
  synthesis_report_json: string;
  confidence_level: string;
}

export interface SynthesisReport {
  ticker: string;
  summary: string;
  technical_status: string;
  news_impact: string;
  conflicting_signals: string | null;
  risk_notes: string;
  confidence_level: string;
}

export interface VisionResult {
  ticker_visible: string | null;
  trend_direction: string | null;
  visible_patterns: string[];
  support_resistance_estimate: {
    support: number[];
    resistance: number[];
  };
  volume_observation: string | null;
  indicators_visible: Array<{ name: string; value_estimate: string }>;
  confidence: number;
  notes: string | null;
}

export interface TechnicalSnapshot {
  ticker: string;
  sma_20: number | null;
  sma_50: number | null;
  sma_200: number | null;
  ema_12: number | null;
  ema_26: number | null;
  rsi_14: number | null;
  macd: {
    macd_line: number;
    signal_line: number;
    histogram: number;
  } | null;
  bollinger: {
    upper: number;
    middle: number;
    lower: number;
  } | null;
  atr_14: number | null;
  volume_anomaly: {
    current_volume: number;
    average_volume: number;
    z_score: number;
  } | null;
  support_levels: number[];
  resistance_levels: number[];
  breakout_signal: {
    direction: string;
    level: number;
    volume_confirmation: boolean;
  } | null;
  computed_at: number;
}

export interface SentimentResult {
  ticker: string;
  overall_sentiment: string;
  weighted_score: number;
  item_count: number;
}

export interface FullAnalysisResult {
  ticker: string;
  vision_result: VisionResult | null;
  technical_snapshot: TechnicalSnapshot | null;
  sentiment_result: SentimentResult | null;
  synthesis_report: SynthesisReport;
  report_id: string;
  created_at: number;
}

export interface Setting {
  key: string;
  value: string;
}

export interface CaptureRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export async function findWebull(): Promise<number> {
  return invoke<number>("find_webull");
}

export async function captureScreen(
  hwnd: number,
  region: CaptureRect
): Promise<string> {
  return invoke<string>("capture_screen", { hwnd, region });
}

export async function saveRegionConfig(
  chartArea: RegionRect,
  tickerArea: RegionRect,
  priceArea: RegionRect
): Promise<void> {
  return invoke("save_region_config", {
    chartArea,
    tickerArea,
    priceArea,
  });
}

export async function getRegionConfig(): Promise<RegionConfig | null> {
  return invoke<RegionConfig | null>("get_region_config");
}

export async function runAnalysis(
  ticker: string,
  imageBase64?: string
): Promise<FullAnalysisResult> {
  return invoke<FullAnalysisResult>("run_analysis", {
    ticker,
    imageBase64: imageBase64 ?? null,
  });
}

export async function addToWatchlist(ticker: string): Promise<void> {
  return invoke("add_to_watchlist", { ticker });
}

export async function removeFromWatchlist(ticker: string): Promise<void> {
  return invoke("remove_from_watchlist", { ticker });
}

export async function getWatchlist(): Promise<WatchlistItem[]> {
  return invoke<WatchlistItem[]>("get_watchlist");
}

export async function getReports(
  ticker?: string,
  limit?: number
): Promise<ReportRecord[]> {
  return invoke<ReportRecord[]>("get_reports", {
    ticker: ticker ?? null,
    limit: limit ?? 50,
  });
}

export async function getReportById(
  id: string
): Promise<ReportRecord | null> {
  return invoke<ReportRecord | null>("get_report_by_id", { id });
}

export async function getSettings(): Promise<Setting[]> {
  return invoke<Setting[]>("get_settings");
}

export async function saveSetting(
  key: string,
  value: string
): Promise<void> {
  return invoke("save_setting", { key, value });
}

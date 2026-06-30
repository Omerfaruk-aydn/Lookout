import { create } from "zustand";
import {
  FullAnalysisResult,
  ReportRecord,
  runAnalysis,
  getReports,
} from "../lib/tauri-bridge";

interface ReportState {
  currentReport: FullAnalysisResult | null;
  history: ReportRecord[];
  analyzing: boolean;
  loading: boolean;
  error: string | null;
  analyzeTicker: (ticker: string, imageBase64?: string) => Promise<void>;
  fetchHistory: (ticker?: string) => Promise<void>;
  clearCurrent: () => void;
}

export const useReportStore = create<ReportState>((set) => ({
  currentReport: null,
  history: [],
  analyzing: false,
  loading: false,
  error: null,

  analyzeTicker: async (ticker: string, imageBase64?: string) => {
    set({ analyzing: true, error: null, currentReport: null });
    try {
      const result = await runAnalysis(ticker, imageBase64);
      set({ currentReport: result, analyzing: false });
    } catch (e) {
      set({ error: String(e), analyzing: false });
    }
  },

  fetchHistory: async (ticker?: string) => {
    set({ loading: true, error: null });
    try {
      const history = await getReports(ticker);
      set({ history, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  clearCurrent: () => set({ currentReport: null }),
}));

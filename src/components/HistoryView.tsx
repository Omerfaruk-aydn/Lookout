import { useEffect, useState } from "react";
import { useReportStore } from "../stores/useReportStore";
import { ReportRecord, SynthesisReport } from "../lib/tauri-bridge";

export function HistoryView() {
  const { history, loading, error, fetchHistory } = useReportStore();
  const [filterTicker, setFilterTicker] = useState("");
  const [selectedReport, setSelectedReport] = useState<SynthesisReport | null>(
    null
  );

  useEffect(() => {
    fetchHistory(filterTicker || undefined);
  }, [fetchHistory, filterTicker]);

  const parseReport = (record: ReportRecord): SynthesisReport | null => {
    try {
      return JSON.parse(record.synthesis_report_json) as SynthesisReport;
    } catch {
      return null;
    }
  };

  const confidenceColor = (level: string): string => {
    switch (level) {
      case "high":
        return "text-accent-green";
      case "medium":
        return "text-accent-yellow";
      case "low":
        return "text-accent-red";
      default:
        return "text-fg-muted";
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <div className="flex gap-2">
        <input
          type="text"
          value={filterTicker}
          onChange={(e) => setFilterTicker(e.target.value.toUpperCase())}
          placeholder="Filter by ticker..."
          className="flex-1 bg-bg-tertiary text-fg-primary border border-border rounded px-3 py-1.5 text-sm font-mono placeholder:text-fg-muted"
        />
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

      {selectedReport && (
        <div className="bg-bg-secondary rounded-lg p-3 border border-accent-blue/30">
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-accent-blue font-mono font-semibold text-sm">
              {selectedReport.ticker}
            </h3>
            <button
              onClick={() => setSelectedReport(null)}
              className="text-fg-muted hover:text-fg-primary text-xs"
            >
              Close
            </button>
          </div>
          <p className="text-fg-primary text-sm mb-2">
            {selectedReport.summary}
          </p>
          <p className="text-fg-secondary text-xs">
            {selectedReport.technical_status}
          </p>
        </div>
      )}

      <div className="flex flex-col gap-2">
        {!loading && history.length === 0 && (
          <p className="text-fg-muted text-sm text-center py-8">
            No reports found.
          </p>
        )}

        {history.map((record) => {
          const report = parseReport(record);
          return (
            <button
              key={record.id}
              onClick={() => report && setSelectedReport(report)}
              className="flex items-center justify-between bg-bg-secondary rounded-lg p-3 border border-border hover:border-accent-blue/30 transition-colors text-left"
            >
              <div>
                <span className="text-fg-primary font-mono font-semibold text-sm">
                  {record.ticker}
                </span>
                <p className="text-fg-muted text-xs mt-0.5">
                  {new Date(record.created_at * 1000).toLocaleString()}
                </p>
              </div>
              <span
                className={`text-xs font-semibold ${confidenceColor(record.confidence_level)}`}
              >
                {record.confidence_level.toUpperCase()}
              </span>
            </button>
          );
        })}
      </div>
    </div>
  );
}

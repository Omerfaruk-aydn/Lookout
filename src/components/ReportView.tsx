import { FullAnalysisResult } from "../lib/tauri-bridge";

interface ReportViewProps {
  report: FullAnalysisResult;
}

function ConfidenceBadge({ level }: { level: string }) {
  const colorMap: Record<string, string> = {
    high: "bg-accent-green/20 text-accent-green border-accent-green/30",
    medium: "bg-accent-yellow/20 text-accent-yellow border-accent-yellow/30",
    low: "bg-accent-red/20 text-accent-red border-accent-red/30",
  };

  const className = colorMap[level] ?? colorMap["low"];

  return (
    <span
      className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-semibold border ${className}`}
    >
      {level.toUpperCase()}
    </span>
  );
}

function SentimentBadge({ sentiment }: { sentiment: string }) {
  const colorMap: Record<string, string> = {
    positive: "text-accent-green",
    negative: "text-accent-red",
    neutral: "text-accent-yellow",
    mixed: "text-accent-purple",
  };

  const className = colorMap[sentiment] ?? "text-fg-secondary";

  return (
    <span className={`text-xs font-semibold ${className}`}>
      {sentiment.toUpperCase()}
    </span>
  );
}

function ReportSection({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="bg-bg-secondary rounded-lg p-3 border border-border">
      <h3 className="text-accent-purple text-xs font-semibold mb-2 uppercase tracking-wide">
        {title}
      </h3>
      {children}
    </div>
  );
}

export function ReportView({ report }: ReportViewProps) {
  const synthesis = report.synthesis_report;

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-bold text-fg-primary font-mono">
          {report.ticker}
        </h2>
        <ConfidenceBadge level={synthesis.confidence_level} />
      </div>

      {synthesis.conflicting_signals && (
        <div className="bg-accent-yellow/10 border border-accent-yellow/30 rounded-lg p-3">
          <p className="text-accent-yellow text-xs font-semibold mb-1">
            Conflicting Signals Detected
          </p>
          <p className="text-fg-secondary text-sm">
            {synthesis.conflicting_signals}
          </p>
        </div>
      )}

      <ReportSection title="Summary">
        <p className="text-fg-primary text-sm leading-relaxed">
          {synthesis.summary}
        </p>
      </ReportSection>

      <ReportSection title="Technical Status">
        <p className="text-fg-primary text-sm leading-relaxed">
          {synthesis.technical_status}
        </p>
      </ReportSection>

      <ReportSection title="News Impact">
        <div className="flex items-center gap-2 mb-1">
          {report.sentiment_result && (
            <SentimentBadge
              sentiment={report.sentiment_result.overall_sentiment}
            />
          )}
        </div>
        <p className="text-fg-primary text-sm leading-relaxed">
          {synthesis.news_impact}
        </p>
      </ReportSection>

      <ReportSection title="Risk Notes">
        <p className="text-fg-primary text-sm leading-relaxed">
          {synthesis.risk_notes}
        </p>
      </ReportSection>

      {report.technical_snapshot && (
        <ReportSection title="Technical Indicators">
          <div className="grid grid-cols-2 gap-2 text-xs font-mono">
            {report.technical_snapshot.rsi_14 !== null && (
              <div className="flex justify-between">
                <span className="text-fg-muted">RSI(14)</span>
                <span
                  className={
                    report.technical_snapshot.rsi_14 > 70
                      ? "text-accent-red"
                      : report.technical_snapshot.rsi_14 < 30
                        ? "text-accent-green"
                        : "text-fg-primary"
                  }
                >
                  {report.technical_snapshot.rsi_14.toFixed(1)}
                </span>
              </div>
            )}
            {report.technical_snapshot.sma_20 !== null && (
              <div className="flex justify-between">
                <span className="text-fg-muted">SMA20</span>
                <span className="text-fg-primary">
                  {report.technical_snapshot.sma_20.toFixed(2)}
                </span>
              </div>
            )}
            {report.technical_snapshot.sma_50 !== null && (
              <div className="flex justify-between">
                <span className="text-fg-muted">SMA50</span>
                <span className="text-fg-primary">
                  {report.technical_snapshot.sma_50.toFixed(2)}
                </span>
              </div>
            )}
            {report.technical_snapshot.macd !== null && (
              <div className="flex justify-between">
                <span className="text-fg-muted">MACD</span>
                <span
                  className={
                    report.technical_snapshot.macd.histogram > 0
                      ? "text-accent-green"
                      : "text-accent-red"
                  }
                >
                  {report.technical_snapshot.macd.histogram.toFixed(4)}
                </span>
              </div>
            )}
            {report.technical_snapshot.atr_14 !== null && (
              <div className="flex justify-between">
                <span className="text-fg-muted">ATR(14)</span>
                <span className="text-fg-primary">
                  {report.technical_snapshot.atr_14.toFixed(2)}
                </span>
              </div>
            )}
          </div>
        </ReportSection>
      )}

      <p className="text-fg-muted text-xs text-center mt-2 italic">
        This report is for informational purposes only and does not constitute
        investment advice.
      </p>
    </div>
  );
}

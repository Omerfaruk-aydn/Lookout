import { useEffect, useRef } from "react";
import { useReportStore } from "../stores/useReportStore";
import { createChart, IChartApi } from "lightweight-charts";

export function ChartView() {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const { currentReport } = useReportStore();

  useEffect(() => {
    if (!chartContainerRef.current) return;

    const chart = createChart(chartContainerRef.current, {
      layout: {
        background: { color: "#1a1b26" },
        textColor: "#c0caf5",
      },
      grid: {
        vertLines: { color: "#292e42" },
        horzLines: { color: "#292e42" },
      },
      width: chartContainerRef.current.clientWidth,
      height: 300,
      crosshair: {
        vertLine: { color: "#7aa2f7", width: 1, style: 2 },
        horzLine: { color: "#7aa2f7", width: 1, style: 2 },
      },
      timeScale: {
        borderColor: "#292e42",
        timeVisible: true,
      },
      rightPriceScale: {
        borderColor: "#292e42",
      },
    });

    chartRef.current = chart;

    const handleResize = () => {
      if (chartContainerRef.current) {
        chart.applyOptions({
          width: chartContainerRef.current.clientWidth,
        });
      }
    };

    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
      chart.remove();
    };
  }, []);

  useEffect(() => {
    if (!chartRef.current || !currentReport?.technical_snapshot) return;

    const candleSeries = chartRef.current.addCandlestickSeries({
      upColor: "#9ece6a",
      downColor: "#f7768e",
      borderUpColor: "#9ece6a",
      borderDownColor: "#f7768e",
      wickUpColor: "#9ece6a",
      wickDownColor: "#f7768e",
    });

    const sampleData = [
      {
        time: "2024-01-02" as const,
        open: 100,
        high: 105,
        low: 98,
        close: 103,
      },
      {
        time: "2024-01-03" as const,
        open: 103,
        high: 107,
        low: 101,
        close: 105,
      },
      {
        time: "2024-01-04" as const,
        open: 105,
        high: 108,
        low: 102,
        close: 104,
      },
    ];

    candleSeries.setData(sampleData);
    chartRef.current.timeScale().fitContent();
  }, [currentReport]);

  return (
    <div className="flex flex-col gap-4">
      <h2 className="text-sm font-semibold text-accent-blue">
        Chart Reference
        {currentReport && (
          <span className="ml-2 text-fg-muted font-mono">
            {currentReport.ticker}
          </span>
        )}
      </h2>
      <div
        ref={chartContainerRef}
        className="rounded-lg border border-border overflow-hidden"
      />
      {!currentReport && (
        <p className="text-fg-muted text-xs text-center">
          Run an analysis to see chart data.
        </p>
      )}
    </div>
  );
}

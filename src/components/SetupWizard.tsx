import { useState } from "react";
import { useSettingsStore } from "../stores/useSettingsStore";
import { findWebull, saveRegionConfig, RegionRect } from "../lib/tauri-bridge";

type SetupStep = "find_webull" | "select_regions" | "complete";

export function SetupWizard() {
  const [step, setStep] = useState<SetupStep>("find_webull");
  const [, setHwnd] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [regions, setRegions] = useState<{
    chart: RegionRect;
    ticker: RegionRect;
    price: RegionRect;
  }>({
    chart: { x_pct: 0.05, y_pct: 0.1, width_pct: 0.7, height_pct: 0.6 },
    ticker: { x_pct: 0.05, y_pct: 0.02, width_pct: 0.3, height_pct: 0.06 },
    price: { x_pct: 0.7, y_pct: 0.02, width_pct: 0.25, height_pct: 0.06 },
  });

  const { setRegionConfig } = useSettingsStore();

  const handleFindWebull = async () => {
    setError(null);
    try {
      const foundHwnd = await findWebull();
      setHwnd(foundHwnd);
      setStep("select_regions");
    } catch (e) {
      setError(String(e));
    }
  };

  const handleSaveRegions = async () => {
    setError(null);
    try {
      await saveRegionConfig(regions.chart, regions.ticker, regions.price);
      setRegionConfig({
        chart_area: regions.chart,
        ticker_area: regions.ticker,
        price_area: regions.price,
      });
      setStep("complete");
    } catch (e) {
      setError(String(e));
    }
  };

  const updateRegion = (
    key: "chart" | "ticker" | "price",
    field: keyof RegionRect,
    value: number
  ) => {
    setRegions((prev) => ({
      ...prev,
      [key]: { ...prev[key], [field]: value },
    }));
  };

  return (
    <div className="flex flex-col h-screen bg-bg-primary p-6 overflow-y-auto">
      <h1 className="text-2xl font-bold text-accent-blue mb-2">
        Lookout Setup
      </h1>
      <p className="text-fg-secondary text-sm mb-6">
        Configure screen capture regions for Webull Desktop.
      </p>

      {error && (
        <div className="bg-accent-red/10 border border-accent-red/30 rounded-lg p-3 mb-4">
          <p className="text-accent-red text-sm">{error}</p>
        </div>
      )}

      {step === "find_webull" && (
        <div className="flex flex-col gap-4">
          <p className="text-fg-primary text-sm">
            Make sure Webull Desktop is open and running, then click below to
            detect it.
          </p>
          <button
            onClick={handleFindWebull}
            className="px-4 py-2 bg-accent-blue text-bg-primary font-semibold rounded-lg hover:bg-accent-blue/80 transition-colors"
          >
            Find Webull Window
          </button>
        </div>
      )}

      {step === "select_regions" && (
        <div className="flex flex-col gap-6">
          <p className="text-fg-primary text-sm">
            Adjust the capture regions below. Values are percentages (0.0-1.0)
            of the Webull window size.
          </p>

          {(["chart", "ticker", "price"] as const).map((regionKey) => (
            <div
              key={regionKey}
              className="bg-bg-secondary rounded-lg p-4 border border-border"
            >
              <h3 className="text-accent-purple font-semibold mb-3 capitalize">
                {regionKey} Area
              </h3>
              <div className="grid grid-cols-2 gap-3">
                {(["x_pct", "y_pct", "width_pct", "height_pct"] as const).map(
                  (field) => (
                    <label key={field} className="flex flex-col gap-1">
                      <span className="text-fg-muted text-xs">
                        {field.replace("_pct", "").replace("_", " ")}
                      </span>
                      <input
                        type="number"
                        step="0.01"
                        min="0"
                        max="1"
                        value={regions[regionKey][field]}
                        onChange={(e) =>
                          updateRegion(
                            regionKey,
                            field,
                            parseFloat(e.target.value) || 0
                          )
                        }
                        className="bg-bg-tertiary text-fg-primary border border-border rounded px-2 py-1 text-sm font-mono"
                      />
                    </label>
                  )
                )}
              </div>
            </div>
          ))}

          <button
            onClick={handleSaveRegions}
            className="px-4 py-2 bg-accent-green text-bg-primary font-semibold rounded-lg hover:bg-accent-green/80 transition-colors"
          >
            Save Regions & Continue
          </button>
        </div>
      )}

      {step === "complete" && (
        <div className="flex flex-col gap-4">
          <div className="bg-accent-green/10 border border-accent-green/30 rounded-lg p-4">
            <p className="text-accent-green font-semibold">Setup Complete!</p>
            <p className="text-fg-secondary text-sm mt-1">
              Lookout is ready to analyze charts. The application will reload
              automatically.
            </p>
          </div>
        </div>
      )}
    </div>
  );
}

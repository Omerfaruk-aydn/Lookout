import { useEffect } from "react";
import { useSettingsStore } from "./stores/useSettingsStore";
import { useWatchlistStore } from "./stores/useWatchlistStore";
import { SetupWizard } from "./components/SetupWizard";
import { OverlayPanel } from "./components/OverlayPanel";

function App() {
  const { setupComplete, loading, loadSettings } = useSettingsStore();
  const { fetchWatchlist } = useWatchlistStore();

  useEffect(() => {
    loadSettings();
    fetchWatchlist();
  }, [loadSettings, fetchWatchlist]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen bg-bg-primary">
        <div className="flex flex-col items-center gap-4">
          <div className="w-8 h-8 border-2 border-accent-blue border-t-transparent rounded-full animate-spin" />
          <p className="text-fg-secondary text-sm">Loading Lookout...</p>
        </div>
      </div>
    );
  }

  if (!setupComplete) {
    return <SetupWizard />;
  }

  return <OverlayPanel />;
}

export default App;

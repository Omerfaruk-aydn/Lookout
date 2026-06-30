import { create } from "zustand";
import { RegionConfig, getRegionConfig, getSettings, Setting } from "../lib/tauri-bridge";

interface SettingsState {
  regionConfig: RegionConfig | null;
  settings: Setting[];
  setupComplete: boolean;
  loading: boolean;
  error: string | null;
  loadSettings: () => Promise<void>;
  setRegionConfig: (config: RegionConfig) => void;
  setSetupComplete: (complete: boolean) => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  regionConfig: null,
  settings: [],
  setupComplete: false,
  loading: false,
  error: null,

  loadSettings: async () => {
    set({ loading: true, error: null });
    try {
      const [regionConfig, settings] = await Promise.all([
        getRegionConfig(),
        getSettings(),
      ]);
      set({
        regionConfig,
        settings,
        setupComplete: regionConfig !== null,
        loading: false,
      });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  setRegionConfig: (config: RegionConfig) =>
    set({ regionConfig: config, setupComplete: true }),

  setSetupComplete: (complete: boolean) => set({ setupComplete: complete }),
}));

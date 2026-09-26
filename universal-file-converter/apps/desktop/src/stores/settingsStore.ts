import { create } from "zustand";
import type { AppSettings } from "@/types/settings";
import { DEFAULT_SETTINGS } from "@/types/settings";

interface SettingsState {
  settings: AppSettings;
  isDirty: boolean;
  setSettings: (settings: AppSettings) => void;
  updateSettings: (path: string, value: unknown) => void;
  markClean: () => void;
  resetToDefaults: () => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: { ...DEFAULT_SETTINGS },
  isDirty: false,

  setSettings: (settings) => set({ settings, isDirty: false }),
  updateSettings: (path, value) =>
    set((state) => {
      const keys = path.split(".");
      const newSettings = JSON.parse(JSON.stringify(state.settings));
      let current: Record<string, unknown> = newSettings;
      for (let i = 0; i < keys.length - 1; i++) {
        current = current[keys[i]] as Record<string, unknown>;
      }
      current[keys[keys.length - 1]] = value;
      return { settings: newSettings, isDirty: true };
    }),
  markClean: () => set({ isDirty: false }),
  resetToDefaults: () => set({ settings: { ...DEFAULT_SETTINGS }, isDirty: true }),
}));

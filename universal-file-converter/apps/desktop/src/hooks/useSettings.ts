import { useEffect, useCallback } from "react";
import { useSettingsStore } from "@/stores/settingsStore";
import { getSettings, saveSettings } from "@/services/ipc";

export function useSettings() {
  const { settings, isDirty, setSettings, updateSettings, markClean, resetToDefaults } =
    useSettingsStore();

  useEffect(() => {
    getSettings()
      .then(setSettings)
      .catch(() => {});
  }, [setSettings]);

  const save = useCallback(async () => {
    try {
      await saveSettings(settings);
      markClean();
    } catch {
      throw new Error("Failed to save settings");
    }
  }, [settings, markClean]);

  return { settings, isDirty, updateSettings, save, resetToDefaults };
}

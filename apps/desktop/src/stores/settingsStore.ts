import { create } from "zustand";

type Theme = "system" | "light" | "dark";

interface SettingsState {
  theme: Theme;
  setTheme: (t: Theme) => void;
}

export const useSettings = create<SettingsState>((set) => ({
  theme: "system",
  setTheme: (theme) => set({ theme }),
}));

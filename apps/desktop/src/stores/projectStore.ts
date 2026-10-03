import { create } from "zustand";
import type { ImportedPage } from "../types/bindings";

interface ProjectState {
  pages: ImportedPage[];
  activeId: string | null;
  addPages: (p: ImportedPage[]) => void;
  removePage: (id: string) => void;
  setActive: (id: string) => void;
  clear: () => void;
}

export const useProject = create<ProjectState>((set) => ({
  pages: [],
  activeId: null,
  addPages: (incoming) =>
    set((s) => {
      const existing = new Set(s.pages.map((p) => p.source_hash));
      const deduped = incoming.filter((p) => !existing.has(p.source_hash));
      const pages = [...s.pages, ...deduped];
      return { pages, activeId: s.activeId ?? pages[0]?.id ?? null };
    }),
  removePage: (id) =>
    set((s) => {
      const pages = s.pages.filter((p) => p.id !== id);
      const activeId =
        s.activeId === id ? pages[0]?.id ?? null : s.activeId;
      return { pages, activeId };
    }),
  setActive: (id) => set({ activeId: id }),
  clear: () => set({ pages: [], activeId: null }),
}));

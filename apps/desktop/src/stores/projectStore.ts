import { create } from "zustand";
import type { DetectedBoundary, ImportedPage } from "../types/bindings";

interface ProjectState {
  pages: ImportedPage[];
  activeId: string | null;
  boundaries: Record<string, DetectedBoundary>;
  addPages: (p: ImportedPage[]) => void;
  removePage: (id: string) => void;
  setActive: (id: string) => void;
  setBoundary: (id: string, b: DetectedBoundary) => void;
  clear: () => void;
}

export const useProject = create<ProjectState>((set) => ({
  pages: [],
  activeId: null,
  boundaries: {},
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
      const activeId = s.activeId === id ? pages[0]?.id ?? null : s.activeId;
      const { [id]: _removed, ...boundaries } = s.boundaries;
      return { pages, activeId, boundaries };
    }),
  setActive: (id) => set({ activeId: id }),
  setBoundary: (id, b) =>
    set((s) => ({ boundaries: { ...s.boundaries, [id]: b } })),
  clear: () => set({ pages: [], activeId: null, boundaries: {} }),
}));

import { create } from "zustand";
import type {
  Corner,
  DetectedBoundary,
  EnhancedPage,
  EnhancementParams,
  ImportedPage,
  WarpedPage,
} from "../types/bindings";

export interface BoundaryState {
  detected: DetectedBoundary | null;
  working: DetectedBoundary | null;
  edited: boolean;
}

/// A persisted warp recipe: the corners the user applied and the resulting
/// cached preview. Rendering is a function of recipe + source hash, so the
/// recipe (not the pixels) is what belongs in a `.docsnap`.
export interface WarpRecipe {
  corners: [Corner, Corner, Corner, Corner];
  warped: WarpedPage;
}

export interface EnhancementRecipe {
  params: EnhancementParams;
  rendered: EnhancedPage;
}

export const defaultEnhancement = (): EnhancementParams => ({
  preset: "Original",
  brightness: 0,
  contrast: 0,
  sharpness: 0,
  shadow_remove: 0,
});

interface ProjectState {
  pages: ImportedPage[];
  activeId: string | null;
  boundaries: Record<string, BoundaryState>;
  warps: Record<string, WarpRecipe>;
  enhancements: Record<string, EnhancementRecipe>;
  addPages: (p: ImportedPage[]) => void;
  removePage: (id: string) => void;
  setActive: (id: string) => void;
  setDetected: (id: string, b: DetectedBoundary) => void;
  setWorking: (id: string, b: DetectedBoundary) => void;
  resetBoundary: (id: string) => void;
  setWarp: (id: string, w: WarpRecipe) => void;
  clearWarp: (id: string) => void;
  setEnhancement: (id: string, e: EnhancementRecipe) => void;
  clearEnhancement: (id: string) => void;
  clear: () => void;
}

export const useProject = create<ProjectState>((set) => ({
  pages: [],
  activeId: null,
  boundaries: {},
  warps: {},
  enhancements: {},
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
      const { [id]: _rb, ...boundaries } = s.boundaries;
      const { [id]: _rw, ...warps } = s.warps;
      const { [id]: _re, ...enhancements } = s.enhancements;
      return { pages, activeId, boundaries, warps, enhancements };
    }),
  setActive: (id) => set({ activeId: id }),
  setDetected: (id, b) =>
    set((s) => ({
      boundaries: {
        ...s.boundaries,
        [id]: { detected: b, working: b, edited: false },
      },
    })),
  setWorking: (id, b) =>
    set((s) => {
      const prev = s.boundaries[id];
      return {
        boundaries: {
          ...s.boundaries,
          [id]: {
            detected: prev?.detected ?? null,
            working: b,
            edited: true,
          },
        },
      };
    }),
  resetBoundary: (id) =>
    set((s) => {
      const prev = s.boundaries[id];
      if (!prev?.detected) return {};
      return {
        boundaries: {
          ...s.boundaries,
          [id]: { detected: prev.detected, working: prev.detected, edited: false },
        },
      };
    }),
  setWarp: (id, w) => set((s) => ({ warps: { ...s.warps, [id]: w } })),
  clearWarp: (id) =>
    set((s) => {
      const { [id]: _r, ...warps } = s.warps;
      return { warps };
    }),
  setEnhancement: (id, e) =>
    set((s) => ({ enhancements: { ...s.enhancements, [id]: e } })),
  clearEnhancement: (id) =>
    set((s) => {
      const { [id]: _r, ...enhancements } = s.enhancements;
      return { enhancements };
    }),
  clear: () => set({
    pages: [], activeId: null, boundaries: {}, warps: {}, enhancements: {},
  }),
}));

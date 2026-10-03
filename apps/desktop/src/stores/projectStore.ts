import { create } from "zustand";
import type {
  Corner,
  DetectedBoundary,
  EnhancedPage,
  EnhancementParams,
  ImportedPage,
  OcrResult,
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
  ocrResults: Record<string, OcrResult>;
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
  setOcr: (id: string, r: OcrResult) => void;
  updateOcrText: (id: string, text: string) => void;
  clearOcr: (id: string) => void;
  reorderPages: (from: number, to: number) => void;
  rotatePage: (id: string) => void;
  renamePage: (id: string, name: string) => void;
  clear: () => void;
}

export const useProject = create<ProjectState>((set) => ({
  pages: [],
  activeId: null,
  boundaries: {},
  warps: {},
  enhancements: {},
  ocrResults: {},
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
      const { [id]: _ro, ...ocrResults } = s.ocrResults;
      return { pages, activeId, boundaries, warps, enhancements, ocrResults };
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
  setOcr: (id, r) => set((s) => ({ ocrResults: { ...s.ocrResults, [id]: r } })),
  updateOcrText: (id, text) =>
    set((s) => {
      const prev = s.ocrResults[id];
      if (!prev) return {};
      return { ocrResults: { ...s.ocrResults, [id]: { ...prev, text } } };
    }),
  clearOcr: (id) =>
    set((s) => {
      const { [id]: _r, ...ocrResults } = s.ocrResults;
      return { ocrResults };
    }),
  reorderPages: (from, to) =>
    set((s) => {
      if (from === to) return {};
      const pages = s.pages.slice();
      const [moved] = pages.splice(from, 1);
      pages.splice(Math.max(0, Math.min(pages.length, to)), 0, moved);
      return { pages };
    }),
  rotatePage: (_id) => ({}),
  renamePage: (_id, _name) => ({}),
  clear: () => set({
    pages: [], activeId: null, boundaries: {}, warps: {}, enhancements: {},
    ocrResults: {},
  }),
}));

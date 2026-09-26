import { create } from "zustand";
import type { ConversionJob, BatchProgress } from "@/types/conversion";

interface BatchState {
  jobs: ConversionJob[];
  isProcessing: boolean;
  isPaused: boolean;
  progress: BatchProgress | null;

  addJobs: (jobs: ConversionJob[]) => void;
  updateJob: (jobId: string, updates: Partial<ConversionJob>) => void;
  removeJob: (jobId: string) => void;
  clearCompleted: () => void;
  clearAll: () => void;
  setIsProcessing: (processing: boolean) => void;
  setIsPaused: (paused: boolean) => void;
  setProgress: (progress: BatchProgress | null) => void;
}

export const useBatchStore = create<BatchState>((set) => ({
  jobs: [],
  isProcessing: false,
  isPaused: false,
  progress: null,

  addJobs: (newJobs) => set((state) => ({ jobs: [...state.jobs, ...newJobs] })),
  updateJob: (jobId, updates) =>
    set((state) => ({
      jobs: state.jobs.map((j) => (j.id === jobId ? { ...j, ...updates } : j)),
    })),
  removeJob: (jobId) =>
    set((state) => ({ jobs: state.jobs.filter((j) => j.id !== jobId) })),
  clearCompleted: () =>
    set((state) => ({
      jobs: state.jobs.filter((j) => j.status !== "completed"),
    })),
  clearAll: () => set({ jobs: [], isProcessing: false, isPaused: false, progress: null }),
  setIsProcessing: (processing) => set({ isProcessing: processing }),
  setIsPaused: (paused) => set({ isPaused: paused }),
  setProgress: (progress) => set({ progress }),
}));

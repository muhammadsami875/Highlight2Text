import { create } from "zustand";
import type { ConversionJob, ConversionOptions, DetectedFile } from "@/types/conversion";
import type { SupportLevel } from "@/types/formats";

interface OutputFormatOption {
  format: string;
  level: SupportLevel;
  notes?: string;
}

interface ConversionState {
  inputFile: DetectedFile | null;
  outputFormat: string | null;
  outputFormats: OutputFormatOption[];
  options: ConversionOptions;
  currentJob: ConversionJob | null;
  isConverting: boolean;

  setInputFile: (file: DetectedFile | null) => void;
  setOutputFormat: (format: string | null) => void;
  setOutputFormats: (formats: OutputFormatOption[]) => void;
  setOptions: (options: Partial<ConversionOptions>) => void;
  resetOptions: () => void;
  setCurrentJob: (job: ConversionJob | null) => void;
  setIsConverting: (converting: boolean) => void;
  reset: () => void;
}

const defaultOptions: ConversionOptions = {
  dpi: 150,
  imageQuality: 85,
  pageSize: "A4",
  orientation: "portrait",
  fontSize: 11,
  fontFamily: "monospace",
  lineNumbers: true,
  lineWrapping: false,
  syntaxTheme: "monokai",
  fitToPage: true,
  ocrEnabled: false,
  ocrLanguage: "eng",
  preserveLayout: true,
};

export const useConversionStore = create<ConversionState>((set) => ({
  inputFile: null,
  outputFormat: null,
  outputFormats: [],
  options: { ...defaultOptions },
  currentJob: null,
  isConverting: false,

  setInputFile: (file) => set({ inputFile: file, outputFormat: null, outputFormats: [], currentJob: null }),
  setOutputFormat: (format) => set({ outputFormat: format }),
  setOutputFormats: (formats) => set({ outputFormats: formats }),
  setOptions: (opts) => set((state) => ({ options: { ...state.options, ...opts } })),
  resetOptions: () => set({ options: { ...defaultOptions } }),
  setCurrentJob: (job) => set({ currentJob: job }),
  setIsConverting: (converting) => set({ isConverting: converting }),
  reset: () =>
    set({
      inputFile: null,
      outputFormat: null,
      outputFormats: [],
      options: { ...defaultOptions },
      currentJob: null,
      isConverting: false,
    }),
}));

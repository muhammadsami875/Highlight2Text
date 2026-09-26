export type ThemeMode = "light" | "dark" | "system";

export type OutputDirMode = "same" | "custom" | "ask";

export type OverwriteMode = "rename" | "overwrite" | "ask";

export interface AppSettings {
  general: GeneralSettings;
  conversion: ConversionSettings;
  pdf: PdfSettings;
  images: ImageSettings;
  ocr: OcrSettings;
  advanced: AdvancedSettings;
}

export interface GeneralSettings {
  theme: ThemeMode;
  language: string;
  startMinimized: boolean;
  showInSystemTray: boolean;
}

export interface ConversionSettings {
  outputDirMode: OutputDirMode;
  customOutputDir: string;
  overwriteMode: OverwriteMode;
  fileSuffix: string;
  tempDir: string;
  maxParallelJobs: number;
}

export interface PdfSettings {
  defaultDpi: number;
  defaultCompression: string;
  defaultPageSize: string;
  defaultOrientation: "portrait" | "landscape";
}

export interface ImageSettings {
  defaultQuality: number;
  defaultFormat: string;
  preserveMetadata: boolean;
}

export interface OcrSettings {
  language: string;
  engine: string;
  autoDetect: boolean;
}

export interface AdvancedSettings {
  enableExperimental: boolean;
  enableDiagnosticLogs: boolean;
  maxFileSizeMb: number;
  processTimeoutSec: number;
}

export const DEFAULT_SETTINGS: AppSettings = {
  general: {
    theme: "system",
    language: "en",
    startMinimized: false,
    showInSystemTray: false,
  },
  conversion: {
    outputDirMode: "same",
    customOutputDir: "",
    overwriteMode: "rename",
    fileSuffix: "-converted",
    tempDir: "",
    maxParallelJobs: 2,
  },
  pdf: {
    defaultDpi: 150,
    defaultCompression: "auto",
    defaultPageSize: "A4",
    defaultOrientation: "portrait",
  },
  images: {
    defaultQuality: 85,
    defaultFormat: "png",
    preserveMetadata: true,
  },
  ocr: {
    language: "eng",
    engine: "tesseract",
    autoDetect: true,
  },
  advanced: {
    enableExperimental: false,
    enableDiagnosticLogs: false,
    maxFileSizeMb: 2048,
    processTimeoutSec: 300,
  },
};

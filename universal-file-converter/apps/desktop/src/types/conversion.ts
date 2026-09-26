export type JobStatus =
  | "queued"
  | "detecting"
  | "planning"
  | "preparing"
  | "converting"
  | "validating"
  | "completed"
  | "failed"
  | "cancelled";

export interface DetectedFile {
  path: string;
  name: string;
  extension: string;
  detectedFormat: string;
  mimeType: string;
  sizeBytes: number;
  formatMismatch: boolean;
  metadata: FileMetadata;
}

export interface FileMetadata {
  pageCount?: number;
  sheetCount?: number;
  imageWidth?: number;
  imageHeight?: number;
  durationMs?: number;
  hasTextLayer?: boolean;
  isPasswordProtected?: boolean;
}

export interface ConversionOptions {
  pageRange?: string;
  dpi?: number;
  imageQuality?: number;
  compression?: string;
  pageSize?: string;
  orientation?: "portrait" | "landscape";
  margins?: { top: number; right: number; bottom: number; left: number };
  fontSize?: number;
  fontFamily?: string;
  lineNumbers?: boolean;
  lineWrapping?: boolean;
  syntaxTheme?: string;
  sheetSelection?: string[];
  fitToPage?: boolean;
  ocrEnabled?: boolean;
  ocrLanguage?: string;
  preserveLayout?: boolean;
  backgroundColor?: string;
  password?: string;
}

export interface ConversionPlanStep {
  from: string;
  to: string;
  engineId: string;
  engineName: string;
}

export interface ConversionPlan {
  steps: ConversionPlanStep[];
  estimatedQuality: number;
  estimatedDuration: string;
}

export interface ConversionJob {
  id: string;
  inputFile: DetectedFile;
  outputFormat: string;
  outputPath: string;
  options: ConversionOptions;
  plan: ConversionPlan | null;
  status: JobStatus;
  progress: number;
  progressMessage: string;
  startedAt: string | null;
  completedAt: string | null;
  error: ConversionError | null;
  warnings: string[];
}

export interface ConversionError {
  code: string;
  message: string;
  details: string | null;
  recoverable: boolean;
}

export interface ConversionResult {
  jobId: string;
  success: boolean;
  outputPath: string | null;
  outputSize: number | null;
  duration: number;
  warnings: string[];
  error: ConversionError | null;
}

export interface BatchProgress {
  totalJobs: number;
  completedJobs: number;
  failedJobs: number;
  cancelledJobs: number;
  currentJobId: string | null;
  currentJobName: string | null;
  overallProgress: number;
  elapsedMs: number;
  estimatedRemainingMs: number | null;
}

export interface EngineInfo {
  id: string;
  name: string;
  version: string | null;
  available: boolean;
  inputFormats: string[];
  outputFormats: string[];
  license: string;
}

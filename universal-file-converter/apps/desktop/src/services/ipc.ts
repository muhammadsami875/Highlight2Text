import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ConversionJob,
  ConversionOptions,
  ConversionResult,
  DetectedFile,
  EngineInfo,
  BatchProgress,
} from "@/types/conversion";
import type { AppSettings } from "@/types/settings";
import type { SupportLevel } from "@/types/formats";

export interface SupportedOutput {
  format: string;
  level: SupportLevel;
  notes?: string;
}

export interface ProgressEvent {
  jobId: string;
  progress: number;
  message: string;
  status: string;
}

export async function onConversionProgress(
  callback: (event: ProgressEvent) => void
): Promise<UnlistenFn> {
  return listen<ProgressEvent>("conversion-progress", (event) => {
    callback(event.payload);
  });
}

export async function detectFile(path: string): Promise<DetectedFile> {
  return invoke("detect_file", { path });
}

export async function getSupportedOutputs(
  inputFormat: string
): Promise<SupportedOutput[]> {
  return invoke("get_supported_outputs", { inputFormat });
}

export async function startConversion(
  inputPath: string,
  outputFormat: string,
  outputDir: string,
  options: ConversionOptions
): Promise<string> {
  return invoke("start_conversion", {
    inputPath,
    outputFormat,
    outputDir,
    options,
  });
}

export async function getJobStatus(jobId: string): Promise<ConversionJob> {
  return invoke("get_job_status", { jobId });
}

export async function cancelJob(jobId: string): Promise<void> {
  return invoke("cancel_job", { jobId });
}

export async function retryJob(jobId: string): Promise<string> {
  return invoke("retry_job", { jobId });
}

export async function startBatch(
  files: Array<{ inputPath: string; outputFormat: string }>,
  outputDir: string,
  options: ConversionOptions
): Promise<string[]> {
  return invoke("start_batch", { files, outputDir, options });
}

export async function getBatchProgress(
  jobIds: string[]
): Promise<BatchProgress> {
  return invoke("get_batch_progress", { jobIds });
}

export async function cancelAllJobs(): Promise<void> {
  return invoke("cancel_all_jobs");
}

export async function getConversionHistory(): Promise<ConversionResult[]> {
  return invoke("get_conversion_history");
}

export async function clearHistory(): Promise<void> {
  return invoke("clear_history");
}

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke("save_settings", { settings });
}

export async function getEngineStatus(): Promise<EngineInfo[]> {
  return invoke("get_engine_status");
}

export async function openFileLocation(path: string): Promise<void> {
  return invoke("open_file_location", { path });
}

export async function getConversionPlan(
  inputFormat: string,
  outputFormat: string
): Promise<import("@/types/conversion").ConversionPlan | null> {
  return invoke("get_conversion_plan", { inputFormat, outputFormat });
}

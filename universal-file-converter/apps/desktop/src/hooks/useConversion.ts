import { useState, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { startConversion, getJobStatus, cancelJob } from "@/services/ipc";
import { useConversionStore } from "@/stores/conversionStore";
import { useHistoryStore } from "@/stores/historyStore";
import type { ConversionJob } from "@/types/conversion";

export function useConversion() {
  const [error, setError] = useState<string | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);
  const {
    inputFile,
    outputFormat,
    options,
    setCurrentJob,
    setIsConverting,
  } = useConversionStore();
  const { addRecentPair } = useHistoryStore();

  const convert = useCallback(
    async (outputDir: string) => {
      if (!inputFile || !outputFormat) return;
      setError(null);
      setIsConverting(true);

      try {
        const jobId = await startConversion(
          inputFile.path,
          outputFormat,
          outputDir,
          options
        );

        unlistenRef.current = await listen<ConversionJob>(
          `conversion-progress-${jobId}`,
          (event) => {
            setCurrentJob(event.payload);
            if (
              event.payload.status === "completed" ||
              event.payload.status === "failed" ||
              event.payload.status === "cancelled"
            ) {
              setIsConverting(false);
              unlistenRef.current?.();
              unlistenRef.current = null;
            }
          }
        );

        const status = await getJobStatus(jobId);
        setCurrentJob(status);
        addRecentPair(inputFile.detectedFormat, outputFormat);

        return jobId;
      } catch (err) {
        setIsConverting(false);
        const message =
          err instanceof Error ? err.message : "Conversion failed";
        setError(message);
        return null;
      }
    },
    [inputFile, outputFormat, options, setCurrentJob, setIsConverting, addRecentPair]
  );

  const cancel = useCallback(
    async (jobId: string) => {
      try {
        await cancelJob(jobId);
      } catch (err) {
        setError(
          err instanceof Error ? err.message : "Failed to cancel conversion"
        );
      }
    },
    []
  );

  return { convert, cancel, error };
}

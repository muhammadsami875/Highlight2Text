import { useState, useCallback, useRef, useEffect } from "react";
import {
  startConversion,
  getJobStatus,
  cancelJob,
  onConversionProgress,
} from "@/services/ipc";
import { useConversionStore } from "@/stores/conversionStore";
import { useHistoryStore } from "@/stores/historyStore";
import { useToastStore } from "@/stores/toastStore";
import type { ProgressEvent } from "@/services/ipc";

export function useConversion() {
  const [error, setError] = useState<string | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);
  const activeJobRef = useRef<string | null>(null);
  const {
    inputFile,
    outputFormat,
    options,
    setCurrentJob,
    setIsConverting,
  } = useConversionStore();
  const { addRecentPair } = useHistoryStore();
  const { addToast } = useToastStore();

  useEffect(() => {
    let mounted = true;

    onConversionProgress((event: ProgressEvent) => {
      if (!mounted) return;
      if (activeJobRef.current && event.jobId === activeJobRef.current) {
        if (event.status === "completed") {
          setIsConverting(false);
          addToast("success", "Conversion completed successfully");
          getJobStatus(event.jobId).then((job) => {
            if (mounted) setCurrentJob(job);
          });
        } else if (event.status === "failed") {
          setIsConverting(false);
          addToast("error", event.message || "Conversion failed");
          getJobStatus(event.jobId).then((job) => {
            if (mounted) setCurrentJob(job);
          });
        } else if (event.status === "cancelled") {
          setIsConverting(false);
          getJobStatus(event.jobId).then((job) => {
            if (mounted) setCurrentJob(job);
          });
        }
      }
    }).then((unlisten) => {
      if (mounted) {
        unlistenRef.current = unlisten;
      } else {
        unlisten();
      }
    });

    return () => {
      mounted = false;
      unlistenRef.current?.();
      unlistenRef.current = null;
    };
  }, [setCurrentJob, setIsConverting]);

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

        activeJobRef.current = jobId;

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

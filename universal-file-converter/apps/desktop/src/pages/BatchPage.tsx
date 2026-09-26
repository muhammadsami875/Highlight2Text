import { useState, useEffect, useCallback } from "react";
import {
  Layers,
  Trash2,
  Play,
  Pause,
  CheckCircle2,
  XCircle,
} from "lucide-react";
import { DropZone } from "@/components/common/DropZone";
import { ProgressBar } from "@/components/common/ProgressBar";
import { StatusBadge } from "@/components/common/StatusBadge";
import { useBatchStore } from "@/stores/batchStore";
import { truncateFilename } from "@/utils/formatters";
import {
  detectFile,
  getSupportedOutputs,
  startBatch,
  cancelAllJobs,
  onConversionProgress,
} from "@/services/ipc";
import type { DetectedFile } from "@/types/conversion";
import { t } from "@/i18n";

interface BatchItem {
  id: string;
  file: DetectedFile;
  outputFormat: string;
  status: "pending" | "queued" | "converting" | "completed" | "failed";
  progress: number;
  error?: string;
}

export function BatchPage() {
  const { clearAll } = useBatchStore();
  const [items, setItems] = useState<BatchItem[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const [jobIdMap, setJobIdMap] = useState<Record<string, string>>({});
  useEffect(() => {
    let mounted = true;
    const unlistenPromise = onConversionProgress((event) => {
      if (!mounted) return;
      const itemId = Object.entries(jobIdMap).find(
        ([, jid]) => jid === event.jobId
      )?.[0];
      if (!itemId) return;

      setItems((prev) =>
        prev.map((it) => {
          if (it.id !== itemId) return it;
          if (event.status === "completed") {
            return { ...it, status: "completed", progress: 100 };
          }
          if (event.status === "failed") {
            return { ...it, status: "failed", error: event.message };
          }
          return { ...it, status: "converting", progress: event.progress * 100 };
        })
      );
    });

    return () => {
      mounted = false;
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [jobIdMap]);

  const handleFilesDropped = useCallback(async (paths: string[]) => {
    const newItems: BatchItem[] = [];

    for (const path of paths) {
      try {
        const detected = await detectFile(path);
        const routes = await getSupportedOutputs(detected.detectedFormat);
        const defaultOutput = routes.length > 0 ? routes[0].format : "pdf";

        newItems.push({
          id: crypto.randomUUID(),
          file: detected,
          outputFormat: defaultOutput,
          status: "pending",
          progress: 0,
        });
      } catch {
        // skip files that can't be detected
      }
    }

    setItems((prev) => [...prev, ...newItems]);
  }, []);

  const handleStartAll = useCallback(async () => {
    const pending = items.filter((it) => it.status === "pending" || it.status === "failed");
    if (pending.length === 0) return;

    setIsProcessing(true);

    try {
      const files = pending.map((it) => ({
        inputPath: it.file.path,
        outputFormat: it.outputFormat,
      }));

      const firstFile = pending[0].file.path;
      const outputDir = firstFile.substring(
        0,
        firstFile.lastIndexOf(/[\\/]/.test(firstFile) ? (firstFile.includes("\\") ? "\\" : "/") : "/")
      ) || ".";

      const jobIds = await startBatch(files, outputDir, {});

      const newMap: Record<string, string> = {};
      pending.forEach((it, i) => {
        if (jobIds[i]) newMap[it.id] = jobIds[i];
      });
      setJobIdMap((prev) => ({ ...prev, ...newMap }));

      setItems((prev) =>
        prev.map((it) => {
          if (newMap[it.id]) return { ...it, status: "queued", progress: 0 };
          return it;
        })
      );
    } catch {
      setIsProcessing(false);
    }
  }, [items]);

  const handleCancelAll = useCallback(async () => {
    try {
      await cancelAllJobs();
      setIsProcessing(false);
      setItems((prev) =>
        prev.map((it) =>
          it.status === "queued" || it.status === "converting"
            ? { ...it, status: "pending", progress: 0 }
            : it
        )
      );
    } catch {
      // silent
    }
  }, []);

  const handleClearCompleted = useCallback(() => {
    setItems((prev) => prev.filter((it) => it.status !== "completed"));
  }, []);

  const handleClearAll = useCallback(() => {
    setItems([]);
    setIsProcessing(false);
    setJobIdMap({});
    clearAll();
  }, [clearAll]);

  const completedCount = items.filter((it) => it.status === "completed").length;
  const overallProgress =
    items.length > 0 ? Math.round((completedCount / items.length) * 100) : 0;

  useEffect(() => {
    if (items.length > 0 && items.every((it) => it.status === "completed" || it.status === "failed")) {
      setIsProcessing(false);
    }
  }, [items]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-bold text-surface-900 dark:text-surface-100">
          {t("batch.title")}
        </h2>
        <div className="flex gap-2">
          {items.length > 0 && (
            <>
              <button onClick={handleClearCompleted} className="btn-ghost text-sm">
                <CheckCircle2 size={14} className="mr-1.5 inline" />
                {t("batch.removeCompleted")}
              </button>
              <button onClick={handleClearAll} className="btn-ghost text-sm text-red-500">
                <Trash2 size={14} className="mr-1.5 inline" />
                {t("batch.clearAll")}
              </button>
            </>
          )}
        </div>
      </div>

      <DropZone onFilesDropped={handleFilesDropped} compact />

      {items.length > 0 && (
        <>
          <div className="card p-4">
            <div className="flex items-center justify-between mb-3">
              <span className="text-sm text-surface-600 dark:text-surface-400">
                {t("batch.progress", {
                  completed: String(completedCount),
                  total: String(items.length),
                })}
              </span>
              <div className="flex gap-2">
                {!isProcessing ? (
                  <button
                    onClick={handleStartAll}
                    className="btn-primary text-sm"
                    disabled={items.every((it) => it.status === "completed")}
                  >
                    <Play size={14} className="mr-1.5 inline" />
                    {t("batch.startAll")}
                  </button>
                ) : (
                  <button onClick={handleCancelAll} className="btn-secondary text-sm">
                    <Pause size={14} className="mr-1.5 inline" />
                    {t("batch.pauseAll")}
                  </button>
                )}
              </div>
            </div>
            <ProgressBar progress={overallProgress} showPercentage />
          </div>

          <div className="space-y-2">
            {items.map((item) => (
              <div key={item.id} className="card p-3 flex items-center gap-3">
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium text-surface-800 dark:text-surface-200 truncate">
                    {truncateFilename(item.file.name, 50)}
                  </p>
                  <p className="text-xs text-surface-400 dark:text-surface-500 mt-0.5">
                    {item.file.detectedFormat.toUpperCase()} &rarr;{" "}
                    {item.outputFormat.toUpperCase()}
                  </p>
                  {item.status === "converting" && (
                    <div className="mt-1.5">
                      <ProgressBar progress={item.progress} />
                    </div>
                  )}
                </div>
                <div className="flex items-center gap-2">
                  {item.status === "completed" && (
                    <CheckCircle2 size={16} className="text-emerald-500" />
                  )}
                  {item.status === "failed" && (
                    <XCircle size={16} className="text-red-500" />
                  )}
                  <StatusBadge status={item.status === "pending" ? "queued" : item.status} />
                </div>
              </div>
            ))}
          </div>
        </>
      )}

      {items.length === 0 && (
        <div className="text-center py-12">
          <Layers
            size={48}
            className="mx-auto mb-3 text-surface-300 dark:text-surface-600"
          />
          <p className="text-surface-400 dark:text-surface-500">
            {t("batch.queueEmpty")}
          </p>
          <p className="text-sm text-surface-300 dark:text-surface-600 mt-1">
            Drop files above or click Add Files to begin
          </p>
        </div>
      )}
    </div>
  );
}

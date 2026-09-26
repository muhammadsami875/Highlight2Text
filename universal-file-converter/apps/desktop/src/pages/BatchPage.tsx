import { Layers, Trash2, Play, Pause, RotateCcw, CheckCircle2 } from "lucide-react";
import { DropZone } from "@/components/common/DropZone";
import { ProgressBar } from "@/components/common/ProgressBar";
import { StatusBadge } from "@/components/common/StatusBadge";
import { useBatchStore } from "@/stores/batchStore";
import { useBatchQueue } from "@/hooks/useBatchQueue";
import { truncateFilename } from "@/utils/formatters";
import { t } from "@/i18n";

export function BatchPage() {
  const { clearAll, clearCompleted } = useBatchStore();
  const { jobs, isProcessing, stats } = useBatchQueue();

  const handleFilesDropped = (_paths: string[]) => {
    // Will be connected to backend in Phase 12
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-bold text-surface-900 dark:text-surface-100">
          {t("batch.title")}
        </h2>
        <div className="flex gap-2">
          {jobs.length > 0 && (
            <>
              <button onClick={clearCompleted} className="btn-ghost text-sm">
                <CheckCircle2 size={14} className="mr-1.5 inline" />
                {t("batch.removeCompleted")}
              </button>
              <button onClick={clearAll} className="btn-ghost text-sm text-red-500">
                <Trash2 size={14} className="mr-1.5 inline" />
                {t("batch.clearAll")}
              </button>
            </>
          )}
        </div>
      </div>

      <DropZone onFilesDropped={handleFilesDropped} compact />

      {jobs.length > 0 && (
        <>
          <div className="card p-4">
            <div className="flex items-center justify-between mb-3">
              <span className="text-sm text-surface-600 dark:text-surface-400">
                {t("batch.progress", {
                  completed: String(stats.completed),
                  total: String(stats.total),
                })}
              </span>
              <div className="flex gap-2">
                <button className="btn-primary text-sm" disabled={isProcessing}>
                  <Play size={14} className="mr-1.5 inline" />
                  {t("batch.startAll")}
                </button>
                {isProcessing && (
                  <button className="btn-secondary text-sm">
                    <Pause size={14} className="mr-1.5 inline" />
                    {t("batch.pauseAll")}
                  </button>
                )}
              </div>
            </div>
            <ProgressBar progress={stats.overallProgress} showPercentage />
          </div>

          <div className="space-y-2">
            {jobs.map((job) => (
              <div
                key={job.id}
                className="card p-3 flex items-center gap-3"
              >
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium text-surface-800 dark:text-surface-200 truncate">
                    {truncateFilename(job.inputFile.name, 50)}
                  </p>
                  <p className="text-xs text-surface-400 dark:text-surface-500 mt-0.5">
                    {job.inputFile.detectedFormat.toUpperCase()} &rarr;{" "}
                    {job.outputFormat.toUpperCase()}
                  </p>
                </div>
                <StatusBadge status={job.status} />
                {job.status === "failed" && (
                  <button className="btn-ghost p-1.5" title="Retry">
                    <RotateCcw size={14} />
                  </button>
                )}
              </div>
            ))}
          </div>
        </>
      )}

      {jobs.length === 0 && (
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

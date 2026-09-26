import { useEffect } from "react";
import { Clock, FolderOpen, RotateCcw, Trash2 } from "lucide-react";
import { useHistoryStore } from "@/stores/historyStore";
import { formatFileSize } from "@/utils/fileSize";
import { formatDuration } from "@/utils/fileSize";
import {
  getConversionHistory,
  clearHistory as clearHistoryIpc,
  openFileLocation,
} from "@/services/ipc";
import { t } from "@/i18n";

export function HistoryPage() {
  const { history, setHistory, clearHistory } = useHistoryStore();

  useEffect(() => {
    getConversionHistory()
      .then(setHistory)
      .catch(() => {});
  }, [setHistory]);

  const handleClearHistory = async () => {
    try {
      await clearHistoryIpc();
      clearHistory();
    } catch {
      // silent
    }
  };

  const handleOpenFolder = (path: string) => {
    openFileLocation(path).catch(() => {});
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-bold text-surface-900 dark:text-surface-100">
          {t("history.title")}
        </h2>
        {history.length > 0 && (
          <button
            onClick={handleClearHistory}
            className="btn-ghost text-sm text-red-500"
          >
            <Trash2 size={14} className="mr-1.5 inline" />
            {t("history.clearHistory")}
          </button>
        )}
      </div>

      {history.length === 0 ? (
        <div className="text-center py-16">
          <Clock
            size={48}
            className="mx-auto mb-3 text-surface-300 dark:text-surface-600"
          />
          <p className="text-surface-400 dark:text-surface-500">
            {t("history.noHistory")}
          </p>
        </div>
      ) : (
        <div className="space-y-2">
          {history.map((item) => (
            <div key={item.jobId} className="card p-4">
              <div className="flex items-start justify-between">
                <div>
                  <p className="text-sm font-medium text-surface-800 dark:text-surface-200">
                    {item.outputPath?.split(/[\\/]/).pop() ?? "Unknown"}
                  </p>
                  <div className="flex items-center gap-3 mt-1 text-xs text-surface-400 dark:text-surface-500">
                    {item.outputSize && (
                      <span>{formatFileSize(item.outputSize)}</span>
                    )}
                    <span>{formatDuration(item.duration)}</span>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <span
                    className={`badge ${item.success ? "badge-success" : "badge-error"}`}
                  >
                    {item.success ? t("status.completed") : t("status.failed")}
                  </span>
                </div>
              </div>
              <div className="flex items-center gap-2 mt-3">
                {item.success && item.outputPath && (
                  <button
                    onClick={() => handleOpenFolder(item.outputPath!)}
                    className="btn-ghost text-xs py-1"
                  >
                    <FolderOpen size={12} className="mr-1 inline" />
                    {t("history.openFolder")}
                  </button>
                )}
                <button className="btn-ghost text-xs py-1">
                  <RotateCcw size={12} className="mr-1 inline" />
                  {t("history.repeat")}
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

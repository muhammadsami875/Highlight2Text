import { ArrowDown, Play, Loader2, CheckCircle2, XCircle } from "lucide-react";
import { DropZone } from "@/components/common/DropZone";
import { FileCard } from "@/components/file/FileCard";
import { FormatSelector } from "@/components/conversion/FormatSelector";
import { ConversionOptionsPanel } from "@/components/conversion/ConversionOptions";
import { ConversionPlanPreview } from "@/components/conversion/ConversionPlanPreview";
import { ProgressBar } from "@/components/common/ProgressBar";
import { StatusBadge } from "@/components/common/StatusBadge";
import { useConversionStore } from "@/stores/conversionStore";
import { useFileDetection } from "@/hooks/useFileDetection";
import { useConversion } from "@/hooks/useConversion";
import { t } from "@/i18n";

export function ConvertPage() {
  const {
    inputFile,
    outputFormat,
    outputFormats,
    currentJob,
    isConverting,
    setOutputFormat,
    reset,
  } = useConversionStore();
  const { detect } = useFileDetection();
  const { convert, error } = useConversion();

  const handleFilesDropped = async (paths: string[]) => {
    if (paths.length > 0) {
      await detect(paths[0]);
    }
  };

  const handleConvert = async () => {
    if (!inputFile || !outputFormat) return;
    const dir = inputFile.path.substring(
      0,
      inputFile.path.lastIndexOf(/[\\/]/.test(inputFile.path) ? (inputFile.path.includes("\\") ? "\\" : "/") : "/")
    );
    await convert(dir || ".");
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-bold text-surface-900 dark:text-surface-100">
          {t("nav.convert")}
        </h2>
        {inputFile && (
          <button onClick={reset} className="btn-ghost text-sm">
            Start Over
          </button>
        )}
      </div>

      {!inputFile ? (
        <DropZone onFilesDropped={handleFilesDropped}>
          <div className="flex flex-col items-center gap-3">
            <div className="p-4 rounded-full bg-primary-50 dark:bg-primary-950/30">
              <ArrowDown
                size={32}
                className="text-primary-500 dark:text-primary-400"
              />
            </div>
            <p className="text-lg font-medium text-surface-500 dark:text-surface-400">
              {t("home.dropzone")}
            </p>
            <p className="text-sm text-surface-400 dark:text-surface-500">
              or click to browse
            </p>
          </div>
        </DropZone>
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="space-y-4">
            <h3 className="text-sm font-semibold text-surface-600 dark:text-surface-400 uppercase tracking-wider">
              {t("convert.inputFile")}
            </h3>
            <FileCard file={inputFile} onRemove={reset} />
          </div>

          <div className="space-y-4">
            <h3 className="text-sm font-semibold text-surface-600 dark:text-surface-400 uppercase tracking-wider">
              {t("convert.outputFormat")}
            </h3>
            <FormatSelector
              formats={outputFormats}
              selected={outputFormat}
              onSelect={setOutputFormat}
            />
          </div>

          <div className="space-y-4">
            <h3 className="text-sm font-semibold text-surface-600 dark:text-surface-400 uppercase tracking-wider">
              {t("convert.options")}
            </h3>
            <ConversionOptionsPanel />
          </div>
        </div>
      )}

      {inputFile && outputFormat && (
        <div className="card p-4 space-y-4">
          <ConversionPlanPreview
            inputFormat={inputFile.detectedFormat}
            outputFormat={outputFormat}
          />
          {currentJob ? (
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  {currentJob.status === "completed" && (
                    <CheckCircle2 size={18} className="text-emerald-500" />
                  )}
                  {currentJob.status === "failed" && (
                    <XCircle size={18} className="text-red-500" />
                  )}
                  {!["completed", "failed", "cancelled"].includes(
                    currentJob.status
                  ) && (
                    <Loader2 size={18} className="text-primary-500 animate-spin" />
                  )}
                  <span className="text-sm font-medium text-surface-700 dark:text-surface-300">
                    {currentJob.progressMessage || t(`status.${currentJob.status}`)}
                  </span>
                </div>
                <StatusBadge status={currentJob.status} />
              </div>
              <ProgressBar
                progress={currentJob.progress}
                indeterminate={
                  currentJob.status === "converting" && currentJob.progress === 0
                }
                color={
                  currentJob.status === "completed"
                    ? "success"
                    : currentJob.status === "failed"
                      ? "error"
                      : "primary"
                }
                showPercentage
              />
              {currentJob.error && (
                <div className="p-3 rounded-lg bg-red-50 dark:bg-red-950/20 border border-red-200 dark:border-red-800">
                  <p className="text-sm text-red-700 dark:text-red-400">
                    {currentJob.error.message}
                  </p>
                  {currentJob.error.details && (
                    <details className="mt-2">
                      <summary className="text-xs text-red-500 cursor-pointer">
                        Technical details
                      </summary>
                      <pre className="mt-1 text-xs text-red-600 dark:text-red-400 whitespace-pre-wrap font-mono">
                        {currentJob.error.details}
                      </pre>
                    </details>
                  )}
                </div>
              )}
              {currentJob.warnings.length > 0 && (
                <div className="p-3 rounded-lg bg-amber-50 dark:bg-amber-950/20 border border-amber-200 dark:border-amber-800">
                  {currentJob.warnings.map((w, i) => (
                    <p
                      key={i}
                      className="text-sm text-amber-700 dark:text-amber-400"
                    >
                      {w}
                    </p>
                  ))}
                </div>
              )}
            </div>
          ) : (
            <button
              onClick={handleConvert}
              disabled={isConverting}
              className="btn-primary w-full flex items-center justify-center gap-2"
            >
              {isConverting ? (
                <>
                  <Loader2 size={18} className="animate-spin" />
                  {t("convert.converting")}
                </>
              ) : (
                <>
                  <Play size={18} />
                  {t("convert.convertButton")}
                </>
              )}
            </button>
          )}
          {error && !currentJob && (
            <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
          )}
        </div>
      )}
    </div>
  );
}

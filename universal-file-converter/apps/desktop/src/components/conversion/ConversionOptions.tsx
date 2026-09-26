import { useConversionStore } from "@/stores/conversionStore";
import { t } from "@/i18n";

export function ConversionOptionsPanel() {
  const { inputFile, outputFormat, options, setOptions } = useConversionStore();

  if (!inputFile || !outputFormat) {
    return (
      <div className="card p-4">
        <p className="text-sm text-surface-400 dark:text-surface-500">
          {t("convert.noOptions")}
        </p>
      </div>
    );
  }

  const isImageOutput = ["png", "jpg", "webp", "bmp", "tiff"].includes(outputFormat);
  const isPdfOutput = outputFormat === "pdf";
  const isCodeInput = [
    "py", "js", "ts", "java", "cpp", "cs", "php", "css", "sql", "json", "xml",
  ].includes(inputFile.detectedFormat);
  const isPdfInput = inputFile.detectedFormat === "pdf";

  return (
    <div className="card p-4 space-y-4">
      <h3 className="text-sm font-semibold text-surface-700 dark:text-surface-300">
        {t("convert.options")}
      </h3>

      {isPdfInput && (
        <div>
          <label className="block text-xs font-medium text-surface-600 dark:text-surface-400 mb-1">
            Page Range
          </label>
          <input
            type="text"
            value={options.pageRange ?? ""}
            onChange={(e) => setOptions({ pageRange: e.target.value || undefined })}
            placeholder="All pages (e.g., 1-5, 8, 10-12)"
            className="input-field text-sm"
          />
        </div>
      )}

      {isPdfInput && (
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={options.ocrEnabled ?? false}
            onChange={(e) => setOptions({ ocrEnabled: e.target.checked })}
            className="rounded border-surface-300 dark:border-surface-600"
          />
          <span className="text-sm text-surface-700 dark:text-surface-300">
            Enable OCR (for scanned PDFs)
          </span>
        </label>
      )}

      {(isImageOutput || isPdfOutput) && (
        <div>
          <label className="block text-xs font-medium text-surface-600 dark:text-surface-400 mb-1">
            DPI
          </label>
          <select
            value={options.dpi ?? 150}
            onChange={(e) => setOptions({ dpi: parseInt(e.target.value) })}
            className="input-field text-sm"
          >
            <option value={72}>72 (Screen)</option>
            <option value={150}>150 (Standard)</option>
            <option value={300}>300 (High Quality)</option>
            <option value={600}>600 (Print)</option>
          </select>
        </div>
      )}

      {isImageOutput && (
        <div>
          <label className="block text-xs font-medium text-surface-600 dark:text-surface-400 mb-1">
            Quality ({options.imageQuality ?? 85}%)
          </label>
          <input
            type="range"
            min={1}
            max={100}
            value={options.imageQuality ?? 85}
            onChange={(e) => setOptions({ imageQuality: parseInt(e.target.value) })}
            className="w-full"
          />
        </div>
      )}

      {isPdfOutput && (
        <>
          <div>
            <label className="block text-xs font-medium text-surface-600 dark:text-surface-400 mb-1">
              Page Size
            </label>
            <select
              value={options.pageSize ?? "A4"}
              onChange={(e) => setOptions({ pageSize: e.target.value })}
              className="input-field text-sm"
            >
              <option value="A4">A4</option>
              <option value="Letter">Letter</option>
              <option value="Legal">Legal</option>
              <option value="A3">A3</option>
            </select>
          </div>
          <div>
            <label className="block text-xs font-medium text-surface-600 dark:text-surface-400 mb-1">
              Orientation
            </label>
            <select
              value={options.orientation ?? "portrait"}
              onChange={(e) =>
                setOptions({
                  orientation: e.target.value as "portrait" | "landscape",
                })
              }
              className="input-field text-sm"
            >
              <option value="portrait">Portrait</option>
              <option value="landscape">Landscape</option>
            </select>
          </div>
        </>
      )}

      {isCodeInput && isPdfOutput && (
        <>
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={options.lineNumbers ?? true}
              onChange={(e) => setOptions({ lineNumbers: e.target.checked })}
              className="rounded border-surface-300 dark:border-surface-600"
            />
            <span className="text-sm text-surface-700 dark:text-surface-300">
              Line Numbers
            </span>
          </label>
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={options.lineWrapping ?? false}
              onChange={(e) => setOptions({ lineWrapping: e.target.checked })}
              className="rounded border-surface-300 dark:border-surface-600"
            />
            <span className="text-sm text-surface-700 dark:text-surface-300">
              Line Wrapping
            </span>
          </label>
          <div>
            <label className="block text-xs font-medium text-surface-600 dark:text-surface-400 mb-1">
              Font Size
            </label>
            <select
              value={options.fontSize ?? 11}
              onChange={(e) => setOptions({ fontSize: parseInt(e.target.value) })}
              className="input-field text-sm"
            >
              {[8, 9, 10, 11, 12, 14, 16].map((s) => (
                <option key={s} value={s}>
                  {s}pt
                </option>
              ))}
            </select>
          </div>
          <div>
            <label className="block text-xs font-medium text-surface-600 dark:text-surface-400 mb-1">
              Theme
            </label>
            <select
              value={options.syntaxTheme ?? "monokai"}
              onChange={(e) => setOptions({ syntaxTheme: e.target.value })}
              className="input-field text-sm"
            >
              <option value="monokai">Monokai</option>
              <option value="github">GitHub</option>
              <option value="solarized-light">Solarized Light</option>
              <option value="solarized-dark">Solarized Dark</option>
              <option value="one-dark">One Dark</option>
            </select>
          </div>
        </>
      )}
    </div>
  );
}

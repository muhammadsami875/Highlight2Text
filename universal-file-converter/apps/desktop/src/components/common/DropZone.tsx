import { useState, useCallback, type DragEvent, type ReactNode, type KeyboardEvent } from "react";
import { clsx } from "clsx";
import { Upload } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { t } from "@/i18n";

interface DropZoneProps {
  onFilesDropped: (paths: string[]) => void;
  children?: ReactNode;
  className?: string;
  compact?: boolean;
  multiple?: boolean;
}

export function DropZone({
  onFilesDropped,
  children,
  className,
  compact = false,
  multiple = true,
}: DropZoneProps) {
  const [isDragOver, setIsDragOver] = useState(false);

  const handleDragOver = useCallback((e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);
  }, []);

  const handleDrop = useCallback(
    (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragOver(false);

      const files = Array.from(e.dataTransfer.files);
      if (files.length > 0) {
        const paths = files.map((f) => (f as File & { path?: string }).path ?? f.name);
        onFilesDropped(paths);
      }
    },
    [onFilesDropped]
  );

  const handleClick = useCallback(async () => {
    try {
      const selected = await open({
        multiple,
        title: "Select files to convert",
      });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        if (paths.length > 0) {
          onFilesDropped(paths);
        }
      }
    } catch {
      // user cancelled
    }
  }, [multiple, onFilesDropped]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        handleClick();
      }
    },
    [handleClick]
  );

  return (
    <div
      role="button"
      tabIndex={0}
      aria-label="Drop files here or click to browse"
      className={clsx(
        compact ? "dropzone !p-6" : "dropzone",
        isDragOver && "dropzone-active",
        "cursor-pointer",
        className
      )}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
    >
      {children ?? (
        <div className="flex flex-col items-center gap-3">
          <Upload
            size={compact ? 32 : 48}
            className="text-surface-400 dark:text-surface-500"
          />
          <p
            className={clsx(
              "font-medium text-surface-500 dark:text-surface-400",
              compact ? "text-sm" : "text-lg"
            )}
          >
            {t("home.dropzone")}
          </p>
          <p className="text-xs text-surface-400 dark:text-surface-500">
            or click to browse
          </p>
        </div>
      )}
    </div>
  );
}

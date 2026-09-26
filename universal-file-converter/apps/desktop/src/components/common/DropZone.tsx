import { useState, useCallback, type DragEvent, type ReactNode } from "react";
import { clsx } from "clsx";
import { Upload } from "lucide-react";
import { t } from "@/i18n";

interface DropZoneProps {
  onFilesDropped: (paths: string[]) => void;
  children?: ReactNode;
  className?: string;
  compact?: boolean;
}

export function DropZone({
  onFilesDropped,
  children,
  className,
  compact = false,
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

  return (
    <div
      className={clsx(
        compact ? "dropzone !p-6" : "dropzone",
        isDragOver && "dropzone-active",
        className
      )}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
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
        </div>
      )}
    </div>
  );
}

import {
  FileText,
  Table,
  Image,
  Code,
  Globe,
  Presentation,
  X,
  AlertTriangle,
} from "lucide-react";
import type { DetectedFile } from "@/types/conversion";
import { formatFileSize } from "@/utils/fileSize";
import { truncateFilename } from "@/utils/formatters";
import { getFormatById } from "@/types/formats";
import { t } from "@/i18n";

interface FileCardProps {
  file: DetectedFile;
  onRemove?: () => void;
}

const CATEGORY_ICONS: Record<string, typeof FileText> = {
  documents: FileText,
  spreadsheets: Table,
  presentations: Presentation,
  pdf: FileText,
  images: Image,
  text: FileText,
  web: Globe,
  code: Code,
  data: Table,
};

export function FileCard({ file, onRemove }: FileCardProps) {
  const formatInfo = getFormatById(file.detectedFormat);
  const Icon = CATEGORY_ICONS[formatInfo?.category ?? ""] ?? FileText;

  return (
    <div className="card p-4">
      <div className="flex items-start gap-3">
        <div className="p-2.5 rounded-lg bg-primary-50 dark:bg-primary-950/30">
          <Icon size={22} className="text-primary-600 dark:text-primary-400" />
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-medium text-sm text-surface-900 dark:text-surface-100 truncate">
            {truncateFilename(file.name)}
          </p>
          <div className="flex items-center gap-2 mt-1">
            <span className="text-xs text-surface-500 dark:text-surface-400 uppercase font-medium">
              {formatInfo?.name ?? file.detectedFormat}
            </span>
            <span className="text-xs text-surface-400 dark:text-surface-500">
              {formatFileSize(file.sizeBytes)}
            </span>
            {file.metadata.pageCount && (
              <span className="text-xs text-surface-400 dark:text-surface-500">
                {file.metadata.pageCount} {t("file.pages").toLowerCase()}
              </span>
            )}
            {file.metadata.imageWidth && file.metadata.imageHeight && (
              <span className="text-xs text-surface-400 dark:text-surface-500">
                {file.metadata.imageWidth} x {file.metadata.imageHeight}
              </span>
            )}
          </div>
          {file.formatMismatch && (
            <div className="flex items-center gap-1.5 mt-2 text-amber-600 dark:text-amber-400">
              <AlertTriangle size={14} />
              <span className="text-xs">{t("file.mismatchWarning")}</span>
            </div>
          )}
        </div>
        {onRemove && (
          <button
            onClick={onRemove}
            className="p-1 hover:bg-surface-100 dark:hover:bg-surface-800 rounded"
            aria-label={t("file.remove")}
          >
            <X size={16} className="text-surface-400" />
          </button>
        )}
      </div>
    </div>
  );
}

import { clsx } from "clsx";
import type { JobStatus } from "@/types/conversion";
import { t } from "@/i18n";

interface StatusBadgeProps {
  status: JobStatus;
}

const STATUS_STYLES: Record<JobStatus, string> = {
  queued: "badge-info",
  detecting: "badge-info",
  planning: "badge-info",
  preparing: "badge-info",
  converting: "badge-warning",
  validating: "badge-warning",
  completed: "badge-success",
  failed: "badge-error",
  cancelled: "badge-error",
};

export function StatusBadge({ status }: StatusBadgeProps) {
  return (
    <span className={clsx("badge", STATUS_STYLES[status])}>
      {t(`status.${status}`)}
    </span>
  );
}

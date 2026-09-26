import { clsx } from "clsx";

interface ProgressBarProps {
  progress: number;
  indeterminate?: boolean;
  size?: "sm" | "md" | "lg";
  color?: "primary" | "success" | "error";
  label?: string;
  showPercentage?: boolean;
}

const COLORS = {
  primary: "bg-primary-500",
  success: "bg-emerald-500",
  error: "bg-red-500",
};

const SIZES = {
  sm: "h-1",
  md: "h-2",
  lg: "h-3",
};

export function ProgressBar({
  progress,
  indeterminate = false,
  size = "md",
  color = "primary",
  label,
  showPercentage = false,
}: ProgressBarProps) {
  const clamped = Math.min(100, Math.max(0, progress));

  return (
    <div className="w-full">
      {(label || showPercentage) && (
        <div className="flex justify-between mb-1">
          {label && (
            <span className="text-xs text-surface-600 dark:text-surface-400">
              {label}
            </span>
          )}
          {showPercentage && !indeterminate && (
            <span className="text-xs font-medium text-surface-700 dark:text-surface-300">
              {Math.round(clamped)}%
            </span>
          )}
        </div>
      )}
      <div
        className={clsx(
          "w-full rounded-full bg-surface-200 dark:bg-surface-700 overflow-hidden",
          SIZES[size]
        )}
      >
        <div
          className={clsx(
            "h-full rounded-full transition-all duration-300 ease-out",
            COLORS[color],
            indeterminate && "animate-pulse w-full"
          )}
          style={indeterminate ? undefined : { width: `${clamped}%` }}
        />
      </div>
    </div>
  );
}

import { useCallback } from "react";
import { useBatchStore } from "@/stores/batchStore";

export function useBatchQueue() {
  const { jobs, isProcessing, isPaused, progress } = useBatchStore();

  const completedCount = jobs.filter((j) => j.status === "completed").length;
  const failedCount = jobs.filter((j) => j.status === "failed").length;
  const pendingCount = jobs.filter(
    (j) => j.status === "queued" || j.status === "converting"
  ).length;

  const canStart = jobs.length > 0 && !isProcessing;
  const canPause = isProcessing && !isPaused;
  const canResume = isProcessing && isPaused;

  const getStats = useCallback(() => {
    return {
      total: jobs.length,
      completed: completedCount,
      failed: failedCount,
      pending: pendingCount,
      overallProgress:
        jobs.length > 0
          ? Math.round((completedCount / jobs.length) * 100)
          : 0,
    };
  }, [jobs.length, completedCount, failedCount, pendingCount]);

  return {
    jobs,
    isProcessing,
    isPaused,
    progress,
    canStart,
    canPause,
    canResume,
    stats: getStats(),
  };
}

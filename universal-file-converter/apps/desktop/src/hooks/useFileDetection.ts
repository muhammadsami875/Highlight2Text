import { useState, useCallback } from "react";
import { detectFile, getSupportedOutputs } from "@/services/ipc";
import { getOutputFormats } from "@/services/formatMatrix";
import { useConversionStore } from "@/stores/conversionStore";

export function useFileDetection() {
  const [isDetecting, setIsDetecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { setInputFile, setOutputFormats } = useConversionStore();

  const detect = useCallback(
    async (filePath: string) => {
      setIsDetecting(true);
      setError(null);
      try {
        const detected = await detectFile(filePath);
        setInputFile(detected);

        const clientFormats = getOutputFormats(detected.detectedFormat);

        try {
          const backendRoutes = await getSupportedOutputs(detected.detectedFormat);
          const backendFormats = new Set(backendRoutes.map((r) => r.format));

          const merged = clientFormats.map((f) => ({
            ...f,
            level: backendFormats.has(f.format) ? f.level : ("experimental" as const),
          }));

          for (const route of backendRoutes) {
            if (!merged.some((m) => m.format === route.format)) {
              merged.push({ format: route.format, level: route.level });
            }
          }

          setOutputFormats(merged);
        } catch {
          setOutputFormats(clientFormats);
        }

        return detected;
      } catch (err) {
        const message =
          err instanceof Error ? err.message : "Failed to detect file format";
        setError(message);
        return null;
      } finally {
        setIsDetecting(false);
      }
    },
    [setInputFile, setOutputFormats]
  );

  return { detect, isDetecting, error };
}

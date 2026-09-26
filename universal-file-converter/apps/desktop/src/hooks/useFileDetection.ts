import { useState, useCallback } from "react";
import { detectFile } from "@/services/ipc";
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
        const outputs = getOutputFormats(detected.detectedFormat);
        setOutputFormats(outputs);
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

import { useEffect, useState } from "react";
import { ArrowRight, Zap } from "lucide-react";
import { getConversionPlan } from "@/services/ipc";
import type { ConversionPlan } from "@/types/conversion";

interface Props {
  inputFormat: string;
  outputFormat: string;
}

export function ConversionPlanPreview({ inputFormat, outputFormat }: Props) {
  const [plan, setPlan] = useState<ConversionPlan | null>(null);

  useEffect(() => {
    if (inputFormat && outputFormat && inputFormat !== outputFormat) {
      getConversionPlan(inputFormat, outputFormat)
        .then(setPlan)
        .catch(() => setPlan(null));
    } else {
      setPlan(null);
    }
  }, [inputFormat, outputFormat]);

  if (!plan || plan.steps.length === 0) return null;

  return (
    <div className="flex items-center gap-1.5 flex-wrap text-xs text-surface-500 dark:text-surface-400">
      <Zap size={12} className="text-primary-500 shrink-0" />
      {plan.steps.map((step, i) => (
        <span key={i} className="flex items-center gap-1">
          {i === 0 && (
            <span className="font-medium text-surface-700 dark:text-surface-300 uppercase">
              {step.from}
            </span>
          )}
          <ArrowRight size={10} className="text-surface-400" />
          <span className="font-medium text-surface-700 dark:text-surface-300 uppercase">
            {step.to}
          </span>
          <span className="text-surface-400">({step.engineName})</span>
        </span>
      ))}
      {plan.steps.length > 1 && (
        <span className="text-amber-500 dark:text-amber-400 ml-1">
          {plan.steps.length}-step conversion
        </span>
      )}
    </div>
  );
}

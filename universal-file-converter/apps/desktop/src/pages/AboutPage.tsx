import { useEffect, useState } from "react";
import { Info, CheckCircle2, XCircle } from "lucide-react";
import { getEngineStatus } from "@/services/ipc";
import type { EngineInfo } from "@/types/conversion";
import { t } from "@/i18n";

export function AboutPage() {
  const [engines, setEngines] = useState<EngineInfo[]>([]);

  useEffect(() => {
    getEngineStatus()
      .then(setEngines)
      .catch(() => {});
  }, []);

  return (
    <div className="space-y-6 max-w-2xl">
      <div className="flex items-start gap-4">
        <div className="p-3 rounded-xl bg-primary-50 dark:bg-primary-950/30">
          <Info size={28} className="text-primary-600 dark:text-primary-400" />
        </div>
        <div>
          <h2 className="text-xl font-bold text-surface-900 dark:text-surface-100">
            {t("about.title")}
          </h2>
          <p className="text-sm text-surface-500 dark:text-surface-400 mt-1">
            {t("about.description")}
          </p>
          <p className="text-xs text-surface-400 dark:text-surface-500 mt-2">
            {t("about.version")} {t("app.version")}
          </p>
        </div>
      </div>

      <section className="card p-5">
        <h3 className="text-sm font-semibold text-surface-700 dark:text-surface-300 mb-4">
          {t("about.engines")}
        </h3>
        <div className="space-y-3">
          {engines.map((engine) => (
            <div
              key={engine.id}
              className="flex items-center justify-between py-2 border-b border-surface-100 dark:border-surface-800 last:border-0"
            >
              <div>
                <p className="text-sm font-medium text-surface-800 dark:text-surface-200">
                  {engine.name}
                </p>
                <p className="text-xs text-surface-400 dark:text-surface-500">
                  {engine.inputFormats.join(", ")} &rarr;{" "}
                  {engine.outputFormats.join(", ")}
                </p>
              </div>
              <div className="flex items-center gap-3">
                <span className="text-xs text-surface-400 dark:text-surface-500">
                  {engine.license}
                </span>
                <div className="flex items-center gap-1">
                  {engine.available ? (
                    <>
                      <CheckCircle2 size={14} className="text-emerald-500" />
                      <span className="text-xs text-emerald-600 dark:text-emerald-400">
                        {t("about.available")}
                      </span>
                    </>
                  ) : (
                    <>
                      <XCircle size={14} className="text-surface-400" />
                      <span className="text-xs text-surface-400 dark:text-surface-500">
                        Not installed
                      </span>
                    </>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      </section>

      <section className="card p-5">
        <h3 className="text-sm font-semibold text-surface-700 dark:text-surface-300 mb-2">
          {t("about.licenses")}
        </h3>
        <p className="text-sm text-surface-500 dark:text-surface-400">
          This application uses open-source libraries and tools. See the
          THIRD-PARTY-NOTICES file for complete license information.
        </p>
      </section>

      <p className="text-xs text-surface-300 dark:text-surface-600 text-center">
        Built with Tauri, React, TypeScript, and Rust
      </p>
    </div>
  );
}

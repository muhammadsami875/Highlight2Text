import { Info, CheckCircle2 } from "lucide-react";
import { t } from "@/i18n";

const ENGINES = [
  { name: "LibreOffice", desc: "Office document conversions", license: "MPL 2.0 / LGPL 3" },
  { name: "Pandoc", desc: "Structured text/document conversions", license: "GPL 2+" },
  { name: "Poppler", desc: "PDF rendering and extraction", license: "GPL 2+" },
  { name: "wkhtmltopdf", desc: "HTML to PDF rendering", license: "LGPL 3" },
  { name: "Tesseract", desc: "Optical character recognition", license: "Apache 2.0" },
  { name: "FFmpeg", desc: "Audio/video conversions", license: "LGPL 2.1+" },
  { name: "image (Rust)", desc: "Image format conversions", license: "MIT / Apache-2.0" },
  { name: "syntect (Rust)", desc: "Syntax highlighting", license: "MIT" },
  { name: "lopdf (Rust)", desc: "PDF text extraction", license: "MIT" },
];

export function AboutPage() {
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
          {ENGINES.map((engine) => (
            <div
              key={engine.name}
              className="flex items-center justify-between py-2 border-b border-surface-100 dark:border-surface-800 last:border-0"
            >
              <div>
                <p className="text-sm font-medium text-surface-800 dark:text-surface-200">
                  {engine.name}
                </p>
                <p className="text-xs text-surface-400 dark:text-surface-500">
                  {engine.desc}
                </p>
              </div>
              <div className="flex items-center gap-3">
                <span className="text-xs text-surface-400 dark:text-surface-500">
                  {engine.license}
                </span>
                <div className="flex items-center gap-1">
                  <CheckCircle2
                    size={14}
                    className="text-emerald-500"
                  />
                  <span className="text-xs text-emerald-600 dark:text-emerald-400">
                    {t("about.available")}
                  </span>
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

import { useNavigate } from "react-router-dom";
import {
  FileText,
  Table,
  Image,
  Globe,
  ArrowRight,
} from "lucide-react";
import { DropZone } from "@/components/common/DropZone";
import { useFileDetection } from "@/hooks/useFileDetection";
import { useHistoryStore } from "@/stores/historyStore";
import { t } from "@/i18n";
import { getFormatById } from "@/types/formats";

const POPULAR_CONVERSIONS = [
  { from: "pdf", to: "docx", icon: FileText, label: "PDF to Word" },
  { from: "pdf", to: "xlsx", icon: Table, label: "PDF to Excel" },
  { from: "docx", to: "pdf", icon: FileText, label: "Word to PDF" },
  { from: "xlsx", to: "pdf", icon: Table, label: "Excel to PDF" },
  { from: "pptx", to: "pdf", icon: FileText, label: "PPT to PDF" },
  { from: "png", to: "pdf", icon: Image, label: "Image to PDF" },
  { from: "html", to: "pdf", icon: Globe, label: "HTML to PDF" },
  { from: "md", to: "pdf", icon: FileText, label: "Markdown to PDF" },
];

export function HomePage() {
  const navigate = useNavigate();
  const { detect } = useFileDetection();
  const { recentPairs } = useHistoryStore();

  const handleFilesDropped = async (paths: string[]) => {
    if (paths.length > 0) {
      await detect(paths[0]);
      navigate("/convert");
    }
  };

  const handleChooseFile = () => {
    navigate("/convert");
  };

  return (
    <div className="space-y-8">
      <div>
        <h2 className="text-2xl font-bold text-surface-900 dark:text-surface-100 mb-1">
          {t("app.name")}
        </h2>
        <p className="text-sm text-surface-500 dark:text-surface-400">
          Convert documents, images, spreadsheets, and more
        </p>
      </div>

      <DropZone onFilesDropped={handleFilesDropped} />

      <div className="flex gap-3">
        <button onClick={handleChooseFile} className="btn-primary">
          {t("home.chooseFile")}
        </button>
        <button
          onClick={() => navigate("/batch")}
          className="btn-secondary"
        >
          {t("home.chooseFolder")}
        </button>
      </div>

      <div>
        <h3 className="text-sm font-semibold text-surface-700 dark:text-surface-300 mb-3">
          {t("home.popularConversions")}
        </h3>
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
          {POPULAR_CONVERSIONS.map((conv) => (
            <button
              key={`${conv.from}-${conv.to}`}
              onClick={() => navigate("/convert")}
              className="card p-3 hover:shadow-md transition-shadow text-left group"
            >
              <div className="flex items-center gap-2 mb-1.5">
                <conv.icon
                  size={16}
                  className="text-primary-500 dark:text-primary-400"
                />
                <ArrowRight
                  size={12}
                  className="text-surface-300 dark:text-surface-600 group-hover:text-primary-400 transition-colors"
                />
              </div>
              <p className="text-xs font-medium text-surface-700 dark:text-surface-300">
                {conv.label}
              </p>
            </button>
          ))}
        </div>
      </div>

      {recentPairs.length > 0 && (
        <div>
          <h3 className="text-sm font-semibold text-surface-700 dark:text-surface-300 mb-3">
            {t("home.recentConversions")}
          </h3>
          <div className="flex flex-wrap gap-2">
            {recentPairs.map((pair, i) => {
              const fromInfo = getFormatById(pair.from);
              const toInfo = getFormatById(pair.to);
              return (
                <button
                  key={i}
                  onClick={() => navigate("/convert")}
                  className="badge badge-info cursor-pointer hover:opacity-80"
                >
                  {fromInfo?.name ?? pair.from.toUpperCase()} &rarr;{" "}
                  {toInfo?.name ?? pair.to.toUpperCase()}
                </button>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}

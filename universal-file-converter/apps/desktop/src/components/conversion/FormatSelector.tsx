import { useState, useMemo } from "react";
import { clsx } from "clsx";
import { Search, Check, AlertCircle, FlaskConical } from "lucide-react";
import { getFormatById, FORMAT_CATEGORIES, type FormatCategory } from "@/types/formats";
import type { SupportLevel } from "@/types/formats";
import { t } from "@/i18n";

interface FormatOption {
  format: string;
  level: SupportLevel;
  notes?: string;
}

interface FormatSelectorProps {
  formats: FormatOption[];
  selected: string | null;
  onSelect: (format: string) => void;
}

export function FormatSelector({
  formats,
  selected,
  onSelect,
}: FormatSelectorProps) {
  const [search, setSearch] = useState("");

  const grouped = useMemo(() => {
    const groups: Record<string, FormatOption[]> = {};
    for (const opt of formats) {
      const info = getFormatById(opt.format);
      const category = info?.category ?? "data";
      if (!groups[category]) groups[category] = [];

      const matchesSearch =
        !search ||
        opt.format.toLowerCase().includes(search.toLowerCase()) ||
        (info?.name ?? "").toLowerCase().includes(search.toLowerCase()) ||
        (info?.description ?? "").toLowerCase().includes(search.toLowerCase());

      if (matchesSearch) {
        groups[category].push(opt);
      }
    }
    return Object.entries(groups)
      .filter(([, items]) => items.length > 0)
      .sort(([a], [b]) => a.localeCompare(b));
  }, [formats, search]);

  if (formats.length === 0) {
    return (
      <div className="card p-6 text-center">
        <AlertCircle
          size={24}
          className="mx-auto mb-2 text-surface-400 dark:text-surface-500"
        />
        <p className="text-sm text-surface-500 dark:text-surface-400">
          {t("convert.selectFormat")}
        </p>
      </div>
    );
  }

  return (
    <div className="card overflow-hidden">
      <div className="p-3 border-b border-surface-200 dark:border-surface-700">
        <div className="relative">
          <Search
            size={16}
            className="absolute left-3 top-1/2 -translate-y-1/2 text-surface-400"
          />
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder={t("convert.searchFormats")}
            className="input-field pl-9 py-1.5 text-sm"
          />
        </div>
      </div>
      <div className="max-h-64 overflow-y-auto p-2">
        {grouped.map(([category, items]) => (
          <div key={category} className="mb-2 last:mb-0">
            <p className="px-2 py-1 text-xs font-semibold text-surface-400 dark:text-surface-500 uppercase tracking-wider">
              {FORMAT_CATEGORIES[category as FormatCategory] ?? category}
            </p>
            {items.map((opt) => {
              const info = getFormatById(opt.format);
              const isSelected = selected === opt.format;
              return (
                <button
                  key={opt.format}
                  onClick={() => onSelect(opt.format)}
                  className={clsx(
                    "w-full flex items-center gap-2 px-3 py-2 rounded-lg text-left text-sm transition-colors",
                    isSelected
                      ? "bg-primary-50 dark:bg-primary-950/30 text-primary-700 dark:text-primary-400"
                      : "hover:bg-surface-100 dark:hover:bg-surface-800 text-surface-700 dark:text-surface-300"
                  )}
                >
                  <span className="font-medium flex-1">
                    {info?.name ?? opt.format.toUpperCase()}
                  </span>
                  {opt.level === "experimental" && (
                    <FlaskConical
                      size={14}
                      className="text-amber-500 dark:text-amber-400"
                    />
                  )}
                  {isSelected && (
                    <Check size={16} className="text-primary-600 dark:text-primary-400" />
                  )}
                </button>
              );
            })}
          </div>
        ))}
        {grouped.length === 0 && (
          <p className="text-center text-sm text-surface-400 py-4">
            No matching formats
          </p>
        )}
      </div>
    </div>
  );
}

import { Save } from "lucide-react";
import { useSettings } from "@/hooks/useSettings";
import { useToastStore } from "@/stores/toastStore";
import { t } from "@/i18n";

export function SettingsPage() {
  const { settings, isDirty, updateSettings, save, resetToDefaults } =
    useSettings();
  const { addToast } = useToastStore();

  const handleSave = async () => {
    try {
      await save();
      addToast("success", "Settings saved");
    } catch {
      addToast("error", "Failed to save settings");
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-bold text-surface-900 dark:text-surface-100">
          {t("settings.title")}
        </h2>
        <div className="flex gap-2">
          <button onClick={resetToDefaults} className="btn-ghost text-sm">
            Reset to Defaults
          </button>
          <button
            onClick={handleSave}
            disabled={!isDirty}
            className="btn-primary text-sm"
          >
            <Save size={14} className="mr-1.5 inline" />
            Save
          </button>
        </div>
      </div>

      <div className="space-y-6">
        <section className="card p-5 space-y-4">
          <h3 className="text-sm font-semibold text-surface-700 dark:text-surface-300">
            {t("settings.general")}
          </h3>
          <div>
            <label className="block text-xs font-medium text-surface-600 dark:text-surface-400 mb-1">
              Theme
            </label>
            <select
              value={settings.general.theme}
              onChange={(e) => updateSettings("general.theme", e.target.value)}
              className="input-field text-sm"
            >
              <option value="system">System</option>
              <option value="light">Light</option>
              <option value="dark">Dark</option>
            </select>
          </div>
          <div>
            <label className="block text-xs font-medium text-surface-600 dark:text-surface-400 mb-1">
              Language
            </label>
            <select
              value={settings.general.language}
              onChange={(e) =>
                updateSettings("general.language", e.target.value)
              }
              className="input-field text-sm"
            >
              <option value="en">English</option>
            </select>
          </div>
        </section>

        <section className="card p-5 space-y-4">
          <h3 className="text-sm font-semibold text-surface-700 dark:text-surface-300">
            {t("settings.conversion")}
          </h3>
          <div>
            <label className="block text-xs font-medium text-surface-600 dark:text-surface-400 mb-1">
              Output Directory
            </label>
            <select
              value={settings.conversion.outputDirMode}
              onChange={(e) =>
                updateSettings("conversion.outputDirMode", e.target.value)
              }
              className="input-field text-sm"
            >
              <option value="same">Same as source file</option>
              <option value="custom">Custom folder</option>
              <option value="ask">Ask every time</option>
            </select>
          </div>
          <div>
            <label className="block text-xs font-medium text-surface-600 dark:text-surface-400 mb-1">
              File Exists Behavior
            </label>
            <select
              value={settings.conversion.overwriteMode}
              onChange={(e) =>
                updateSettings("conversion.overwriteMode", e.target.value)
              }
              className="input-field text-sm"
            >
              <option value="rename">Add number suffix</option>
              <option value="overwrite">Overwrite</option>
              <option value="ask">Ask every time</option>
            </select>
          </div>
          <div>
            <label className="block text-xs font-medium text-surface-600 dark:text-surface-400 mb-1">
              File Suffix
            </label>
            <input
              type="text"
              value={settings.conversion.fileSuffix}
              onChange={(e) =>
                updateSettings("conversion.fileSuffix", e.target.value)
              }
              className="input-field text-sm"
              placeholder="-converted"
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-surface-600 dark:text-surface-400 mb-1">
              Parallel Conversions
            </label>
            <select
              value={settings.conversion.maxParallelJobs}
              onChange={(e) =>
                updateSettings(
                  "conversion.maxParallelJobs",
                  parseInt(e.target.value)
                )
              }
              className="input-field text-sm"
            >
              <option value={1}>1</option>
              <option value={2}>2</option>
              <option value={4}>4</option>
            </select>
          </div>
        </section>

        <section className="card p-5 space-y-4">
          <h3 className="text-sm font-semibold text-surface-700 dark:text-surface-300">
            {t("settings.pdf")}
          </h3>
          <div>
            <label className="block text-xs font-medium text-surface-600 dark:text-surface-400 mb-1">
              Default DPI
            </label>
            <select
              value={settings.pdf.defaultDpi}
              onChange={(e) =>
                updateSettings("pdf.defaultDpi", parseInt(e.target.value))
              }
              className="input-field text-sm"
            >
              <option value={72}>72</option>
              <option value={150}>150</option>
              <option value={300}>300</option>
              <option value={600}>600</option>
            </select>
          </div>
          <div>
            <label className="block text-xs font-medium text-surface-600 dark:text-surface-400 mb-1">
              Default Page Size
            </label>
            <select
              value={settings.pdf.defaultPageSize}
              onChange={(e) =>
                updateSettings("pdf.defaultPageSize", e.target.value)
              }
              className="input-field text-sm"
            >
              <option value="A4">A4</option>
              <option value="Letter">Letter</option>
              <option value="Legal">Legal</option>
            </select>
          </div>
        </section>

        <section className="card p-5 space-y-4">
          <h3 className="text-sm font-semibold text-surface-700 dark:text-surface-300">
            {t("settings.advanced")}
          </h3>
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={settings.advanced.enableExperimental}
              onChange={(e) =>
                updateSettings("advanced.enableExperimental", e.target.checked)
              }
              className="rounded border-surface-300 dark:border-surface-600"
            />
            <span className="text-sm text-surface-700 dark:text-surface-300">
              Enable experimental converters
            </span>
          </label>
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={settings.advanced.enableDiagnosticLogs}
              onChange={(e) =>
                updateSettings(
                  "advanced.enableDiagnosticLogs",
                  e.target.checked
                )
              }
              className="rounded border-surface-300 dark:border-surface-600"
            />
            <span className="text-sm text-surface-700 dark:text-surface-300">
              Enable diagnostic logging
            </span>
          </label>
          <div>
            <label className="block text-xs font-medium text-surface-600 dark:text-surface-400 mb-1">
              Max File Size (MB)
            </label>
            <input
              type="number"
              value={settings.advanced.maxFileSizeMb}
              onChange={(e) =>
                updateSettings(
                  "advanced.maxFileSizeMb",
                  parseInt(e.target.value) || 2048
                )
              }
              min={1}
              max={10240}
              className="input-field text-sm"
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-surface-600 dark:text-surface-400 mb-1">
              Process Timeout (seconds)
            </label>
            <input
              type="number"
              value={settings.advanced.processTimeoutSec}
              onChange={(e) =>
                updateSettings(
                  "advanced.processTimeoutSec",
                  parseInt(e.target.value) || 300
                )
              }
              min={30}
              max={3600}
              className="input-field text-sm"
            />
          </div>
        </section>
      </div>
    </div>
  );
}

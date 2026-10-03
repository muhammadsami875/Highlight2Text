import { useSettings } from "../stores/settingsStore";

export default function Settings() {
  const { theme, setTheme } = useSettings();
  return (
    <section className="p-8 max-w-2xl space-y-6">
      <h1 className="text-xl font-semibold">Settings</h1>
      <div>
        <div className="text-sm font-medium mb-2">Appearance</div>
        <div className="flex gap-2">
          {(["system", "light", "dark"] as const).map((t) => (
            <button
              key={t}
              onClick={() => setTheme(t)}
              className={`px-3 py-1.5 rounded-md border text-sm capitalize ${
                theme === t
                  ? "border-brand-500 text-brand-500"
                  : "border-neutral-800 text-neutral-300 hover:bg-neutral-800"
              }`}
            >
              {t}
            </button>
          ))}
        </div>
      </div>
      <p className="text-xs text-neutral-500">
        OCR, image, PDF, storage, shortcuts, privacy, and advanced panes are added
        in later phases.
      </p>
    </section>
  );
}

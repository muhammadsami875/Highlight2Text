import { useEffect, useState } from "react";
import { useSettings } from "../stores/settingsStore";
import { cacheSummary, clearCache, type CacheSummary } from "../services/ipc";

function formatBytes(n: number) {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 * 1024 * 1024) return `${(n / 1024 / 1024).toFixed(1)} MB`;
  return `${(n / 1024 / 1024 / 1024).toFixed(2)} GB`;
}

export default function Settings() {
  const { theme, setTheme } = useSettings();
  const [cache, setCache] = useState<CacheSummary | null>(null);

  async function refresh() { try { setCache(await cacheSummary()); } catch { /* ignore */ } }
  useEffect(() => { refresh(); }, []);

  return (
    <section className="p-8 max-w-2xl space-y-8">
      <h1 className="text-xl font-semibold">Settings</h1>

      <Group title="Appearance">
        <div className="flex gap-2">
          {(["system", "light", "dark"] as const).map((t) => (
            <button key={t} onClick={() => setTheme(t)}
              className={`px-3 py-1.5 rounded-md border text-sm capitalize ${
                theme === t ? "border-brand-500 text-brand-500" : "border-neutral-800 text-neutral-300 hover:bg-neutral-800"
              }`}>
              {t}
            </button>
          ))}
        </div>
      </Group>

      <Group title="Privacy">
        <ul className="text-sm text-neutral-300 space-y-1.5 list-disc pl-5">
          <li>All processing is on this device. No images, text, or filenames leave the machine.</li>
          <li>OCR runs via a local Tesseract sidecar; language packs live under the app's resources dir.</li>
          <li>The webcam preview only starts after you press <em>Start camera</em>.</li>
          <li>No telemetry, no network calls by default.</li>
        </ul>
      </Group>

      <Group title="Storage & cache">
        {cache ? (
          <div className="text-sm text-neutral-300 space-y-1">
            <div>Cache: <span className="font-mono text-xs">{cache.path}</span></div>
            <div>{formatBytes(cache.bytes)} across {cache.files} files</div>
          </div>
        ) : (
          <div className="text-sm text-neutral-500">Reading cache…</div>
        )}
        <div className="mt-3 flex gap-2">
          <button onClick={refresh} className="text-sm px-3 py-1.5 rounded border border-neutral-700">Refresh</button>
          <button
            onClick={async () => { await clearCache(); await refresh(); }}
            className="text-sm px-3 py-1.5 rounded border border-red-700 text-red-300 hover:bg-red-900/20"
          >
            Clear cache
          </button>
        </div>
      </Group>

      <Group title="About">
        <div className="text-sm text-neutral-400">
          DocSnap v0.1.0 · offline-first document scanning + OCR. See the
          <code> docs/ </code>folder for architecture and phase notes.
        </div>
      </Group>
    </section>
  );
}

function Group({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <h2 className="text-sm font-medium mb-3">{title}</h2>
      {children}
    </div>
  );
}

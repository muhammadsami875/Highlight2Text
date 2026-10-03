import { useEffect, useState } from "react";
import { getAppInfo, type AppInfo } from "../services/ipc";

export default function Ocr() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    getAppInfo()
      .then(setInfo)
      .catch((e) => setErr(String(e)));
  }, []);

  return (
    <section className="p-8 max-w-3xl">
      <h1 className="text-xl font-semibold mb-2">OCR</h1>
      <p className="text-neutral-400 text-sm mb-6">
        Tesseract sidecar wiring arrives in Phase 8. Smoke-test of the Tauri IPC
        bridge below.
      </p>
      <div className="rounded-lg border border-neutral-800 bg-neutral-900 p-4 text-sm">
        {err && <div className="text-red-400">IPC error: {err}</div>}
        {info ? (
          <pre className="whitespace-pre-wrap text-neutral-300">
{JSON.stringify(info, null, 2)}
          </pre>
        ) : !err ? (
          <span className="text-neutral-500">Loading app info…</span>
        ) : null}
      </div>
    </section>
  );
}

import { useMemo, useRef, useState } from "react";
import { useProject } from "../stores/projectStore";
import { assetUrl, runOcr } from "../services/ipc";
import type { OcrOptions, PageMode } from "../types/bindings";

const LANGS: { code: string; label: string }[] = [
  { code: "eng", label: "English" },
  { code: "ara", label: "Arabic" },
  { code: "urd", label: "Urdu" },
  { code: "hin", label: "Hindi" },
  { code: "fra", label: "French" },
  { code: "deu", label: "German" },
  { code: "spa", label: "Spanish" },
  { code: "ita", label: "Italian" },
  { code: "por", label: "Portuguese" },
  { code: "chi_sim", label: "Chinese (Simplified)" },
  { code: "jpn", label: "Japanese" },
  { code: "kor", label: "Korean" },
];
const MODES: PageMode[] = ["Auto","SingleBlock","MultiBlock","SingleLine","SparseText","Table"];

export default function Ocr() {
  const { pages, activeId, warps, enhancements, ocrResults, setActive, setOcr, updateOcrText } =
    useProject();
  const [langs, setLangs] = useState<string[]>(["eng"]);
  const [mode, setMode] = useState<PageMode>("Auto");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const textRef = useRef<HTMLTextAreaElement | null>(null);

  const active = pages.find((p) => p.id === activeId) ?? null;
  const ocr = active ? ocrResults[active.id] ?? null : null;

  function imageForOcr(): string | null {
    if (!active) return null;
    return enhancements[active.id]?.rendered.preview_path
      ?? warps[active.id]?.warped.preview_path
      ?? active.source_path;
  }

  async function run() {
    const src = imageForOcr();
    if (!active || !src) return;
    const opts: OcrOptions = { languages: langs.join("+"), mode };
    setBusy(true); setErr(null);
    try {
      const res = await runOcr(src, opts);
      setOcr(active.id, res);
    } catch (e) {
      setErr(String((e as { message?: unknown })?.message ?? e));
    } finally {
      setBusy(false);
    }
  }

  const avgConf = useMemo(() => {
    if (!ocr?.words?.length) return null;
    const sum = ocr.words.reduce((a, w) => a + w.confidence, 0);
    return sum / ocr.words.length;
  }, [ocr]);

  const highlights = useMemo(() => {
    if (!query.trim() || !ocr) return [];
    const q = query.toLowerCase();
    return ocr.words.filter((w) => w.text.toLowerCase().includes(q));
  }, [query, ocr]);

  function toggleLang(code: string) {
    setLangs((xs) => xs.includes(code) ? xs.filter((x) => x !== code) : [...xs, code]);
  }

  return (
    <section className="grid grid-cols-[220px_1fr_1fr] h-full">
      <aside className="border-r border-neutral-800 bg-neutral-900 p-3 overflow-y-auto">
        <div className="text-xs uppercase tracking-wide text-neutral-500 mb-2">Pages</div>
        {pages.length === 0 && <p className="text-xs text-neutral-500">Import pages in the Editor first.</p>}
        <ul className="space-y-1">
          {pages.map((p, i) => (
            <li key={p.id}>
              <button
                onClick={() => setActive(p.id)}
                className={`w-full text-left text-xs rounded px-2 py-1 ${
                  activeId === p.id ? "bg-brand-500/15 text-brand-500" : "text-neutral-300 hover:bg-neutral-800"
                }`}
              >
                Page {i + 1} · {ocrResults[p.id] ? "OCR ✓" : "—"}
              </button>
            </li>
          ))}
        </ul>
      </aside>

      <div className="relative flex items-center justify-center bg-neutral-950 overflow-hidden">
        {active ? (
          <div className="relative inline-block max-w-full max-h-full">
            <img
              src={assetUrl(imageForOcr()!)}
              alt="ocr source"
              className="max-w-full max-h-full object-contain block select-none"
            />
            {ocr && (
              <svg
                viewBox={`0 0 ${active.width} ${active.height}`}
                preserveAspectRatio="xMidYMid meet"
                className="absolute inset-0 w-full h-full pointer-events-none"
              >
                {ocr.words.map((w, i) => {
                  const isHit = highlights.includes(w);
                  const low = w.confidence > 0 && w.confidence < 0.6;
                  const stroke = isHit ? "#f59e0b" : low ? "#ef4444" : "#2b6cff";
                  return (
                    <rect
                      key={i}
                      x={w.bbox[0]} y={w.bbox[1]}
                      width={w.bbox[2]} height={w.bbox[3]}
                      fill="transparent" stroke={stroke}
                      strokeWidth={Math.max(1, Math.min(active.width, active.height) * 0.001)}
                    />
                  );
                })}
              </svg>
            )}
          </div>
        ) : (
          <div className="text-neutral-500 text-sm">Select a page.</div>
        )}
      </div>

      <aside className="border-l border-neutral-800 bg-neutral-900 flex flex-col">
        <div className="p-3 border-b border-neutral-800 space-y-2">
          <div className="flex gap-2">
            <select
              value={mode}
              onChange={(e) => setMode(e.target.value as PageMode)}
              className="bg-neutral-800 border border-neutral-700 rounded text-xs px-2 py-1"
            >
              {MODES.map((m) => <option key={m} value={m}>{m}</option>)}
            </select>
            <button
              disabled={!active || busy}
              onClick={run}
              className="flex-1 rounded-md bg-brand-500 hover:bg-brand-600 disabled:opacity-50 text-white text-sm py-1.5"
            >
              {busy ? "Recognizing…" : "Run OCR"}
            </button>
          </div>
          <div className="flex flex-wrap gap-1">
            {LANGS.map((l) => (
              <button
                key={l.code}
                onClick={() => toggleLang(l.code)}
                className={`text-[10px] rounded px-1.5 py-0.5 border ${
                  langs.includes(l.code)
                    ? "border-brand-500 text-brand-500"
                    : "border-neutral-700 text-neutral-400"
                }`}
                title={`Needs ${l.code}.traineddata in resources/tessdata`}
              >
                {l.label}
              </button>
            ))}
          </div>
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Find in text…"
            className="w-full bg-neutral-800 border border-neutral-700 rounded text-xs px-2 py-1"
          />
          {ocr && (
            <div className="text-xs text-neutral-400">
              {ocr.words.length} words · lang {ocr.languages}
              {avgConf != null && ` · avg confidence ${Math.round(avgConf * 100)}%`}
            </div>
          )}
          {err && <div className="text-xs text-red-400 break-all">{err}</div>}
        </div>
        <div className="flex-1 p-3 flex flex-col">
          <textarea
            ref={textRef}
            value={ocr?.text ?? ""}
            onChange={(e) => active && updateOcrText(active.id, e.target.value)}
            placeholder="Recognized text appears here. Edit freely."
            className="flex-1 w-full resize-none bg-neutral-950 border border-neutral-800 rounded text-sm p-2 font-mono"
          />
          <div className="pt-2 flex gap-2">
            <button
              disabled={!ocr?.text}
              onClick={() => ocr?.text && navigator.clipboard.writeText(ocr.text)}
              className="text-xs px-3 py-1.5 rounded border border-neutral-700 hover:border-brand-500/60 disabled:opacity-40"
            >
              Copy text
            </button>
          </div>
        </div>
      </aside>
    </section>
  );
}

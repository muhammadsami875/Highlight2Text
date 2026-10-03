import { useState } from "react";
import { pickImages, runOcr } from "../services/ipc";
import type { OcrOptions, PageMode } from "../types/bindings";

interface Row {
  path: string;
  status: "queued" | "running" | "done" | "error" | "cancelled";
  message?: string;
  text?: string;
}

export default function Batch() {
  const [rows, setRows] = useState<Row[]>([]);
  const [running, setRunning] = useState(false);
  const [cancel, setCancel] = useState(false);
  const [langs, setLangs] = useState("eng");
  const [mode, setMode] = useState<PageMode>("Auto");

  async function pick() {
    const files = await pickImages();
    setRows((r) => [...r, ...files.map<Row>((p) => ({ path: p, status: "queued" }))]);
  }

  async function start() {
    setRunning(true); setCancel(false);
    const opts: OcrOptions = { languages: langs, mode };
    const copy = rows.slice();
    for (let i = 0; i < copy.length; i++) {
      if (cancel) {
        copy[i] = { ...copy[i], status: "cancelled" };
        continue;
      }
      if (copy[i].status === "done") continue;
      copy[i] = { ...copy[i], status: "running" };
      setRows(copy.slice());
      try {
        const res = await runOcr(copy[i].path, opts);
        copy[i] = { ...copy[i], status: "done", text: res.text };
      } catch (e) {
        copy[i] = { ...copy[i], status: "error",
          message: String((e as { message?: unknown })?.message ?? e) };
      }
      setRows(copy.slice());
    }
    setRunning(false);
  }

  const done = rows.filter((r) => r.status === "done").length;
  const errs = rows.filter((r) => r.status === "error").length;

  return (
    <section className="p-8 max-w-4xl">
      <h1 className="text-xl font-semibold mb-4">Batch OCR</h1>
      <div className="flex gap-2 items-center mb-4">
        <input
          value={langs}
          onChange={(e) => setLangs(e.target.value)}
          className="bg-neutral-800 border border-neutral-700 rounded text-xs px-2 py-1 w-32"
          placeholder="eng+deu"
        />
        <select
          value={mode}
          onChange={(e) => setMode(e.target.value as PageMode)}
          className="bg-neutral-800 border border-neutral-700 rounded text-xs px-2 py-1"
        >
          {(["Auto","SingleBlock","MultiBlock","SingleLine","SparseText","Table"] as PageMode[])
            .map((m) => <option key={m}>{m}</option>)}
        </select>
        <button onClick={pick} className="text-sm px-3 py-1.5 rounded border border-neutral-700">
          Add files…
        </button>
        <button
          disabled={rows.length === 0 || running}
          onClick={start}
          className="text-sm px-3 py-1.5 rounded bg-brand-500 hover:bg-brand-600 disabled:opacity-40 text-white"
        >
          {running ? "Running…" : `Run on ${rows.length}`}
        </button>
        {running && (
          <button onClick={() => setCancel(true)} className="text-sm px-3 py-1.5 rounded border border-red-700 text-red-300">
            Cancel
          </button>
        )}
      </div>
      <div className="text-xs text-neutral-400 mb-2">
        {done} done · {errs} error(s) · {rows.length} total
      </div>
      <ol className="divide-y divide-neutral-800 border border-neutral-800 rounded">
        {rows.map((r, i) => (
          <li key={i} className="px-3 py-2 text-xs flex items-center gap-3">
            <span className={`inline-block w-20 ${badge(r.status)}`}>{r.status}</span>
            <span className="flex-1 break-all">{r.path}</span>
            {r.message && <span className="text-red-400 max-w-xs truncate">{r.message}</span>}
          </li>
        ))}
      </ol>
    </section>
  );
}

function badge(s: Row["status"]) {
  switch (s) {
    case "done": return "text-emerald-400";
    case "error": return "text-red-400";
    case "running": return "text-brand-500";
    case "cancelled": return "text-neutral-500";
    default: return "text-neutral-400";
  }
}

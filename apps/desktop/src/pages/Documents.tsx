import { useEffect, useMemo, useState } from "react";
import { listProjects, loadProject, saveProject } from "../services/ipc";
import type { ProjectEntry, ProjectPage } from "../types/bindings";
import { useProject } from "../stores/projectStore";

export default function Documents() {
  const [entries, setEntries] = useState<ProjectEntry[]>([]);
  const [query, setQuery] = useState("");
  const [name, setName] = useState("Untitled");
  const [err, setErr] = useState<string | null>(null);
  const { pages, boundaries, warps, enhancements, ocrResults, addPages } = useProject();

  async function refresh() {
    try { setEntries(await listProjects()); setErr(null); }
    catch (e) { setErr(String((e as Error).message ?? e)); }
  }
  useEffect(() => { refresh(); }, []);

  async function doSave() {
    const projectPages: ProjectPage[] = pages.map((p) => ({
      id: p.id,
      source_path: p.source_path,
      source_hash: p.source_hash,
      width: p.width,
      height: p.height,
      rotation: 0,
      corners: (boundaries[p.id]?.working?.corners ?? null) as ProjectPage["corners"],
      warp_target: warps[p.id] ? [warps[p.id]!.warped.width, warps[p.id]!.warped.height] : null,
      // Opaque JSON payloads; the Rust side stores them verbatim.
      enhancement: enhancements[p.id]?.params ? JSON.stringify(enhancements[p.id]!.params) : null,
      ocr: ocrResults[p.id] ? JSON.stringify(ocrResults[p.id]) : null,
    }));
    try { await saveProject(name, projectPages); await refresh(); }
    catch (e) { setErr(String((e as Error).message ?? e)); }
  }

  async function doOpen(path: string) {
    try {
      const proj = await loadProject(path);
      const asImports = proj.pages.map((p) => ({
        id: p.id,
        source_path: p.source_path,
        mime: "image/*",
        width: p.width,
        height: p.height,
        byte_size: 0,
        source_hash: p.source_hash,
        preview_path: p.source_path,
        thumbnail_path: p.source_path,
      }));
      addPages(asImports);
      setErr(null);
    } catch (e) {
      setErr(String((e as Error).message ?? e));
    }
  }

  // Phase 17 — search across OCR results of currently loaded pages, plus
  // project names on disk.
  const results = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return [];
    const hits: { page: number; line: string }[] = [];
    pages.forEach((p, i) => {
      const t = ocrResults[p.id]?.text ?? "";
      t.split(/\r?\n/).forEach((line) => {
        if (line.toLowerCase().includes(q)) hits.push({ page: i + 1, line });
      });
    });
    return hits.slice(0, 100);
  }, [query, pages, ocrResults]);

  const projMatches = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return entries;
    return entries.filter((e) => e.name.toLowerCase().includes(q));
  }, [query, entries]);

  return (
    <section className="p-8 max-w-4xl space-y-6">
      <h1 className="text-xl font-semibold">Documents</h1>
      <div className="flex gap-2 items-center">
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search projects and OCR text in open pages…"
          className="flex-1 bg-neutral-800 border border-neutral-700 rounded text-sm px-3 py-2"
        />
      </div>

      <div className="rounded-lg border border-neutral-800">
        <div className="px-4 py-2 border-b border-neutral-800 flex items-center justify-between">
          <h2 className="text-sm font-medium">Projects</h2>
          <div className="flex gap-2">
            <input
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="bg-neutral-800 border border-neutral-700 rounded text-xs px-2 py-1 w-40"
            />
            <button onClick={doSave} className="text-xs px-3 py-1 rounded bg-brand-500 hover:bg-brand-600 text-white">
              Save current
            </button>
          </div>
        </div>
        {projMatches.length === 0 && (
          <div className="px-4 py-6 text-sm text-neutral-500">No projects yet.</div>
        )}
        <ul className="divide-y divide-neutral-800">
          {projMatches.map((e) => (
            <li key={e.path} className="px-4 py-2 flex items-center justify-between text-sm">
              <div>
                <div>{e.name}</div>
                <div className="text-xs text-neutral-500">
                  {e.modified_ms ? new Date(e.modified_ms).toLocaleString() : ""}
                </div>
              </div>
              <button onClick={() => doOpen(e.path)}
                className="text-xs px-2 py-1 rounded border border-neutral-700 hover:border-brand-500/60">
                Open
              </button>
            </li>
          ))}
        </ul>
      </div>

      {results.length > 0 && (
        <div className="rounded-lg border border-neutral-800">
          <div className="px-4 py-2 border-b border-neutral-800 text-sm font-medium">
            Matches in open pages ({results.length})
          </div>
          <ol className="divide-y divide-neutral-800 text-sm">
            {results.map((r, i) => (
              <li key={i} className="px-4 py-1.5 flex gap-3">
                <span className="text-xs text-neutral-500 w-10">p{r.page}</span>
                <span className="flex-1 truncate text-neutral-300">{r.line}</span>
              </li>
            ))}
          </ol>
        </div>
      )}

      {err && <div className="text-xs text-red-400 break-all">{err}</div>}
    </section>
  );
}

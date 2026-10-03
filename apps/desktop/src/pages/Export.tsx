import { useMemo, useState } from "react";
import { useProject } from "../stores/projectStore";
import {
  exportImage, exportPdf, exportText, savePathWith, savePdfPath,
} from "../services/ipc";
import type {
  ImageFormatKind, MarginSize, PageSize, PdfExportOptions, PdfExportPage, Quality,
} from "../types/bindings";

function effectiveImageFor(id: string, source: string,
  warps: Record<string, { warped: { preview_path: string } }>,
  enh: Record<string, { rendered: { preview_path: string; width: number; height: number } }>) {
  return enh[id]?.rendered.preview_path ?? warps[id]?.warped.preview_path ?? source;
}

export default function Export() {
  const { pages, warps, enhancements, ocrResults } = useProject();
  const [pageSize, setPageSize] = useState<PageSize>("A4");
  const [margin, setMargin] = useState<MarginSize>("Normal");
  const [quality, setQuality] = useState<Quality>("Balanced");
  const [searchable, setSearchable] = useState(true);
  const [imgKind, setImgKind] = useState<ImageFormatKind>("Png");
  const [imgQuality, setImgQuality] = useState(92);
  const [msg, setMsg] = useState<string | null>(null);
  const [err, setErr] = useState<string | null>(null);

  const exportPages = useMemo<PdfExportPage[]>(
    () => pages.map((p) => {
      const img = effectiveImageFor(p.id, p.source_path, warps, enhancements);
      const dims = enhancements[p.id]?.rendered ?? warps[p.id]?.warped;
      return {
        image_path: img,
        width: dims?.width ?? p.width,
        height: dims?.height ?? p.height,
        words: ocrResults[p.id]?.words.map((w) => ({ text: w.text, bbox: w.bbox })) ?? null,
      };
    }),
    [pages, warps, enhancements, ocrResults],
  );

  async function doExportPdf() {
    const out = await savePdfPath();
    if (!out) return;
    const opts: PdfExportOptions = { page_size: pageSize, margin, quality, searchable };
    try {
      await exportPdf(exportPages, out, opts);
      setMsg(`Saved PDF: ${out}`); setErr(null);
    } catch (e) {
      setErr(String((e as { message?: unknown })?.message ?? e)); setMsg(null);
    }
  }

  async function doExportText() {
    const out = await savePathWith("txt", "DocSnap");
    if (!out) return;
    const bodies = pages.map((p) => ocrResults[p.id]?.text ?? "");
    try {
      await exportText(out, bodies);
      setMsg(`Saved text: ${out}`); setErr(null);
    } catch (e) {
      setErr(String((e as { message?: unknown })?.message ?? e)); setMsg(null);
    }
  }

  async function doExportImages() {
    for (const p of pages) {
      const ext = imgKind.toLowerCase();
      const out = await savePathWith(ext, p.id.slice(0, 8));
      if (!out) return;
      const src = effectiveImageFor(p.id, p.source_path, warps, enhancements);
      try { await exportImage(src, out, imgKind, imgQuality); }
      catch (e) {
        setErr(String((e as { message?: unknown })?.message ?? e));
        return;
      }
    }
    setMsg(`Saved ${pages.length} image(s)`);
    setErr(null);
  }

  return (
    <section className="p-8 max-w-3xl space-y-8">
      <h1 className="text-xl font-semibold">Export</h1>
      {pages.length === 0 && <p className="text-sm text-neutral-400">Import pages first in the Editor.</p>}

      <Section title="PDF">
        <div className="grid grid-cols-2 gap-3">
          <Field label="Page size">
            <select value={pageSize} onChange={(e) => setPageSize(e.target.value as PageSize)} className={selectCls}>
              {(["A4","Letter","Legal","Original"] as PageSize[]).map((s) => <option key={s}>{s}</option>)}
            </select>
          </Field>
          <Field label="Margin">
            <select value={margin} onChange={(e) => setMargin(e.target.value as MarginSize)} className={selectCls}>
              {(["None","Small","Normal"] as MarginSize[]).map((s) => <option key={s}>{s}</option>)}
            </select>
          </Field>
          <Field label="Compression">
            <select value={quality} onChange={(e) => setQuality(e.target.value as Quality)} className={selectCls}>
              {(["High","Balanced","Small"] as Quality[]).map((s) => <option key={s}>{s}</option>)}
            </select>
          </Field>
          <Field label="Searchable">
            <label className="flex items-center gap-2 text-sm">
              <input type="checkbox" checked={searchable} onChange={(e) => setSearchable(e.target.checked)} />
              Include OCR text layer (uses current OCR results)
            </label>
          </Field>
        </div>
        <button onClick={doExportPdf} disabled={pages.length === 0} className={btnPrimary}>Export PDF</button>
      </Section>

      <Section title="Text">
        <p className="text-xs text-neutral-500 mb-2">Writes one plain-text file. Pages separated by form-feed.</p>
        <button onClick={doExportText} disabled={pages.length === 0} className={btnSecondary}>Export .txt</button>
      </Section>

      <Section title="Images">
        <div className="grid grid-cols-2 gap-3">
          <Field label="Format">
            <select value={imgKind} onChange={(e) => setImgKind(e.target.value as ImageFormatKind)} className={selectCls}>
              {(["Png","Jpeg","Webp","Tiff"] as ImageFormatKind[]).map((s) => <option key={s}>{s}</option>)}
            </select>
          </Field>
          {imgKind === "Jpeg" && (
            <Field label={`JPEG quality (${imgQuality})`}>
              <input type="range" min={1} max={100} value={imgQuality}
                onChange={(e) => setImgQuality(parseInt(e.target.value, 10))}
                className="w-full accent-brand-500" />
            </Field>
          )}
        </div>
        <button onClick={doExportImages} disabled={pages.length === 0} className={btnSecondary}>
          Export {pages.length} image(s)
        </button>
      </Section>

      {msg && <div className="text-xs text-emerald-400">{msg}</div>}
      {err && <div className="text-xs text-red-400 break-all">{err}</div>}
    </section>
  );
}

const selectCls = "bg-neutral-800 border border-neutral-700 rounded text-sm px-2 py-1.5";
const btnPrimary = "mt-3 rounded-md bg-brand-500 hover:bg-brand-600 disabled:opacity-40 text-white text-sm px-4 py-2";
const btnSecondary = "mt-3 rounded-md border border-neutral-700 hover:border-brand-500/60 disabled:opacity-40 text-neutral-200 text-sm px-4 py-2";

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="border-t border-neutral-800 pt-4">
      <h2 className="text-sm font-medium mb-3">{title}</h2>
      {children}
    </div>
  );
}
function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="block">
      <div className="text-xs text-neutral-400 mb-1">{label}</div>
      {children}
    </label>
  );
}

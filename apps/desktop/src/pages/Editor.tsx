import { useCallback, useMemo, useState } from "react";
import PageThumbnailStrip from "../components/PageThumbnailStrip";
import { useProject } from "../stores/projectStore";
import { assetUrl, importImage, pickImages } from "../services/ipc";
import { useDropTarget } from "../hooks/useDropTarget";

function formatBytes(n: number) {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

export default function Editor() {
  const { pages, activeId, addPages, removePage, setActive } = useProject();
  const [busy, setBusy] = useState(false);
  const [errors, setErrors] = useState<string[]>([]);

  const importMany = useCallback(
    async (paths: string[]) => {
      if (paths.length === 0) return;
      setBusy(true);
      setErrors([]);
      const ok = [];
      const errs: string[] = [];
      for (const p of paths) {
        try {
          ok.push(await importImage(p));
        } catch (e: unknown) {
          const msg =
            typeof e === "object" && e && "message" in e
              ? String((e as { message: unknown }).message)
              : String(e);
          errs.push(`${p}: ${msg}`);
        }
      }
      if (ok.length) addPages(ok);
      if (errs.length) setErrors(errs);
      setBusy(false);
    },
    [addPages],
  );

  const dropOver = useDropTarget(importMany);

  const active = useMemo(
    () => pages.find((p) => p.id === activeId) ?? null,
    [pages, activeId],
  );

  return (
    <section className="grid grid-cols-[220px_1fr_280px] h-full">
      <aside className="border-r border-neutral-800 bg-neutral-900 flex flex-col">
        <div className="px-3 py-2 text-xs uppercase tracking-wide text-neutral-500 border-b border-neutral-800">
          Pages
        </div>
        <div className="flex-1 overflow-y-auto">
          <PageThumbnailStrip
            pages={pages}
            activeId={activeId}
            onSelect={setActive}
            onRemove={removePage}
          />
        </div>
        <div className="p-2 border-t border-neutral-800 space-y-2">
          <button
            disabled={busy}
            onClick={async () => importMany(await pickImages())}
            className="w-full rounded-md bg-brand-500 hover:bg-brand-600 disabled:opacity-50 text-white text-sm py-2"
          >
            {busy ? "Importing…" : "Import images"}
          </button>
        </div>
      </aside>

      <div
        className={`relative flex items-center justify-center overflow-hidden ${
          dropOver ? "bg-brand-500/5" : ""
        }`}
      >
        {active ? (
          <img
            src={assetUrl(active.preview_path)}
            alt="page preview"
            className="max-w-full max-h-full object-contain"
          />
        ) : (
          <div className="text-neutral-500 text-sm text-center max-w-sm px-6">
            Drop PNG, JPEG, TIFF, BMP, or WebP files anywhere in this window, or
            use <span className="text-neutral-300">Import images</span>.
          </div>
        )}
        {dropOver && (
          <div className="absolute inset-4 border-2 border-dashed border-brand-500 rounded-xl pointer-events-none" />
        )}
      </div>

      <aside className="border-l border-neutral-800 bg-neutral-900 p-4 text-sm overflow-y-auto">
        <div className="text-xs uppercase tracking-wide text-neutral-500 mb-3">
          Details
        </div>
        {active ? (
          <dl className="space-y-2 text-neutral-300">
            <Row k="File">
              <span className="break-all">{active.source_path}</span>
            </Row>
            <Row k="Type">{active.mime}</Row>
            <Row k="Dimensions">
              {active.width} × {active.height}
            </Row>
            <Row k="Size">{formatBytes(active.byte_size)}</Row>
            <Row k="Hash">
              <span className="font-mono text-xs">
                {active.source_hash.slice(0, 16)}…
              </span>
            </Row>
          </dl>
        ) : (
          <p className="text-neutral-500">Select a page to see its metadata.</p>
        )}

        {errors.length > 0 && (
          <div className="mt-6 rounded-md border border-red-800 bg-red-900/20 p-3 text-xs">
            <div className="text-red-300 font-medium mb-1">
              {errors.length} file(s) failed
            </div>
            <ul className="list-disc pl-4 text-red-200 space-y-1">
              {errors.slice(0, 5).map((e, i) => (
                <li key={i} className="break-all">{e}</li>
              ))}
              {errors.length > 5 && <li>…and {errors.length - 5} more</li>}
            </ul>
          </div>
        )}
      </aside>
    </section>
  );
}

function Row({ k, children }: { k: string; children: React.ReactNode }) {
  return (
    <div className="grid grid-cols-[90px_1fr] gap-2">
      <dt className="text-neutral-500">{k}</dt>
      <dd>{children}</dd>
    </div>
  );
}

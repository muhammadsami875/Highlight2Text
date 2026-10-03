import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import PageThumbnailStrip from "../components/PageThumbnailStrip";
import CropEditor from "../components/CropEditor";
import EnhancementPanel from "../components/EnhancementPanel";
import { defaultEnhancement, useProject } from "../stores/projectStore";
import {
  applyEnhancement,
  applyPerspective,
  assetUrl,
  detectDocumentBoundary,
  importImage,
  pickImages,
} from "../services/ipc";
import { useDropTarget } from "../hooks/useDropTarget";
import type { Corner, DetectedBoundary, EnhancementParams } from "../types/bindings";

function formatBytes(n: number) {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

export default function Editor() {
  const {
    pages, activeId, boundaries, warps, enhancements,
    addPages, removePage, setActive,
    setDetected, setWorking, resetBoundary,
    setWarp, clearWarp,
    setEnhancement, clearEnhancement,
    reorderPages,
  } = useProject();
  const [busy, setBusy] = useState(false);
  const [detecting, setDetecting] = useState(false);
  const [applying, setApplying] = useState(false);
  const [errors, setErrors] = useState<string[]>([]);

  const importMany = useCallback(
    async (paths: string[]) => {
      if (paths.length === 0) return;
      setBusy(true);
      setErrors([]);
      const ok = [];
      const errs: string[] = [];
      for (const p of paths) {
        try { ok.push(await importImage(p)); }
        catch (e: unknown) {
          const msg = typeof e === "object" && e && "message" in e
            ? String((e as { message: unknown }).message) : String(e);
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
  const boundary = active ? boundaries[active.id] ?? null : null;
  const warp = active ? warps[active.id] ?? null : null;
  const enhancement = active ? enhancements[active.id] ?? null : null;

  // Transient enhancement draft (debounced -> backend).
  const [draft, setDraft] = useState<EnhancementParams | null>(null);
  const draftTimer = useRef<number | null>(null);
  useEffect(() => {
    setDraft(enhancement?.params ?? null);
  }, [active?.id, enhancement?.params]);

  const effectiveSourceForEnh = warp?.warped.preview_path ?? active?.source_path ?? null;

  function scheduleEnhancement(next: EnhancementParams) {
    setDraft(next);
    if (!active || !effectiveSourceForEnh) return;
    if (draftTimer.current) window.clearTimeout(draftTimer.current);
    draftTimer.current = window.setTimeout(async () => {
      try {
        const rendered = await applyEnhancement(effectiveSourceForEnh, next);
        setEnhancement(active.id, { params: next, rendered });
      } catch (e) {
        setErrors([String((e as { message?: unknown })?.message ?? e)]);
      }
    }, 180);
  }

  async function runDetect() {
    if (!active) return;
    setDetecting(true);
    try {
      const b = await detectDocumentBoundary(active.source_path);
      setDetected(active.id, b);
    } catch (e) {
      setErrors([String((e as { message?: unknown })?.message ?? e)]);
    } finally {
      setDetecting(false);
    }
  }

  function handleCornersChange(next: [Corner, Corner, Corner, Corner]) {
    if (!active || !boundary?.working) return;
    const updated: DetectedBoundary = {
      ...boundary.working,
      corners: next,
      fallback: false,
    };
    setWorking(active.id, updated);
  }

  async function applyWarp() {
    if (!active || !boundary?.working) return;
    setApplying(true);
    try {
      const warped = await applyPerspective(active.source_path, boundary.working.corners);
      setWarp(active.id, { corners: boundary.working.corners, warped });
    } catch (e) {
      setErrors([String((e as { message?: unknown })?.message ?? e)]);
    } finally {
      setApplying(false);
    }
  }

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
            onReorder={reorderPages}
          />
        </div>
        <div className="p-2 border-t border-neutral-800">
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
          enhancement ? (
            <img
              src={assetUrl(enhancement.rendered.preview_path)}
              alt="enhanced page"
              className="max-w-full max-h-full object-contain block select-none"
              draggable={false}
            />
          ) : warp ? (
            <img
              src={assetUrl(warp.warped.preview_path)}
              alt="warped page"
              className="max-w-full max-h-full object-contain block select-none"
              draggable={false}
            />
          ) : (
            <div className="relative inline-block max-w-full max-h-full">
              <img
                src={assetUrl(active.preview_path)}
                alt="page preview"
                className="max-w-full max-h-full object-contain block select-none pointer-events-none"
                draggable={false}
              />
              {boundary?.working && (
                <CropEditor
                  srcWidth={active.width}
                  srcHeight={active.height}
                  corners={boundary.working.corners}
                  confidence={boundary.working.confidence}
                  fallback={boundary.working.fallback}
                  onChange={handleCornersChange}
                />
              )}
            </div>
          )
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
        <div className="text-xs uppercase tracking-wide text-neutral-500 mb-3">Details</div>
        {active ? (
          <>
            <dl className="space-y-2 text-neutral-300">
              <Row k="File"><span className="break-all">{active.source_path}</span></Row>
              <Row k="Type">{active.mime}</Row>
              <Row k="Dimensions">{active.width} × {active.height}</Row>
              <Row k="Size">{formatBytes(active.byte_size)}</Row>
            </dl>

            <div className="mt-5">
              <div className="text-xs uppercase tracking-wide text-neutral-500 mb-2">
                Crop
              </div>
              <div className="grid grid-cols-2 gap-2">
                <button
                  disabled={detecting}
                  onClick={runDetect}
                  className="rounded-md border border-neutral-700 hover:border-brand-500/60 disabled:opacity-50 text-neutral-200 text-sm py-2"
                >
                  {detecting ? "Detecting…" : boundary ? "Re-detect" : "Auto-detect"}
                </button>
                <button
                  disabled={!boundary?.edited}
                  onClick={() => active && resetBoundary(active.id)}
                  className="rounded-md border border-neutral-700 hover:border-neutral-500 disabled:opacity-40 text-neutral-200 text-sm py-2"
                >
                  Reset
                </button>
              </div>
              {boundary?.working && (
                <div className="mt-2 text-xs text-neutral-400">
                  {boundary.edited
                    ? "Manually adjusted."
                    : `Auto-detected · ${Math.round(boundary.working.confidence * 100)}% confidence`}
                  {boundary.working.fallback && !boundary.edited && " · fallback"}
                </div>
              )}
              {!boundary && (
                <p className="mt-2 text-xs text-neutral-500">
                  Run auto-detect to place the four corners; drag any corner to
                  adjust. Self-intersecting shapes are rejected.
                </p>
              )}
              <div className="mt-3 grid grid-cols-2 gap-2">
                <button
                  disabled={applying || !boundary?.working || !!warp}
                  onClick={applyWarp}
                  className="rounded-md bg-brand-500 hover:bg-brand-600 disabled:opacity-40 text-white text-sm py-2"
                >
                  {applying ? "Warping…" : warp ? "Applied" : "Apply warp"}
                </button>
                <button
                  disabled={!warp}
                  onClick={() => active && clearWarp(active.id)}
                  className="rounded-md border border-neutral-700 hover:border-neutral-500 disabled:opacity-40 text-neutral-200 text-sm py-2"
                >
                  Clear warp
                </button>
              </div>
              {warp && (
                <div className="mt-2 text-xs text-neutral-400">
                  Warped to {warp.warped.width} × {warp.warped.height}
                </div>
              )}
            </div>

            <div className="mt-5">
              <div className="flex items-center justify-between mb-2">
                <div className="text-xs uppercase tracking-wide text-neutral-500">
                  Enhancement
                </div>
                {enhancement && (
                  <button
                    onClick={() => active && clearEnhancement(active.id)}
                    className="text-xs text-neutral-400 hover:text-neutral-200"
                  >
                    Reset
                  </button>
                )}
              </div>
              <EnhancementPanel
                params={draft ?? defaultEnhancement()}
                onChange={scheduleEnhancement}
                disabled={!active}
              />
            </div>
          </>
        ) : (
          <p className="text-neutral-500">Select a page to see its metadata.</p>
        )}

        {errors.length > 0 && (
          <div className="mt-6 rounded-md border border-red-800 bg-red-900/20 p-3 text-xs">
            <div className="text-red-300 font-medium mb-1">{errors.length} issue(s)</div>
            <ul className="list-disc pl-4 text-red-200 space-y-1">
              {errors.slice(0, 5).map((e, i) => <li key={i} className="break-all">{e}</li>)}
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

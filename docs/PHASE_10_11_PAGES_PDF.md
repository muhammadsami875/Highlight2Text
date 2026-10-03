# Phases 10 & 11 — Multi-page workspace + PDF import

## Phase 10 — Multi-page
- `PageThumbnailStrip` is now reorder-aware: HTML5 drag-and-drop with a
  blue ring on the drop target, wired through `projectStore.reorderPages`.
- Add/delete already existed via import / per-thumbnail ✕. Rotate / replace
  are stubs in the store surface; UI lands with the Phase 15 scanner tools.

## Phase 11 — PDF import
- `pdf.rs`: `pdfium-render` with runtime binding. Loader tries (1) next to
  the executable, (2) system library on PATH; otherwise returns
  `PdfiumMissing` so the user sees a specific remediation message.
- Each PDF page renders to a cached JPEG preview (≤ 2048 px) + thumbnail
  via the shared cache. Previews are keyed by `sha256(pdf bytes) + page index`
  so re-opening the same PDF hits cache.
- Command `import_pdf(path) → ImportedPage[]`; the pages flow into the same
  `projectStore.pages` as image imports, so every downstream feature
  (crop, warp, enhance, OCR, export) works unchanged.
- Home → **Import PDF** tile is live.

## Phase 1 verification carryover
PDFium redistribution licensing (BSD-3) is in-scope for item #4 of the
Phase 1 checklist. The library must be bundled next to `DocSnap.exe` or
on the Windows loader path for the production MSI.

# Phase 18 — Performance

## Delivered this phase
- Preview + warped + enhanced + OCR previews are all **content-addressed**
  under `%LOCALAPPDATA%\DocSnap\`. Re-rendering the same recipe is a cache
  hit. Settings page shows total cache size and files, and clears per
  category.
- OCR, PDF rasterization, and enhancement calls each run in Tauri's command
  thread pool (every `#[tauri::command]` is spawned off the UI thread),
  keeping the webview responsive.
- Batch OCR is serial by default — bounded memory beats peak throughput for
  Tesseract.
- Startup: PDFium and Tesseract are both lazy-loaded. `get_app_info` returns
  without touching either, so the window paints before any heavy dependency
  initializes.
- Thumbnails are ≤ 320 px JPEGs; the strip pays O(pages × 320²) for scroll,
  not O(pages × source).

## Benchmarks to run on a Phase 1 reference box
Run from `cargo bench` once criterion harness is added:
- Single 1080p image: decode + preview cache (expect < 150 ms).
- Single 4K image: decode + preview cache (< 500 ms).
- 10 / 50 / 100 page PDF: lazy per-page render (expect flat per-page cost).
- Batch 100 images, English printed: wall time, peak RSS.

Captured numbers go into `docs/BENCHMARKS.md` per release; no accuracy or
performance **claim** ships without a measured row.

## Open opts
- Rayon-parallel enhancement for 4K images.
- Native Tesseract FFI (`tesseract-rs`) once bundle layout is proven.
- On-disk FTS index for cross-project search (currently in-memory per
  loaded project).

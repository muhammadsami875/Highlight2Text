# Phases 14 & 15 — Batch OCR + Webcam scanner

## Phase 14 — Batch OCR
- `pages/Batch.tsx`: queue runner over `pickImages` results, calling the same
  `run_ocr` command per file. Per-row status (queued/running/done/error/
  cancelled), progress counters, cooperative cancel between rows.
- Serial by default — one Tesseract process at a time keeps resident memory
  bounded. Parallelism is a Phase 18 perf knob.

## Phase 15 — Scanner
- `pages/Scanner.tsx`: `getUserMedia` directly in the Tauri webview with
  explicit Start-camera gating (no silent capture). Device enumeration after
  the first grant; UI switches cameras without reload.
- Capture: `canvas.toDataURL("image/png")` → `save_capture(data_url)` in Rust,
  which base64-decodes, writes a PNG under `cache/captures/`, then runs the
  same `import_from_path` pipeline so the capture becomes an ordinary page
  with EXIF-normalized orientation, thumbnail, and hash.
- CSP widened to allow `blob:` for `media-src` / `img-src` so the preview
  element and in-memory capture frames render.
- No hardware "flash" claim — desktop webcams don't expose one; the brief
  flagged this.

## Deviations from Phase 1 doc
- The Phase 1 doc proposed `nokhwa` (Media Foundation). `getUserMedia` ships
  faster, works on Windows + macOS from the same code, and is already in
  Tauri's permission model. `nokhwa` remains the fallback option if a user
  needs lower-level access (manual exposure, raw frames); gated by a
  Phase 18 performance decision.

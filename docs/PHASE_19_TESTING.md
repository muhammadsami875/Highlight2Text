# Phase 19 — Testing

## Delivered
- Rust unit tests live beside the code they cover:
  - `document_detection`: corner ordering, polygon area.
  - `perspective`: identity homography round-trip, target dim selection.
  - `security`: missing-path rejection, directory rejection.
- CI runs `cargo check` + `cargo test` for `docsnap` on `windows-latest` and
  `ubuntu-latest` (`.github/workflows/ci.yml`). TS typecheck runs on Ubuntu.

## Not yet
- Golden fixture corpus (clean / low-light / skewed / perspective / shadow /
  colored BG / low-res / high-res / multi-column / table / EN+UR / EN+AR /
  photographed / PDF / multi-page). Each fixture ships with:
  - A source image (self-generated or SIL OFL text set).
  - Ground-truth text.
  - Expected detector corners.
- `cargo-fuzz` targets for image decode and PDF parse.
- Playwright flow over the Tauri webview build for the Editor and OCR paths.

These rely on the fixture-licensing decision in the Phase 1 checklist and
land once that closes.

## CER / WER reporting
When the corpus is in place, `scripts/eval-ocr.ps1` runs OCR over every
fixture, writes per-class CER/WER into `docs/ACCURACY.md`, and compares to
the previous release's row. No global accuracy % ships without this file.

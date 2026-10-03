# Phases 8 & 9 — OCR engine + text editor

## Phase 8 — Engine
- `ocr.rs`: Tesseract sidecar manager. Spawns `tesseract` as a child process
  twice per page (text + TSV), with `TESSDATA_PREFIX` pointed at bundled
  `resources/tessdata/` when present and `which("tesseract")` as a dev
  fallback. Clear `EngineMissing` error when neither resolves.
- `PageMode` → PSM mapping: Auto 3, SingleBlock 6, MultiBlock 1, SingleLine 7,
  SparseText 11, Table 4.
- Per-word output: text, bbox, confidence (0..1, only reported — never
  fabricated), line id.
- Command `run_ocr(path, options) → OcrResult`.

## Phase 9 — Text editor
- OCR page: three panes. Left — page list; center — the image for OCR
  (prefers enhancement > warp > source, same chain as export will use) with
  per-word SVG boxes; right — language chips (eng/ara/urd/hin/fra/deu/spa/
  ita/por/chi_sim/jpn/kor), mode dropdown, Run OCR, Find-in-text, and the
  editable text pane. Multilingual via `+` concat on language codes.
- Low-confidence (<60%) words highlight red; search matches highlight amber.
- `projectStore.ocrResults` indexed by page id; text edits persist.

## Known gaps
- Language chips are shown unconditionally. Phase 20 adds a language-pack
  installer that disables chips whose `*.traineddata` isn't present.
- The CLI integration is the Phase 1 "sidecar" promise; swapping to a native
  Tesseract FFI (e.g. `tesseract-rs`) is a Phase 18 optimization once
  bundling is tested.
- No handwriting recognition; printed text only, as the brief requires.

# Phase 1 — Architecture & Foundations

Working name: **DocSnap** (placeholder; original branding, not derivative of CamScanner / Adobe Scan / MS Lens).
Target: Windows 10 / 11 desktop, offline-first. macOS portable.

---

## 1. High-Level Architecture

Three-layer app, process-isolated:

```
┌──────────────────────────────────────────────────────────┐
│  UI Layer  (React + TypeScript, Vite, Tailwind)          │
│  - Pages, stores (Zustand), hooks, non-destructive edits │
└──────────────▲────────────── IPC (Tauri commands) ───────┘
               │
┌──────────────┴───────────────────────────────────────────┐
│  Rust Core  (Tauri 2 backend)                            │
│  - Orchestration, project store, filesystem,             │
│    job queue, async workers (tokio)                      │
│  - FFI into image/pdf libs; spawns OCR sidecar           │
└──────┬──────────────┬──────────────┬─────────────────────┘
       │              │              │
   OpenCV FFI    PDF libs       OCR Sidecar
   (opencv-rs)   (pdfium +      (Tesseract 5.x,
                  printpdf)      bundled binary)
```

Rationale: Tauri gives small installer, native perf, and OS integration without an Electron runtime. React/TS keeps UI iteration fast. Rust holds CPU-bound pipelines close to the libraries that already do the work well. OCR runs out-of-process as a sidecar so a Tesseract crash or hang cannot take down the app, and language data can be swapped at runtime.

---

## 2. Component Responsibilities

### 2.1 Frontend (`apps/desktop/src`)
- **pages/**: Home, Scanner, Editor, OCR, Documents, Settings.
- **components/**: `CropEditor` (SVG corner handles), `PageThumbnailStrip`, `EnhancementPanel`, `OcrTextPane`, `CameraCapture`, `BatchQueue`.
- **stores/**: `projectStore` (open project, pages, dirty state, undo/redo stacks), `settingsStore`, `jobStore` (OCR/export progress), `uiStore` (theme, zoom).
- **services/**: thin IPC wrappers for Tauri commands; no business logic here.
- **hooks/**: `useProject`, `useOcr`, `useCamera`, `useShortcuts`, `useDropTarget`.
- Non-destructive editing: all page edits are **recipes** (JSON ops) applied to a cached source image; original asset is never mutated.

### 2.2 Rust core (`apps/desktop/src-tauri/src`)
- `commands/` — Tauri `#[tauri::command]` surface, versioned.
- `image_processing/` — decode, orient, denoise, enhance, deskew (OpenCV).
- `document_detection/` — boundary + corner pipeline (see §5).
- `perspective/` — corner ordering + warp (see §6).
- `ocr/` — sidecar manager, language registry, job dispatch, hOCR/TSV parser.
- `pdf/` — PDF import (pdfium render), PDF export (printpdf/lopdf), searchable layer builder.
- `camera/` — Windows: `nokhwa` (Media Foundation backend); macOS later: AVFoundation via same `nokhwa` abstraction.
- `filesystem/` — safe path handling, atomic writes, dialogs.
- `projects/` — `.docsnap` package reader/writer (see §9).
- `export/` — image / text / PDF / DOCX / CSV writers.
- `settings/` — serde-persisted user config.
- `security/` — path validation, MIME sniffing, size limits, sidecar sandboxing.

### 2.3 Workers / jobs
- Tokio runtime; a bounded `JobQueue` with priorities (ui-blocking, interactive-preview, background).
- Heavy tasks (OCR, batch, PDF export, 4K pipeline) are jobs with cancel tokens and progress events pushed to UI via Tauri events (`job:progress`, `job:complete`, `job:error`).

---

## 3. OCR Architecture

- **Engine**: Tesseract 5.x, shipped as a bundled sidecar binary (Windows: `tesseract.exe` + `tessdata/`), located under the app's resources dir — resolved via `tauri::api::path::resource_dir`. Users do not install Tesseract.
- **Invocation**: Rust spawns the sidecar per job with `--psm`, `-l`, and a temp input image; outputs TSV + hOCR for coordinates and per-word confidence. Timeouts + kill-on-cancel.
- **Language registry**: `tessdata/` is scanned at startup; UI shows only installed packs. Language packs are downloadable as separate signed bundles (post-MVP) so the base installer stays small.
- **Multilingual**: multiple `-l eng+deu` combinations supported where the engine permits; UI exposes a multi-select.
- **Modes mapping** (user → PSM):
  - AUTO → 3, SINGLE BLOCK → 6, MULTI BLOCK → 1, SINGLE LINE → 7, SPARSE → 11, TABLE/DOC → 4.
- **Preprocessing for OCR** is a separate copy of the page (grayscale → denoise → adaptive threshold → deskew → DPI normalize to ~300). The *display* page is not replaced.
- **Pluggable**: `trait OcrEngine { fn recognize(&self, img, opts) -> OcrResult; }` so a future engine (PaddleOCR / cloud opt-in) slots in without touching UI.
- **Confidence**: taken verbatim from Tesseract's per-word confidence; never fabricated. Words below a user-configurable threshold are flagged in the review view.

---

## 4. OpenCV / Image-Processing Pipeline

Rust binding: `opencv` crate (statically linked OpenCV built against prebuilt Windows binaries, or vcpkg). All operations run on an `cv::Mat` graph with lazy evaluation per page.

Canonical pipeline (non-destructive, each stage optional):

```
Decode  → Orient (EXIF)  → Document Detect  → Corner Adjust
       → Perspective Warp  → Deskew  → Enhance preset
       → Shadow/Background cleanup  → Sharpen/Denoise
       → (branch) Display render        ─────→ UI
       → (branch) OCR preprocess copy   ─────→ OCR engine
```

Enhancement presets (deterministic recipes, no hidden ML by default):
- **Original**: identity.
- **Color**: mild WB + contrast stretch.
- **Auto**: CLAHE + mild gamma.
- **Document**: adaptive threshold with gentle smoothing (preserves hand strokes).
- **B&W**: Otsu after denoise.
- **Grayscale**: luminance channel only.
- **High Contrast**: CLAHE + S-curve.

Shadow removal: morphological background estimation (dilate large kernel → divide) — standard document-flattening approach, strength-controlled.

---

## 5. Document Detection Strategy

Multi-strategy with voting; no single hardcoded threshold.

1. Downscale to max ~1024 px.
2. Convert to Lab; use L channel.
3. Bilateral filter.
4. Three edge candidates in parallel:
   - a. Canny with auto thresholds (median × [0.66, 1.33]).
   - b. Structured edge (OpenCV ximgproc) if available.
   - c. Gradient-magnitude threshold.
5. Morphological close to join edges.
6. Find contours, keep top-k by area.
7. `approxPolyDP` with adaptive epsilon; accept quads with convex, area ≥ 20% of frame, min angle > 60°.
8. Score candidates by (area, rectangularity, edge-support, central-ness).
9. Fallback: if no quad passes, return the full-frame rectangle with a `low_confidence` flag so UI prompts manual crop.
10. Corners ordered TL/TR/BR/BL by sum/diff of coords.

The score and corners are returned to UI; the user can accept, re-run, or drag.

---

## 6. Perspective Correction Strategy

- Given ordered 4 corners `(TL, TR, BR, BL)` in source pixels.
- Target width `W = max(||TR−TL||, ||BR−BL||)`, height `H = max(||BL−TL||, ||BR−TR||)`.
- `getPerspectiveTransform` + `warpPerspective` at native resolution of the source (no upscale). Output size `(W, H)` clamped to source megapixel budget.
- Reject self-intersecting polygons (segment-segment test) before warp.
- The warp matrix is stored in the project; the warp is re-applied on export rather than baked eagerly, so corner edits remain reversible.

---

## 7. PDF Architecture

- **Import**: `pdfium-render` (Chromium PDFium) via bundled DLL. Lazy page access; render page `n` to `RGBA` at target DPI only when needed (preview = 96–144 dpi, OCR = 300 dpi, export = original). A `PdfSource` keeps a handle + page count; pages never all held in RAM.
- **Export image-only PDF**: `printpdf` — each page is a JPEG/PNG XObject at chosen compression.
- **Searchable PDF**: same page image **plus** an invisible text layer:
  - Render text with `Rendering Mode 3` (invisible) using a stock font (Helvetica for latin, embedded NotoSans for CJK/Arabic/Urdu — license permitting).
  - Position words from Tesseract's hOCR bounding boxes mapped to PDF units; scale font size to box height.
  - RTL scripts: shape with `rustybuzz` before placement.
- **Page sizes**: A4 / Letter / Legal / Original / Custom; margins preset.
- **Compression**: High (lossless PNG), Balanced (JPEG q85), Small (JPEG q65 + downscale cap).

---

## 8. Multi-Page Document Model

```rust
struct Project {
  id: Uuid,
  name: String,
  version: u32,                 // schema version
  created_at: DateTime, updated_at: DateTime,
  pages: Vec<Page>,             // explicit order
  settings: ProjectSettings,
}

struct Page {
  id: Uuid,
  source: AssetRef,             // path inside the .docsnap or external
  thumbnail: AssetRef,          // small cached jpg
  rotation: i16,                // 0/90/180/270
  corners: Option<[(f32,f32);4]>, // in source px
  warp_target: Option<(u32,u32)>,
  enhancement: EnhancementRecipe,
  deskew_deg: f32,
  ocr: Option<OcrResult>,       // text + words[] + confidences + lang
  notes: Option<String>,
}
```

Edits are **recipes**, not pixels. Rendering a page composes recipes on demand; results are cached by content hash.

Undo/redo: per-page command stack in the frontend; persisted actions are commit-level (snapshot of recipe).

---

## 9. Local Project Storage (`.docsnap`)

A `.docsnap` file is a **ZIP container** (stable, portable, easy to inspect):

```
project.json          # Project struct, schema versioned
assets/
  <uuid>.jpg|png|tiff # originals (user may choose "copy" or "reference")
cache/
  thumbs/<uuid>.jpg
  previews/<hash>.jpg
ocr/
  <uuid>.json         # OCR raw outputs
```

- **Atomic save**: write to `name.docsnap.tmp` then `rename` over the target; on Windows use `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING`.
- **Autosave**: throttled every N seconds *into the cache*, promoted to main file on explicit Save.
- **Backward compat**: `version` field + a migration table; older projects open read-only if a migration is missing.
- **Never silently overwrite**: on Save As over an existing file, UI confirms.

---

## 10. Security Model

- **Input validation**: Magic-byte sniffing before extension trust; max dimensions/size enforced; refuse decoding on mismatch.
- **Path handling**: all user paths resolved and checked against `canonicalize`; reject paths escaping the project root for in-project operations; no shell interpolation ever.
- **Sidecar**: Tesseract invoked via `std::process::Command` with explicit args, no shell; temp files in a per-session dir under `LOCALAPPDATA\DocSnap\tmp` with restrictive ACLs; cleaned on exit.
- **PDFium / OpenCV**: pinned versions; fuzz-tested entry points (decoder, PDF parse) in CI.
- **Camera**: no capture until user clicks Start; permission dialog explains purpose.
- **Network**: default-deny. No telemetry, no uploads. Any future cloud feature is a clearly labelled opt-in with per-request confirmation.
- **Updater**: Tauri updater behind HTTPS + signature verification; disabled unless user opts in.
- **Secrets**: none stored. If future cloud OCR is added, credentials go through Windows Credential Manager / macOS Keychain.

---

## 11. Tauri Native / Sidecar Architecture

- Tauri 2 with `tauri.conf.json` declaring:
  - `bundle.resources`: `tessdata/`, `pdfium.dll` (Win) / `.dylib` (mac), bundled fonts.
  - `bundle.externalBin`: `tesseract` (per-triple binary: `tesseract-x86_64-pc-windows-msvc.exe`).
- Capabilities (ACL): tight allowlist — fs scoped to project + export dirs, dialog, shell *only* for `tesseract` externalBin, no HTTP by default.
- A `SidecarManager` in Rust owns child-process lifetimes, backpressure, and crash restart with exponential backoff.

---

## 12. Dependency & Licensing Analysis (to verify in Phase 1 exit)

| Component            | License            | Notes |
|---                   |---                 |---|
| Tauri 2              | MIT / Apache-2.0   | OK for commercial redistribution. |
| React, TypeScript    | MIT                | OK. |
| Tailwind             | MIT                | OK. |
| Zustand              | MIT                | OK. |
| Rust std + tokio     | MIT / Apache-2.0   | OK. |
| `opencv` crate       | MIT                | OpenCV itself: **Apache-2.0 (≥ 4.5)**; verify version. Earlier was BSD. |
| OpenCV binaries      | Apache-2.0         | Build from source or ship prebuilt; attribution required. |
| Leptonica (via Tess) | BSD-2 style        | OK, attribute. |
| Tesseract 5.x        | Apache-2.0         | OK for bundling; attribute. |
| `tessdata_fast`      | Apache-2.0         | Verify per-language; some older models were different — **confirm per pack**. |
| PDFium               | BSD-3              | OK; attribute. |
| `pdfium-render`      | MIT / Apache-2.0   | OK. |
| `printpdf` / `lopdf` | MIT                | OK. |
| `image`, `imageproc` | MIT / Apache-2.0   | OK. |
| `nokhwa` (camera)    | Apache-2.0 / MIT   | Verify Media Foundation feature flags. |
| `rustybuzz`          | MIT                | OK. |
| Noto fonts (CJK/Arabic/Urdu) | SIL OFL 1.1 | OK; include license file. |
| `docx-rs`            | MIT                | For DOCX export. |
| `calamine`/`rust_xlsxwriter` | MIT / Apache-2.0 | For XLSX export. |

Deliverables at Phase 1 close: `THIRD-PARTY-NOTICES.md`, `LICENSES/` dir, per-language traineddata license index.

---

## 13. Repository Structure

```
/apps
  /desktop
    /src                 # React + TS
      /components
      /pages
      /hooks
      /stores
      /services
      /types
      /utils
      /styles
    /src-tauri
      /src
        /commands
        /ocr
        /image_processing
        /document_detection
        /perspective
        /pdf
        /camera
        /filesystem
        /projects
        /export
        /settings
        /security
      /resources
        /tessdata        # bundled language packs (lazy loaded)
        /binaries        # tesseract sidecar
        /fonts           # Noto fallbacks
      tauri.conf.json
      Cargo.toml
    package.json
    vite.config.ts
/packages
  /ocr-ipc-schema        # shared TS/Rust types (via ts-rs or specta)
  /ui-kit                # original DocSnap components, icons, tokens
/docs
  PHASE_1_ARCHITECTURE.md
  THIRD-PARTY-NOTICES.md
  SECURITY.md
  TESTING.md
/tests
  /fixtures              # test corpus (see §15)
  /e2e
  /perf
/scripts
  build-windows.ps1
  bundle-tessdata.ps1
/.github/workflows
  ci.yml
  release-windows.yml
```

Monorepo tooling: `pnpm` workspaces + Cargo workspace. Type sharing via **specta** (Rust → TS types) so IPC schemas can't drift.

---

## 14. Development Roadmap

Phase ladder matches the prompt (1 → 22). Gating:

- Each phase lands behind a feature flag in `settingsStore.experimental`.
- No phase ships to users until its tests in §15 are green and its licensing row in §12 is confirmed.
- Phase 2 cannot start until the Phase-1 verification checklist (end of this doc) is signed off.

Key milestones:
- **M1 (end P7)**: single-image → cropped → enhanced → saved PNG.
- **M2 (end P9)**: single-image → OCR → TXT export, local only.
- **M3 (end P12)**: multi-page project → searchable PDF.
- **M4 (end P15)**: webcam capture end-to-end.
- **M5 (end P20)**: signed Windows installer, offline, with ≥ 2 language packs.

---

## 15. Testing Strategy

- **Unit (Rust)**: pure functions in `image_processing`, `perspective`, `pdf`, `projects`. Run under `cargo nextest`.
- **Golden tests**: curated fixture corpus (see categories below). For each, snapshot: detected corners, warped image hash, enhancement histogram summary, OCR text (CER/WER vs ground truth).
- **Integration**: end-to-end pipelines invoked through the Tauri command surface with a headless harness.
- **UI**: Playwright against the Tauri webview build; keyboard shortcuts, drag/drop, undo/redo, batch cancel.
- **Property tests**: `proptest` for corner ordering, PDF unit mapping, non-self-intersecting polygon invariants.
- **Fuzzing**: `cargo-fuzz` targets for image decode and PDF parse.
- **Fixture categories** (shipped under `tests/fixtures`, licenses recorded): clean printed, low-light, skewed, perspective, shadowed, colored BG, low-res, high-res, multi-column, table, EN+UR, EN+AR, photographed, PDF, multi-page.
- **Accuracy reporting**: OCR CER/WER published per fixture class per release. No global "accuracy %" claim without a public dataset.
- **CI**: GitHub Actions, Windows + Linux (Linux for Rust unit tests only). Nightly perf run.

---

## 16. Performance Strategy

- **Memory**: Pages stream from disk; only the active + N neighbors are decoded. Thumbnails are tiny JPEGs. OCR runs on a page-size-capped grayscale copy.
- **Compute**: OpenCV uses prebuilt binaries with TBB / OpenMP where licensing allows; Rust side uses Rayon for CPU-parallel enhancement.
- **Caching**: content-addressed preview cache keyed by `(source_hash, recipe_hash)`; LRU eviction; user-visible size + clear button.
- **Backpressure**: `JobQueue` with configurable concurrency per class (default: 1 OCR, 2 enhancement, 1 PDF export). Prevents OOM on batch runs.
- **UI responsiveness**: all > 50 ms work is a job; progress is pushed every 100 ms or 5% whichever first.
- **Benchmarks** (recorded, not promised): single 1080p / single 4K / 10-page / 50-page / 100-page PDF / 100-image batch; metrics = wall time, peak RSS, CPU%, disk I/O.
- **Startup**: Tesseract and PDFium lazy-loaded on first use, not at app launch.

---

## Verification Checklist — Must Be Confirmed Before Phase 2

1. **Tauri 2** stable release and current `tauri.conf.json` v2 schema confirmed against official docs.
2. **OpenCV** version (**≥ 4.9**) license (Apache-2.0) and Windows prebuilt build strategy chosen (vcpkg vs. official prebuilt vs. build-from-source).
3. **Tesseract 5.x** redistribution confirmed; `tessdata_fast` vs `tessdata_best` license status checked for every language in the initial pack list.
4. **PDFium** redistribution licenses confirmed for Windows x64 (and arm64 if targeted).
5. **nokhwa** camera crate's Windows Media Foundation backend verified against current Windows 10/11 behavior.
6. **specta** (or `ts-rs`) chosen and proven for Rust↔TS type sharing.
7. **Font licensing** for RTL/CJK OCR text layers in searchable PDFs resolved (SIL OFL fonts selected).
8. **Signing / notarization** story for the Windows installer (Authenticode cert source, timestamping).
9. **Original branding assets** scoped: app name, logo, color tokens, icon set — none derivative of CamScanner / Adobe Scan / MS Lens.
10. **Legal review** of the `THIRD-PARTY-NOTICES.md` seed.
11. **Minimum hardware target** chosen (RAM/CPU) and benchmarks baseline captured on it.
12. **Threat model** signed off: file inputs, sidecar process, no-network default, future cloud opt-in path.
13. **Fixture corpus licensing** for test images (self-generated vs. permissively licensed) confirmed.
14. **Monorepo tooling** (pnpm + Cargo workspaces + specta) scaffolded in a throwaway spike to confirm the type-sharing + build pipeline works end-to-end on Windows.

Once all 14 items are green, Phase 2 (Tauri + React + TypeScript + Rust project setup) begins.

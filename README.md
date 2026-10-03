# DocSnap

Offline-first document scanner and OCR app for Windows (macOS portable).
Scaffolded end-to-end across 22 phases on this branch; a Windows dev box can
build and run it, then burn down the Phase 1 verification checklist before
cutting a release.

> This repository also contains an earlier **Highlight2Text** Chrome extension
> at the root (`background.js`, `content.js`, `popup.*`, `manifest.json`,
> `universal-file-converter/`). It is kept as-is. The desktop app lives
> under `apps/desktop/`.

---

## Quickstart (Windows dev)

```powershell
pnpm install
# first run creates src/types/bindings.ts from Rust command signatures
pnpm --filter docsnap-desktop tauri dev
```

Before `tauri build`, stage bundled dependencies into
`apps/desktop/src-tauri/resources/`:

- `binaries/tesseract.exe` + its DLLs (Apache-2.0)
- `tessdata/*.traineddata` for every language chip the UI shows
- `binaries/pdfium.dll` (BSD-3)
- `fonts/NotoSans-*.ttf` for RTL/CJK text layers (SIL OFL)

The release workflow (`.github/workflows/release-windows.yml`) fails loudly
if any of the above is missing. Full operator checklist:
[`docs/PHASE_20_INSTALLER.md`](docs/PHASE_20_INSTALLER.md).

---

## What's where

```
apps/desktop/
  src/                 React + TS UI
    pages/             Home / Scanner / Editor / Ocr / Batch / Export / Documents / Settings
    components/        CropEditor, EnhancementPanel, PageThumbnailStrip
    stores/            Zustand (projectStore, settingsStore)
    services/ipc.ts    Typed Tauri invoke wrappers
    types/bindings.ts  Rust ↔ TS types (specta-exported)
  src-tauri/           Rust backend
    src/
      commands.rs      All frontend-reachable IPC
      image_processing.rs   Decode, EXIF orient, enhancement presets + sliders
      document_detection.rs Canny → contours → RDP → scored quad + fallback
      perspective.rs   8×8 homography + bilinear warp
      ocr.rs           Tesseract sidecar (text + TSV)
      pdf.rs           pdfium-render import + printpdf export (searchable)
      export.rs        TXT / PNG / JPEG / WebP / TIFF writers
      projects.rs      .docsnap ZIP reader/writer, atomic save
      security.rs      Canonical path, size cap, magic-byte MIME
      filesystem.rs    Cache dirs (previews / thumbs / ocr / captures)
    resources/         bundled tessdata / binaries / fonts (release only)
docs/                  one markdown per phase (see table below)
.github/workflows/     ci.yml and release-windows.yml
Cargo.toml             workspace manifest
pnpm-workspace.yaml
THIRD-PARTY-NOTICES.md seed; regenerated per release
```

---

## Phases

| # | Doc | Highlights |
|---|---|---|
| 1 | [architecture](docs/PHASE_1_ARCHITECTURE.md) | 16-section architecture + 14-item verification checklist |
| 2 | [scaffold](docs/PHASE_2_SCAFFOLD.md) | Tauri 2 + Vite + strict TS + Rust workspace |
| 3 | [import](docs/PHASE_3_IMPORT.md) | Drag-drop + file picker, EXIF orient, SHA-256 cache |
| 4 | [detection](docs/PHASE_4_DETECTION.md) | Canny + contours + RDP, scored quad, amber fallback |
| 5 | [manual crop](docs/PHASE_5_MANUAL_CROP.md) | Pointer-captured corners, self-intersect guard |
| 6 | [perspective](docs/PHASE_6_PERSPECTIVE.md) | Homography solve + inverse-sampled warp |
| 7 | [enhancement](docs/PHASE_7_ENHANCEMENT.md) | 7 presets + 4 sliders, debounced |
| 8–9 | [OCR](docs/PHASE_8_9_OCR.md) | Tesseract sidecar; per-word boxes; language chips |
| 10–11 | [pages + PDF](docs/PHASE_10_11_PAGES_PDF.md) | Drag-reorder; pdfium-render import |
| 12–13 | [export](docs/PHASE_12_13_EXPORT.md) | Searchable PDF; txt / png / jpeg / webp / tiff |
| 14–15 | [batch + scanner](docs/PHASE_14_15_BATCH_SCANNER.md) | Queue + cancel; webcam capture |
| 16–17 | [projects + search](docs/PHASE_16_17_PROJECTS_SEARCH.md) | `.docsnap` ZIP; OCR text search |
| 18 | [performance](docs/PHASE_18_PERFORMANCE.md) | Content-addressed cache; benchmark plan |
| 19 | [testing](docs/PHASE_19_TESTING.md) | Unit tests; fixture plan; CER/WER protocol |
| 20 | [installer](docs/PHASE_20_INSTALLER.md) | MSI + NSIS pipeline; operator checklist |
| 21 | [security + privacy](docs/PHASE_21_SECURITY_PRIVACY.md) | Threat model; privacy posture |
| 22 | [macOS prep](docs/PHASE_22_MACOS_PREP.md) | What's portable, what needs shims |

---

## Non-destructive guarantee

Edits are recipes, not pixels.

- Boundary → four source-pixel corners.
- Warp → `corners + target dims`.
- Enhancement → preset + four scalars.
- OCR → text + per-word boxes + confidences (reported, never fabricated).

`.docsnap` persists these recipes; renders are a pure function of recipe +
`source_hash`, cached under `%LOCALAPPDATA%\DocSnap\`. Clearing the cache
invalidates derived files only; originals are never touched.

---

## Keyboard shortcuts

| Combo | Action |
|---|---|
| Ctrl+O | Import images |
| Ctrl+Shift+O | Import PDF |
| Ctrl+E | Export page |
| Ctrl+Shift+E | OCR page |
| Ctrl+F | Documents / search |
| Delete | Remove the active page |

---

## Deliberate deviations from the Phase 1 doc

Each is explained inline in its phase doc:

- **OpenCV deferred.** `imageproc` + hand-written homography keep Windows
  builds dependency-free until the vcpkg/prebuilt decision closes. Pipeline
  is behind a module boundary; swap is local.
- **`getUserMedia` instead of `nokhwa`.** Fewer OS shims, same permission
  model.
- **CJK/RTL searchable-PDF layers use Helvetica for now.** Noto embedding
  is a Phase 20 operator step; Latin text layers already work.
- **DOCX / XLSX export left as follow-ups.** The brief itself cautions
  against promising perfect layout; shipped only after a measured smoke
  test on representative OCR output.

---

## What has NOT been done

- The code compiles **on paper** — no `cargo check` or `pnpm install` has
  run inside this scaffold. First Windows build will likely surface
  version-pin and API-drift issues (particularly `pdfium-render`,
  `printpdf`, `tauri-specta` — all RC/pre-1.0). Treat the first dev-box
  pass as the Phase-1 verification spike.
- No fixture corpus. See [Phase 19 doc](docs/PHASE_19_TESTING.md) for the
  categories and the CER/WER protocol.
- No measured benchmarks or accuracy numbers; none are claimed anywhere in
  the UI or docs.
- No signed MSI, no notarization, no language-pack installer UI.

---

## License

Project license: TBD. Bundled components retain their own licenses — see
[`THIRD-PARTY-NOTICES.md`](THIRD-PARTY-NOTICES.md) and the per-component
`LICENSES/` folder that the installer places next to the executable.

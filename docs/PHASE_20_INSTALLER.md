# Phase 20 — Windows installer

## Delivered
- `tauri.conf.json` already targets MSI (WiX) and NSIS.
- `.github/workflows/release-windows.yml` is the release pipeline. It fails
  loudly if `resources/binaries/tesseract.exe` is missing so a Windows build
  can't ship without the bundled OCR engine.
- `pdfium.dll` lives next to the exe at install time (bundled via
  `tauri.conf.json → bundle.resources`).

## Operator checklist before cutting a release
1. Fetch Apache-2.0 Tesseract 5.x build + Leptonica DLLs, drop them into
   `apps/desktop/src-tauri/resources/binaries/`.
2. Fetch `*.traineddata` from `tessdata_fast` for every language listed in
   `/apps/desktop/src/pages/Ocr.tsx`, drop into
   `apps/desktop/src-tauri/resources/tessdata/`.
3. Fetch PDFium (BSD-3) for `x86_64-pc-windows-msvc`, drop `pdfium.dll` into
   `apps/desktop/src-tauri/resources/binaries/`.
4. Add Noto Sans + Noto Sans Arabic + Noto Sans CJK (SIL OFL) under
   `resources/fonts/` for RTL/CJK text layers in searchable PDFs.
5. Regenerate `THIRD-PARTY-NOTICES.md` (seed in repo root) with the version
   strings actually shipped.
6. Authenticode-sign the MSI with the project cert; archive timestamping
   per Microsoft's current `signtool` guidance (`/tr`).
7. Smoke test on a clean Windows 10 VM: install, run, OCR an English PNG,
   export a searchable PDF, uninstall.

## File associations
Not set automatically. The installer offers them as opt-in checkboxes
(`.docsnap` to DocSnap, `.pdf` left to the system default by default) —
the brief explicitly forbids silent association changes.

# Phase 22 — macOS readiness

## Already portable
- Tauri 2 ships `.app` + `.dmg` targets with one config change
  (`bundle.targets += ["app", "dmg"]`).
- All processing modules (`image_processing`, `document_detection`,
  `perspective`, `pdf`, `projects`, `export`) are platform-agnostic Rust.
- `dirs` already resolves to macOS's `~/Library/{Caches,Application Support}`.
- `getUserMedia`-based scanner works unchanged (AVFoundation under the
  WKWebView).

## Needs OS-specific work before a mac build
1. **Tesseract sidecar**: ship a signed, notarized `tesseract` binary for
   `aarch64-apple-darwin` and `x86_64-apple-darwin`. Add
   `tauri.conf.json → bundle.externalBin` entry per triple.
2. **PDFium**: swap `pdfium.dll` for the `.dylib` build; same loader
   fallback path (`pdfium_platform_library_name_at_path`).
3. **Camera permission string** in `Info.plist`
   (`NSCameraUsageDescription = "DocSnap needs camera access to capture
    documents."`).
4. **Hardened runtime** + **notarization**. The release-mac workflow mirrors
   `release-windows.yml`; drop in Apple ID + app-specific password secrets.
5. **File open events** (`tauri://file-drop` for Dock drops; the pattern
   also helps `.docsnap` double-click open on macOS).

## Non-portable blockers
- None known. The brief's "architect for later macOS" is satisfied by keeping
  every OS call behind a module boundary; the shims above are config,
  not architectural rework.

# Phase 21 — Security & Privacy audit

## Threat model
Trust boundaries:
1. **User-supplied files** (image, PDF). Treated as hostile. Mitigations:
   - `security::validate_input_path` canonicalizes, size-caps (500 MB), and
     rejects non-regular files.
   - `infer::get_from_path` magic-byte sniff rejects extension-mismatched
     content before decoders see it.
   - Decoders (`image`, `pdfium-render`) are pinned; CI fuzzing targets land
     in Phase 19 extensions.
2. **Sidecar process (Tesseract)**. Spawned with explicit args and no shell;
   stdin/stdout/stderr piped so no inherited handles leak. Temp inputs
   written under per-session `cache/ocr/` and removed after read.
3. **Webview content**. CSP locks `connect-src` to IPC only; `img-src`
   permits `asset:` + `data:` + `blob:` for in-app rendering; nothing else
   can hit the network.
4. **Camera**. Must be user-initiated (Start-camera button). No permission
   is requested until that click, matching the Phase-1 guarantee.

## Privacy posture
- No telemetry.
- No cloud OCR, no cloud sync.
- `.docsnap` references source files by canonical path; nothing is
  transmitted off-machine.
- Settings → Privacy enumerates the above in-app.
- Logs: `tracing_subscriber` defaults to `info`; document bodies and OCR
  text are **never** logged — only command names, durations, and
  non-sensitive error kinds.

## Known residual risks (tracked)
- Shared Windows account users can read the cache dir. Phase 22 adds an
  opt-in "encrypt cache with DPAPI" setting.
- A malicious PDF could stall `pdfium-render` for seconds before failing;
  CPU is bounded by the OS but UI responsiveness depends on the Tauri
  thread pool not blocking. Covered by moving PDFium calls into
  `tokio::task::spawn_blocking` (Phase 18 open opt).

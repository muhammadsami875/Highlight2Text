# Phase 3 — Image Import & Preview

## Delivered

### Rust (`apps/desktop/src-tauri`)
- `security.rs`: path canonicalization, 500 MB cap, magic-byte MIME sniffing (`infer`).
- `filesystem.rs`: platform-appropriate cache dir (`%LOCALAPPDATA%\DocSnap\…` on Windows).
- `image_processing.rs`: decode with the `image` crate (PNG/JPEG/TIFF/BMP/WebP), EXIF orientation applied in-pixels so downstream stages ignore the tag, SHA-256 content hash, cached JPEG preview (≤ 2048 px) + thumbnail (≤ 320 px).
- `commands.rs`: `import_image(path) -> ImportedPage | CommandError` with typed error union; validation rejects unsupported MIME types explicitly.
- `tauri-specta` wired: Rust command signatures export to `src/types/bindings.ts` on debug builds.

### Frontend
- `stores/projectStore.ts`: Zustand store, dedupes by content hash, tracks active page.
- `hooks/useDropTarget.ts`: subscribes to Tauri's native `onDragDropEvent` (the web `drop` event doesn't carry filesystem paths).
- `components/PageThumbnailStrip.tsx`: thumbnails via `convertFileSrc`.
- `pages/Editor.tsx`: three-pane layout — thumbnails, preview, metadata; shows mime, dimensions, file size, 16-char hash prefix; drag-and-drop overlay; collects per-file errors.
- `pages/Home.tsx`: Import tile wired; PDF / Scanner tiles explicitly disabled with phase labels.

### Configuration
- `tauri.conf.json`: `dragDropEnabled: true`, `assetProtocol` with a conservative scope (cache + user docs + pictures + downloads + desktop), CSP widened to allow `asset:` and `http(s)://asset.localhost` as `img-src` only.
- `capabilities/default.json`: scoped `fs:scope` matching the asset-protocol scope.

## Non-destructive guarantee
Originals are read-only from this point: imports copy nothing, write only derived artifacts into the cache, and refer to the source by canonical path + SHA-256.

## Known gaps (filled in later phases)
- No HEIF/HEIC decode in this phase; add via a feature-gated decoder once licensing is reviewed.
- No PDF import (Phase 11).
- No corner detection yet; preview is the full source image (Phase 4).
- `image` crate TIFF support is limited; large / BigTIFF files remain untested.

## Exit criteria
1. `cargo check -p docsnap` passes on `x86_64-pc-windows-msvc`.
2. `pnpm --filter docsnap-desktop typecheck` passes.
3. In `tauri dev`, dropping 5 mixed PNG/JPEG files on the window renders all 5 as thumbnails and the first fills the preview.
4. A rotated-by-EXIF JPEG renders in its visual orientation (tag ignored by consumers).
5. An invalid file (text renamed to `.jpg`) is rejected with a readable error row — no crash.

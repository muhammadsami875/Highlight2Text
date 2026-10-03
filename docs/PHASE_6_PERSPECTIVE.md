# Phase 6 — Perspective Correction

## Delivered

### Rust (`perspective.rs`)
- Target dims = `(max(top, bottom), max(left, right))` edge lengths of the
  quad, clamped to a 24 MP output budget to protect against ill-conditioned
  (near-collinear) corners.
- Direct linear homography solve: 4 point pairs → 8×8 system via Gauss-Jordan
  with partial pivoting; returns `None` on singular systems so the command
  surfaces a `degenerate quad` error instead of panicking.
- Inverse-sample warp with bilinear interpolation (RGB8).
- Preview written to the per-session cache, keyed by SHA-256 over
  `(source_path, corner coordinates)`, so the same recipe hits the cache on
  reload.
- Unit tests: identity homography round-trip, dimension selection.

### Command
`apply_perspective(path, corners[4]) -> WarpedPage { preview_path, width, height, warp_hash }`.

### Frontend
- `projectStore.warps` keyed by page id, stores `{ corners, warped }` —
  the **recipe**, not the pixel buffer; the pixels live in the cache file the
  Rust side wrote and the frontend references by `convertFileSrc`.
- Editor sidebar: Apply warp / Clear warp buttons. While `warp` is set, the
  preview swaps to the warped image and the `CropEditor` steps aside so the
  user isn't offered meaningless corner edits on an already-rectified page.

## Non-destructive guarantee
The warp is a recipe held in state. The original is untouched; cleaning the
cache invalidates derived files only. In Phase 16, projects persist the
recipe (corners + target dims), not the warped bytes.

## Deviations from Phase 1 doc
- OpenCV is still deferred; the pure-Rust solver is numerically fine for 4
  well-conditioned corners (which the Phase-5 self-intersection check and
  frame-clamp already guarantee). Keeps the Phase 1 vcpkg decision open.

## Exit criteria
1. `cargo check -p docsnap` passes.
2. `cargo test -p docsnap perspective` passes.
3. In `tauri dev`: on a photographed-at-angle document, Apply warp produces
   a rectified image whose horizontal lines in the result are visibly
   horizontal.
4. On a degenerate quad (three corners stacked), the backend returns the
   `degenerate quad` error and the Editor shows it in the issues box
   without crashing.
5. Clearing the warp returns to the live `CropEditor` and preserves the
   user's corner edits.

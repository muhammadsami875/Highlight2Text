# Phase 4 — Document Boundary Detection

## Delivered

### Rust pipeline (`document_detection.rs`)
1. Downscale to 1024 px long side.
2. Grayscale → Gaussian blur (σ = 1.4).
3. Canny with auto thresholds derived from the per-image median (×0.66 / ×1.33).
4. Dilate (L1 norm, 2 px) to close small edge gaps.
5. Contour trace via `imageproc::contours::find_contours`, outer borders only.
6. Ramer-Douglas-Peucker approximation at ε = 2% of perimeter.
7. Keep convex quads with area ≥ 20% of the frame.
8. Score each quad by `0.55·area_frac + 0.30·rectangularity + 0.15·centrality`.
9. Fallback: tight bounding rectangle of edge pixels, flagged `fallback = true`.
10. Order corners TL / TR / BR / BL by `(x+y)` and `(y−x)`.
11. Rescale back to source pixel coordinates.

Unit tests cover corner ordering and polygon area.

### Command
`detect_document_boundary(path) -> DetectedBoundary` with `corners[4]`, `confidence 0..1`, `fallback: bool`.

### Frontend
- `CropOverlay` SVG overlay with `viewBox = source dims`, so corner coordinates render correctly over the aspect-preserving preview `<img>`.
- Confidence badge in the overlay; amber styling when the detector fell back.
- `projectStore.boundaries` map keyed by page id.
- Editor sidebar "Auto-detect boundary" button; re-detection is one click.

## Deviations from Phase 1 doc
- OpenCV is deferred. The Phase 1 doc specifies OpenCV long-term; `imageproc`
  (pure Rust) keeps the Windows build dependency-free until the vcpkg /
  prebuilt / build-from-source decision in the Phase 1 checklist closes.
  The pipeline is isolated behind the module boundary, so the swap is local.
- No `ximgproc` StructuredEdges candidate yet (OpenCV-only); Canny is the single
  edge producer for now.

## Not yet
- Hough-line candidate as a third strategy.
- Multi-document detection (receipts side-by-side) — out of scope until Phase 7+.
- Manual corner dragging (Phase 5).

## Exit criteria
1. `cargo check -p docsnap` passes.
2. `cargo test -p docsnap document_detection` passes.
3. `pnpm --filter docsnap-desktop typecheck` passes.
4. In `tauri dev`, importing a photographed document and pressing Auto-detect
   draws a reasonable blue quad with ≥ 50% confidence.
5. Importing a plain screenshot (no document in frame) triggers the amber
   fallback with low confidence rather than throwing.

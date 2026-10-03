# Phase 5 — Manual Corner Adjustment

## Delivered

### `CropEditor` component
- SVG overlay with four draggable handles, each a large transparent hit-box
  (double the visible radius) above a visible dot.
- Pointer events use `setPointerCapture`, so a drag that leaves the handle
  keeps tracking until pointer-up.
- Client coordinates → source-pixel coordinates respects
  `preserveAspectRatio="xMidYMid meet"` letterboxing, so handle positions
  stay pinned under the cursor at any zoom.
- Corners are clamped to the source frame; drags that would make two
  non-adjacent edges intersect are rejected (segment-segment test).
- Fallback and confidence styling carried over from the view-only overlay.

### Project store (`projectStore.ts`)
`boundaries[pageId]` now has shape:
```
{ detected: DetectedBoundary | null,
  working:  DetectedBoundary | null,
  edited:   boolean }
```
- `setDetected` seeds both `detected` and `working` and clears `edited`.
- `setWorking` only mutates `working` and sets `edited = true`.
- `resetBoundary` restores `working := detected`.

### Editor UI
- Side panel: two-button row — Auto-detect / Re-detect, and Reset (enabled
  only when the user has edited).
- Status line reflects auto vs manual state and shows confidence/fallback.
- Apply (perspective warp) is called out as Phase 6.

## Invariants held after this phase
1. The frontend never persists a self-intersecting quad.
2. Corners are always in source-pixel coordinates (same frame as the Rust
   detector output), so downstream Rust stages don't need to know about the
   preview scale.
3. User edits are preserved across page switches via the store.

## Exit criteria
1. `pnpm --filter docsnap-desktop typecheck` passes.
2. Dragging a corner outside the image clamps to the edge.
3. Dragging a corner across the opposite edge is refused — the quad never
   turns into a bowtie.
4. Switching to another page and back keeps the user's adjustment.
5. Pressing Reset after an edit returns to the auto-detected corners and
   greys itself out.

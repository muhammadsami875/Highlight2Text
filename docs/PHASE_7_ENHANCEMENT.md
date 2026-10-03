# Phase 7 — Image Enhancement

## Delivered
- Rust `image_processing::apply_enhancement` with 7 presets (Original, Color,
  Auto, Document, Black & White, Grayscale, High Contrast) and four sliders
  (brightness, contrast, sharpness, shadow/background flattening). Each render
  is cached under a SHA-256 of `(source_path, params)`.
- Presets: grey-world WB, histogram-equalized luminance (CLAHE-like),
  adaptive threshold for documents, Otsu B&W, sigmoid S-curve.
- Shadow / background: large-sigma Gaussian estimates illumination; divide +
  renormalize to a near-white page.
- Command `apply_enhancement(path, params) → EnhancedPage`.
- Frontend: `EnhancementPanel` (preset chips + sliders), Editor applies the
  enhancement to the warped preview if a warp is active, otherwise the
  original. Changes debounced at 180 ms. Reset clears the recipe.

## Non-destructive
Enhancement is a recipe (preset + four scalars). The pixels live in the
preview cache; the project (Phase 16) stores the recipe only.

## Deviations
True CLAHE (local, tile-based) requires OpenCV; `equalize_histogram` is a
global proxy. Noted in Phase 1 verification carryover.

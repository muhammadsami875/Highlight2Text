# Phases 12 & 13 — PDF export + text/image export

## Phase 12 — Searchable PDF
- `pdf::export_pdf` composes an image-only PDF, or when `searchable=true`
  overlays an invisible text layer: each OCR word is drawn with the
  Helvetica built-in font, positioned by mapping its source-pixel bbox into
  the drawn image's PDF rectangle. Font size derives from box height.
- `PdfExportOptions`: page_size (A4/Letter/Legal/Original), margin
  (None/Small/Normal), quality (High/Balanced/Small — compression knob to
  be fully wired to printpdf's image encoders in Phase 18 perf), searchable.
- The chain used for both pixels and words: `enhanced → warped → original`,
  matching the OCR page's own precedence, so the overlay aligns with the
  pixels being drawn.

## Phase 13 — Text + image export
- `export.rs::write_text`: joins per-page OCR text with form-feed separators
  (U+000C), the standard "new page" byte that most text viewers respect.
- `export.rs::write_image`: PNG / JPEG / WebP / TIFF via the `image` crate;
  JPEG uses the user's quality slider through `JpegEncoder::new_with_quality`.
- Export page UI: PDF, Text, Images sections; validated per-section;
  success/error toast.

## Deviations
- CJK / RTL text layers still use Helvetica for Phase 12 — correct glyph
  shaping requires embedding Noto fonts (Phase 1 verification #7). Hidden
  text under Helvetica is still searchable for Latin scripts and selectable;
  Noto embedding lands with the final MSI build.
- DOCX / XLSX export deferred (the brief marks DOCX as "where available",
  explicitly cautioning against promising perfect layout; it will land as a
  tagged follow-up once a Rust `docx-rs` smoke test on a representative
  OCR page confirms acceptable output).

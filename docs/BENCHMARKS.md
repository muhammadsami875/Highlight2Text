# Benchmarks

Fill after each release on the Phase-1 reference box.

| Case                       | Wall time | Peak RSS | Notes |
|---                         |---        |---       |---    |
| Decode + cache 1080p PNG   |           |          |       |
| Decode + cache 4K JPEG     |           |          |       |
| Detect boundary 1080p      |           |          |       |
| Warp 1080p                 |           |          |       |
| OCR eng 1080p (document)   |           |          |       |
| OCR eng+deu 1080p          |           |          |       |
| OCR urd 1080p              |           |          |       |
| PDF import 10-page         |           |          |       |
| PDF import 50-page         |           |          |       |
| PDF export (image-only)    |           |          |       |
| PDF export (searchable)    |           |          |       |
| Batch 100 images, eng      |           |          |       |

Methodology:
- Hardware snapshot in the first row of the table.
- Three cold runs, three warm, record median.
- No background Tesseract or PDFium process; no antivirus live scan on the
  cache directory.
- `tracing=info` only. Debug symbols stripped for release-mode binaries.

# DocSnap — Third-party notices (seed)

Operators regenerate this file with the exact version strings at release
time (see `docs/PHASE_20_INSTALLER.md`, step 5).

| Component                | License         | Role in DocSnap |
|---                       |---              |---|
| Tauri 2                  | MIT / Apache-2  | Desktop shell |
| React, React Router      | MIT             | UI |
| TypeScript               | Apache-2        | UI build |
| Tailwind CSS             | MIT             | UI styling |
| Zustand                  | MIT             | UI state |
| Rust std + tokio         | MIT / Apache-2  | Backend runtime |
| `image`, `imageproc`     | MIT / Apache-2  | Decode + CV pipeline |
| `kamadak-exif`           | BSD-2           | EXIF orientation |
| `infer`                  | MIT             | Magic-byte MIME |
| `sha2`, `hex`            | MIT / Apache-2  | Content-addressed caching |
| `dirs`                   | MIT             | Platform paths |
| `uuid`                   | MIT / Apache-2  | Page IDs |
| `which`                  | MIT             | Dev-time Tesseract lookup |
| `serde`, `serde_json`    | MIT / Apache-2  | IPC / project files |
| `tracing`                | MIT             | Logging |
| `specta`, `tauri-specta` | MIT             | Rust↔TS type sharing |
| `pdfium-render`          | MIT / Apache-2  | PDF import glue |
| **PDFium** (binary)      | BSD-3           | PDF rendering |
| **Tesseract 5.x** (bin)  | Apache-2        | Local OCR engine |
| **Leptonica** (bin)      | BSD-2 style     | Tesseract dep |
| **tessdata_fast / *.traineddata** | per-pack | OCR language data |
| `printpdf`               | MIT             | PDF generation |
| `zip`                    | MIT             | `.docsnap` container |
| `base64`                 | MIT / Apache-2  | Capture decode |
| **Noto Sans / Arabic / CJK** (fonts) | SIL OFL 1.1 | Searchable PDF text layer |

Every bundled binary and dataset above is redistributable under its listed
license. See each project's own LICENSE file for the full text, placed under
`LICENSES/` in the installed bundle.

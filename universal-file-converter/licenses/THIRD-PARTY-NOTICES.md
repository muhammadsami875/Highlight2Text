# Third-Party Notices

This application uses the following third-party libraries and tools:

## Bundled Rust Dependencies (compiled into the binary)

| Crate | License | Usage |
|-------|---------|-------|
| tauri | MIT/Apache-2.0 | Application framework |
| serde | MIT/Apache-2.0 | Serialization |
| serde_json | MIT/Apache-2.0 | JSON parsing |
| uuid | MIT/Apache-2.0 | Unique identifiers |
| tokio | MIT | Async runtime |
| thiserror | MIT/Apache-2.0 | Error handling |
| log | MIT/Apache-2.0 | Logging |
| env_logger | MIT/Apache-2.0 | Log output |
| chrono | MIT/Apache-2.0 | Date/time |
| image | MIT/Apache-2.0 | Image processing |
| lopdf | MIT | PDF manipulation |
| infer | MIT | File type detection |
| zip | MIT | ZIP archive handling |
| calamine | MIT | Spreadsheet reading (XLSX/XLS/ODS) |
| rust_xlsxwriter | MIT/Apache-2.0 | XLSX writing |
| csv | MIT/Unlicense | CSV parsing |
| syntect | MIT | Syntax highlighting |
| pulldown-cmark | MIT | Markdown parsing |
| dirs-next | MIT/Apache-2.0 | Platform directories |
| open | MIT | Open files/URLs |

## Bundled Frontend Dependencies

| Package | License | Usage |
|---------|---------|-------|
| React | MIT | UI framework |
| Tailwind CSS | MIT | Styling |
| Zustand | MIT | State management |
| Vite | MIT | Build tooling |
| TypeScript | Apache-2.0 | Language |

## Optional External Engines (not bundled, user-installed)

| Engine | License | Usage | Notes |
|--------|---------|-------|-------|
| LibreOffice | MPL-2.0/LGPL-3.0 | Office document conversion | Used as external process only; not linked |
| Pandoc | GPL-2.0+ | Document conversion | Used as external process only; not linked |
| Poppler | GPL-2.0+ | PDF rendering | Used as external process only; not linked |
| Tesseract | Apache-2.0 | OCR | Used as external process only |
| FFmpeg | LGPL-2.1+ | Media conversion (future) | LGPL build only; used as external process |

### GPL Compliance Note

GPL-licensed tools (Pandoc, Poppler) are invoked as separate processes via
`std::process::Command`. They are never statically or dynamically linked into
this application's binary. This usage model is consistent with the GPL's
definition of a "separate work" and does not create a derivative work.

Users must install these tools separately. The application detects their
availability at runtime and gracefully degrades when they are not present.

## License Texts

The full license texts for all bundled dependencies can be found in the
Cargo.lock file and each crate's repository. Run `cargo license` to generate
a complete listing.

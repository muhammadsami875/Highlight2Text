# Universal File Converter — Phase 1: Architecture & Planning

## 1. System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri 2 Shell                        │
│  ┌───────────────────────┐  ┌────────────────────────┐  │
│  │   React/TS Frontend   │  │    Rust Backend Core    │  │
│  │                       │  │                         │  │
│  │  Pages:               │  │  Modules:               │  │
│  │   Home                │  │   commands/   (IPC)     │  │
│  │   Convert             │  │   conversion/ (core)    │  │
│  │   Batch               │  │   detectors/  (format)  │  │
│  │   History             │  │   planner/    (routing)  │  │
│  │   Favorites           │  │   engines/    (registry) │  │
│  │   Settings            │  │   process/    (runner)   │  │
│  │   About               │  │   jobs/       (queue)    │  │
│  │                       │  │   filesystem/ (I/O)      │  │
│  │  State: Zustand       │  │   security/   (sandbox)  │  │
│  │  IPC: Tauri invoke    │  │   history/    (DB)       │  │
│  │  i18n: key-based      │  │   settings/   (config)   │  │
│  └───────┬───────────────┘  └──────────┬─────────────┘  │
│          │        Tauri IPC            │                │
│          └─────────────────────────────┘                │
│                        │                                │
│              ┌─────────▼──────────┐                     │
│              │  Process Runner    │                     │
│              │  (child processes) │                     │
│              └─────────┬──────────┘                     │
│    ┌───────────┬───────┼───────┬───────────┐           │
│    ▼           ▼       ▼       ▼           ▼           │
│ LibreOffice  Pandoc  FFmpeg  Poppler  Tesseract        │
│ (headless)                   /pdf     (OCR)            │
└─────────────────────────────────────────────────────────┘
```

**Frontend**: React 18 + TypeScript + Vite, styled with Tailwind CSS. State via Zustand. All user-facing strings through i18n keys.

**Backend**: Rust (Tauri 2 backend). All conversion logic lives here. Frontend never touches files directly — everything goes through Tauri commands.

**IPC boundary**: Tauri's `invoke` system. Frontend sends typed commands, backend returns typed responses. Progress updates via Tauri events.

**External engines**: Spawned as child processes through `ProcessRunner`. Never via shell strings.

---

## 2. Conversion Engine Architecture

```
ConversionRequest {input_path, detected_format, target_format, options}
       │
       ▼
  InputDetector ──► validates file, detects true format
       │
       ▼
  CapabilityCheck ──► queries ConverterRegistry for supported routes
       │
       ▼
  ConversionPlanner ──► finds best path (direct or multi-step)
       │
       ▼
  EngineSelector ──► picks converter with highest priority for route
       │
       ▼
  Preprocessor ──► temp dir setup, input preparation
       │
       ▼
  ConversionExecutor ──► runs converter (may chain multiple steps)
       │
       ▼
  OutputValidator ──► structural validation of output file
       │
       ▼
  Postprocessor ──► rename, move to output dir, cleanup temps
       │
       ▼
  Result {status, output_path, warnings, duration}
```

### Converter trait (Rust)

```rust
pub trait Converter: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn input_formats(&self) -> &[FileFormat];
    fn output_formats(&self) -> &[FileFormat];
    fn can_convert(&self, from: FileFormat, to: FileFormat) -> bool;
    fn priority(&self, from: FileFormat, to: FileFormat) -> u8; // 0-255, higher = preferred
    fn platform_support(&self) -> &[Platform];
    fn is_available(&self) -> bool; // engine installed/bundled?
    fn convert(&self, request: &ConversionTask) -> Result<ConversionOutput, ConversionError>;
    fn cancel(&self, job_id: &str) -> Result<(), ConversionError>;
}
```

### ConverterRegistry

A `Vec<Box<dyn Converter>>` populated at startup. Methods:
- `find_direct(from, to) -> Vec<&dyn Converter>` — converters that handle this pair directly
- `find_chain(from, to) -> Vec<ConversionPlan>` — multi-step routes
- `best_route(from, to) -> Option<ConversionPlan>` — highest-scoring route
- `supported_outputs(from) -> Vec<FileFormat>` — all reachable outputs for an input format
- `is_supported(from, to) -> bool`

---

## 3. Format Detection Architecture

Detection runs three checks and reconciles:

| Check | Method | Confidence |
|-------|--------|-----------|
| Extension | Parse filename | Low |
| Magic bytes | Read first 16–8192 bytes, match signatures | High |
| MIME sniff | `infer` crate or custom matchers | Medium |

**Magic byte signatures (subset)**:
| Format | Signature |
|--------|-----------|
| PDF | `%PDF-` |
| ZIP (DOCX/XLSX/PPTX/ODS/ODT/ODP) | `PK\x03\x04` then check `[Content_Types].xml` or `mimetype` entry |
| PNG | `\x89PNG\r\n\x1a\n` |
| JPEG | `\xff\xd8\xff` |
| GIF | `GIF87a` / `GIF89a` |
| TIFF | `II\x2a\x00` / `MM\x00\x2a` |
| WEBP | `RIFF....WEBP` |
| BMP | `BM` |
| OLE2 (DOC/XLS/PPT) | `\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1` |
| RTF | `{\rtf` |
| SQLite | `SQLite format 3` |
| MP4 | `....ftyp` at offset 4 |
| MP3 | `\xff\xfb` / `\xff\xf3` / `ID3` |
| WAV | `RIFF....WAVE` |
| FLAC | `fLaC` |
| OGG | `OggS` |

**OpenXML disambiguation** (after ZIP detection):
- Read `[Content_Types].xml` inside the ZIP
- `word/` → DOCX
- `xl/` → XLSX
- `ppt/` → PPTX

**ODF disambiguation**:
- Read `mimetype` file inside the ZIP (uncompressed, first entry)
- `application/vnd.oasis.opendocument.text` → ODT
- `application/vnd.oasis.opendocument.spreadsheet` → ODS
- `application/vnd.oasis.opendocument.presentation` → ODP

**OLE2 disambiguation** (DOC/XLS/PPT):
- Use `cfb` or `ole` crate to read compound file
- Check for `WordDocument` stream → DOC
- Check for `Workbook` / `Book` stream → XLS
- Check for `PowerPoint Document` stream → PPT

**Mismatch handling**:
If extension says PDF but magic says PNG → return `FormatMismatch { extension_says, detected }`. Frontend shows warning, user decides.

**Text-based format detection** (no magic bytes):
For files that are plaintext: read first 4KB, check:
- `<!DOCTYPE html>` or `<html` → HTML
- Starts with `---` (YAML front matter) or has `# ` headings → Markdown (heuristic)
- Valid JSON parse → JSON
- Valid XML parse → XML
- CSV heuristic (consistent delimiter counts across lines) → CSV
- Otherwise → TXT

Fallback: extension-based guess, flagged as low-confidence.

---

## 4. Converter Plugin Architecture

Each converter is a Rust struct implementing the `Converter` trait. They're registered in a central manifest.

### Planned converters for MVP:

| Converter ID | Engine | Inputs | Outputs |
|-------------|--------|--------|---------|
| `libreoffice` | LibreOffice headless | DOC,DOCX,XLS,XLSX,PPT,PPTX,ODT,ODS,ODP,RTF,CSV,HTML,TXT | PDF,DOCX,XLSX,PPTX,ODT,ODS,ODP,HTML,TXT,CSV |
| `pandoc` | Pandoc | MD,HTML,DOCX,RST,LaTeX,TXT | DOCX,HTML,PDF,TXT,MD |
| `pdf_text` | `pdf-extract` / `lopdf` (Rust) | PDF | TXT |
| `pdf_image` | Poppler `pdftoppm`/`pdftocairo` | PDF | PNG,JPG |
| `pdf_table` | Tabula-java or `camelot`-equivalent | PDF | CSV,XLSX |
| `image_convert` | `image` crate (Rust) | PNG,JPG,WEBP,BMP,TIFF,GIF | PNG,JPG,WEBP,BMP,TIFF |
| `image_to_pdf` | `printpdf` / `lopdf` (Rust) | PNG,JPG,WEBP,BMP,TIFF | PDF |
| `code_to_pdf` | `syntect` → HTML → wkhtmltopdf or headless Chromium | PY,JS,TS,JAVA,CPP,C,CS,PHP,CSS,SQL,JSON,XML,RS,GO,RB | PDF,HTML |
| `html_to_pdf` | wkhtmltopdf or headless Chromium | HTML | PDF |
| `ocr` | Tesseract | PNG,JPG,TIFF,PDF(scanned) | TXT,searchable PDF |
| `ffmpeg` | FFmpeg | MP4,MKV,AVI,MOV,WEBM,MP3,WAV,AAC,FLAC,OGG,M4A | MP4,WEBM,MP3,WAV,AAC,FLAC,OGG |

### Manifest structure

```rust
pub struct ConverterManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub engine: EngineType,          // Bundled | External { executable }
    pub license: String,
    pub input_formats: Vec<FileFormat>,
    pub output_formats: Vec<FileFormat>,
    pub platforms: Vec<Platform>,     // Windows, macOS, Linux
    pub priority: u8,
    pub capabilities: Vec<Capability>, // OCR, TableExtraction, SyntaxHighlight, etc.
}
```

Adding a new converter: implement `Converter`, add to registry in `engines/mod.rs`. No UI changes needed — the format matrix auto-updates.

---

## 5. Conversion Planning Algorithm

```
fn plan(from: FileFormat, to: FileFormat, registry: &Registry) -> Option<ConversionPlan> {
    // 1. Try direct route
    if let Some(converters) = registry.find_direct(from, to) {
        return Some(Plan::Direct { converter: best_by_priority(converters) });
    }

    // 2. Try two-step route (BFS, depth=2)
    for intermediate in ALL_FORMATS {
        if let (Some(c1), Some(c2)) = (
            registry.find_direct(from, intermediate),
            registry.find_direct(intermediate, to)
        ) {
            candidates.push(Plan::Chain {
                steps: vec![
                    Step { from, to: intermediate, converter: best(c1) },
                    Step { from: intermediate, to, converter: best(c2) },
                ]
            });
        }
    }

    // 3. Try three-step route (for PDF→OCR→DOCX chains)
    // Same BFS with depth=3, only if no 2-step found

    // 4. Score candidates: quality * 3 + speed * 2 + reliability * 2 + priority
    // 5. Return highest-scoring plan, or None
}
```

**Scoring weights**:
- Quality (fidelity): 3x
- Speed: 2x
- Reliability (engine maturity): 2x
- Priority (from converter manifest): 1x

**Known multi-step routes (MVP)**:

| Input | Intermediate(s) | Output | Engines |
|-------|-----------------|--------|---------|
| MD | HTML | PDF | Pandoc → html_to_pdf |
| PY/JS/TS/etc. | HTML | PDF | syntect → html_to_pdf |
| PDF (scanned) | PNG | TXT | pdftoppm → Tesseract |
| PDF (scanned) | PNG → TXT | DOCX | pdftoppm → Tesseract → Pandoc |
| TXT | — | DOCX | Pandoc (direct) |
| TXT | — | PDF | LibreOffice (direct) or Pandoc→PDF |

---

## 6. Supported Format Matrix (MVP)

✅ = Supported | 🔶 = Experimental | ❌ = Not supported

**FROM ↓ / TO →**

| | PDF | DOCX | XLSX | PPTX | HTML | TXT | CSV | PNG | JPG | WEBP | ODT | ODS |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **PDF** | — | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| **DOCX** | ✅ | — | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ |
| **DOC** | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ |
| **XLSX** | ✅ | ❌ | — | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **XLS** | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **CSV** | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | — | ❌ | ❌ | ❌ | ❌ | ❌ |
| **PPTX** | ✅ | ❌ | ❌ | — | ❌ | 🔶 | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ |
| **PPT** | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **ODT** | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | — | ❌ |
| **ODS** | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | — |
| **ODP** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **RTF** | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **TXT** | ✅ | ✅ | ❌ | ❌ | ❌ | — | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **MD** | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **HTML** | ✅ | ✅ | ❌ | ❌ | — | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ |
| **PNG** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | — | ✅ | ✅ | ❌ | ❌ |
| **JPG** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | — | ✅ | ❌ | ❌ |
| **WEBP** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | — | ❌ | ❌ |
| **TIFF** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ |
| **BMP** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ |
| **PY/JS/TS/...** | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

---

## 7. Engine-to-Conversion-Family Mapping

| Conversion Family | Primary Engine | Fallback | Notes |
|---|---|---|---|
| Office → PDF | LibreOffice headless | — | Best fidelity for Office formats |
| Office → Office | LibreOffice headless | — | DOC→DOCX, XLS→XLSX, etc. |
| PDF → TXT | `lopdf`/`pdf-extract` (Rust) | Poppler `pdftotext` | Pure Rust preferred |
| PDF → Images | Poppler `pdftoppm` | `pdfium` | Poppler widely available |
| PDF → DOCX | `pdf-extract` + custom reconstruction | LibreOffice (limited) | Hard problem, best-effort |
| PDF → CSV/XLSX | Tabula-java | Custom table extractor | Tabula is gold standard for tables |
| Image → Image | `image` crate (Rust) | — | Pure Rust, no external dep |
| Image → PDF | `printpdf` (Rust) | — | Pure Rust |
| HTML → PDF | wkhtmltopdf | Headless Chromium | wkhtmltopdf simpler to bundle |
| MD → HTML/DOCX | Pandoc | `pulldown-cmark` (Rust) | Pandoc richer, Rust fallback |
| Code → PDF | `syntect` → HTML → wkhtmltopdf | — | Never executes code |
| OCR | Tesseract | — | Industry standard |
| Audio/Video | FFmpeg | — | Future phase |

---

## 8. PDF Conversion Strategy

### PDF → TXT
- Use `lopdf` to extract text streams per page
- Preserve reading order via content stream parsing
- Insert `--- PAGE N ---` markers between pages
- Handle encodings (WinAnsi, Unicode, Identity-H CMap)

### PDF → Images
- Use Poppler `pdftoppm` for rasterization
- Support DPI: 72, 150, 300, 600
- Output formats: PNG (default), JPEG, TIFF
- Per-page output: `page-001.png`, `page-002.png`

### PDF → DOCX
Pipeline:
1. Extract text with positions via `lopdf`
2. Detect paragraphs by vertical spacing
3. Detect headings by font size
4. Extract images from PDF resources
5. Detect tables by line/cell geometry (basic)
6. Build DOCX using `docx-rs` crate
7. Warn: "Layout is approximate. Complex PDFs may need manual adjustment."

For scanned PDFs (no text layer):
1. Detect: if text extraction yields <10 chars/page → likely scanned
2. Prompt: "Scanned PDF detected. Run OCR?"
3. Render pages to PNG at 300 DPI
4. OCR each page with Tesseract
5. Reconstruct document from OCR text

### PDF → XLSX/CSV
1. Use Tabula-java to detect table regions
2. Extract cell data per table
3. Build XLSX with `calamine` or `rust_xlsxwriter`
4. Multiple tables → multiple sheets or multiple CSV files
5. Warn: "Complex layouts may require manual correction."

### Searchable PDF (OCR overlay)
1. Render each page to image
2. OCR with Tesseract, get word positions
3. Overlay invisible text layer on original PDF using `lopdf`

---

## 9. Office Conversion Strategy

**Primary engine**: LibreOffice headless via CLI

```
soffice --headless --convert-to pdf --outdir /tmp/job-xxx input.docx
```

**Supported conversions via LibreOffice**:
- DOCX/DOC/ODT/RTF → PDF, DOCX, ODT, HTML, TXT
- XLSX/XLS/ODS/CSV → PDF, XLSX, ODS, CSV
- PPTX/PPT/ODP → PDF, PPTX, ODP, PNG (via PDF intermediate)

**Considerations**:
- LibreOffice is LGPL/MPL — can bundle on Windows if distributed properly
- Run with `--norestore --nofirststartwizard --nologo`
- One instance at a time (LibreOffice uses a user profile lock)
- Solution: use `--env:UserInstallation=file:///tmp/job-xxx-profile` per job
- Timeout: 120s default, configurable
- Font handling: bundle common fonts or rely on system fonts

---

## 10. Spreadsheet Conversion Strategy

### XLSX → CSV
- Use `calamine` crate (Rust, MIT) to read XLSX/XLS/ODS
- Allow sheet selection (all sheets, specific sheet)
- Handle: formulas (output computed values), dates, numbers, strings
- CSV encoding: UTF-8 with BOM option

### CSV → XLSX
- Parse CSV with `csv` crate (Rust)
- Auto-detect delimiter (comma, tab, semicolon, pipe)
- Build XLSX with `rust_xlsxwriter`
- Auto-size columns where possible

### XLSX → PDF
- Via LibreOffice headless
- Options: portrait/landscape, fit-to-page, sheet selection

### PDF → XLSX
- Tabula-java for table extraction
- Multiple tables → multiple sheets

---

## 11. HTML → PDF Strategy

**Primary engine**: wkhtmltopdf (LGPLv3)
- Bundled binary (~40MB on Windows)
- Renders WebKit, supports CSS, JS, images
- CLI: `wkhtmltopdf --page-size A4 --margin-top 10 input.html output.pdf`
- Supports: print CSS, page breaks, headers/footers, TOC

**Fallback**: Headless Chromium via Playwright/Puppeteer protocol
- Heavier (~200MB) but more modern CSS support
- Use only if wkhtmltopdf quality insufficient

**Security for HTML → PDF**:
- Local files only by default
- `--disable-javascript` option
- `--disable-external-links` option
- Block network requests unless user explicitly enables
- Sandbox the rendering process

---

## 12. Code → PDF Strategy

Pipeline:
```
source.py
  → syntect (Rust): tokenize + syntax highlight → HTML with inline CSS
  → wrap in HTML template:
      - monospace font (configurable)
      - line numbers (optional)
      - page title = filename
      - page numbers in footer
      - configurable font size, margins, line wrapping
  → wkhtmltopdf → output.pdf
```

**Language detection**: `syntect` uses TextMate grammar definitions. It supports 100+ languages. Detect by extension:

| Extension | Language |
|-----------|----------|
| .py | Python |
| .js | JavaScript |
| .ts | TypeScript |
| .java | Java |
| .cpp, .cc, .cxx | C++ |
| .c | C |
| .cs | C# |
| .php | PHP |
| .css | CSS |
| .sql | SQL |
| .json | JSON |
| .xml | XML |
| .rs | Rust |
| .go | Go |
| .rb | Ruby |
| .sh | Shell |
| .yaml, .yml | YAML |

**Options exposed to user**:
- Theme: light/dark (Monokai, Solarized, GitHub, VS Code-like)
- Font size: 8–16pt
- Line numbers: on/off
- Line wrapping: on/off
- Page size: A4/Letter/Legal
- Orientation: portrait/landscape
- Header: filename, date
- Footer: page numbers

**Security**: source files are read as raw bytes/text. Never compiled, interpreted, or executed.

---

## 13. Image Conversion Strategy

**Engine**: `image` crate (Rust, MIT/Apache-2.0)

Supported operations:
| From | To | Method |
|------|-----|--------|
| PNG → JPG | Decode PNG, encode JPEG with quality param | `image::ImageFormat` |
| PNG → WEBP | Decode PNG, encode WebP | `webp` crate |
| JPG → PNG | Decode JPEG, encode PNG | `image::ImageFormat` |
| JPG → WEBP | Decode, re-encode | `webp` crate |
| WEBP → PNG/JPG | Decode WebP, encode target | `webp` + `image` |
| TIFF → PNG/JPG | Decode, encode | `image` |
| BMP → PNG/JPG/WEBP | Decode, encode | `image` |
| Any image → PDF | Decode, embed in PDF page | `printpdf` |
| Multiple images → PDF | One image per page | `printpdf` |

**Options**:
- JPEG quality: 1–100 (default 85)
- WebP quality: 1–100 (default 80)
- PNG compression: 1–9 (default 6)
- Resize: optional width/height/percentage
- DPI: preserve or set

**Transparency**: warn when converting PNG (with alpha) → JPG. Offer background color selection (default white).

---

## 14. Media Conversion Strategy (Future Phase)

**Engine**: FFmpeg (LGPL/GPL depending on build)

**MVP scope**: Architecture only. No media UI until Phase 11+.

**Planned conversions**:
- MP4 → MP3 (extract audio)
- WAV → MP3
- MP3 → WAV
- MP4 → WEBM
- MOV → MP4
- MKV → MP4
- AVI → MP4
- FLAC → MP3
- OGG → MP3

**Bundling**: FFmpeg LGPL build (no GPL codecs) can be redistributed. Must build without GPL components or use system-installed FFmpeg.

**Not in MVP**: video editing, transcoding options, codec selection UI.

---

## 15. Security Architecture

### Threat Model

| Threat | Mitigation |
|--------|-----------|
| Malicious PDF | Render in sandboxed Poppler/LibreOffice; never execute embedded JS |
| Malicious Office doc | LibreOffice `--norestore --infilter` with macro execution disabled |
| Path traversal | Canonicalize all paths; reject `..`; validate within allowed dirs |
| Command injection | Never construct shell strings; use `std::process::Command` with arg arrays |
| Temp file race | Unique job UUIDs; create dirs with restrictive permissions |
| Resource exhaustion | File size limit (default 2GB); process timeout (default 300s); kill hung processes |
| Sensitive data leakage | Never log file contents; temp files deleted post-conversion |
| Arbitrary network access | HTML renderer: `--disable-external-links` by default |

### Process Isolation

```rust
pub struct ProcessRunner {
    executable: PathBuf,
    args: Vec<String>,        // never from shell expansion
    working_dir: PathBuf,
    env: HashMap<String, String>,
    timeout: Duration,
    stdin: Option<Vec<u8>>,
    stdout_capture: bool,
    stderr_capture: bool,
}

impl ProcessRunner {
    pub fn run(&self) -> Result<ProcessOutput, ProcessError>;
    pub fn run_with_cancel(&self, cancel: CancellationToken) -> Result<ProcessOutput, ProcessError>;
}
```

Rules:
1. `Command::new(executable).args(&self.args)` — never `.arg(format!(...))` with user input
2. Working directory always inside temp job folder
3. Environment is explicit whitelist, not inherited
4. Timeout enforced; process killed on expiry
5. Child processes tracked; killed on app exit

### File Path Sanitization

```rust
fn sanitize_path(input: &str) -> Result<PathBuf, SecurityError> {
    let path = PathBuf::from(input);
    let canonical = path.canonicalize()?;
    // Reject if outside allowed directories
    if !canonical.starts_with(&allowed_base) {
        return Err(SecurityError::PathTraversal);
    }
    Ok(canonical)
}
```

### LibreOffice Hardening
- `--norestore` — no recovery dialogs
- `--nofirststartwizard` — no interactive prompts
- `--nologo` — no splash
- Disable macros: `--infilter="writer_pdf_Export"` or macro security level
- Separate user profile per job: prevents cross-job state leakage

---

## 16. Temporary File Architecture

```
{app_data}/temp/
  └── jobs/
      └── {uuid}/
          ├── input/           # copied/linked input file
          ├── intermediate/    # intermediate conversion artifacts
          ├── output/          # final output before move
          └── logs/            # converter stdout/stderr
```

### Lifecycle
1. **Create**: `jobs/{uuid}/` created at job start
2. **Populate**: input file copied (not moved) into `input/`
3. **Convert**: engines write to `intermediate/` and `output/`
4. **Finalize**: validated output moved to user's chosen destination
5. **Cleanup**: entire `jobs/{uuid}/` deleted
6. **Crash recovery**: on startup, scan `jobs/` for abandoned folders older than 1 hour → prompt user or auto-delete

### Rules
- UUID v4 for job IDs — never predictable
- Directories created with `0o700` permissions on Unix
- Input files copied, never modified in-place
- Temp folder location configurable in settings
- Total temp usage monitored; warn at configurable threshold

---

## 17. Output Validation Architecture

Every converter's output goes through format-specific validation before marking complete:

| Format | Validation |
|--------|-----------|
| PDF | Parse with `lopdf`: valid xref table, page count > 0, file size > 0 |
| DOCX | Valid ZIP; contains `[Content_Types].xml` and `word/document.xml` |
| XLSX | Valid ZIP; contains `[Content_Types].xml` and `xl/workbook.xml` |
| PPTX | Valid ZIP; contains `[Content_Types].xml` and `ppt/presentation.xml` |
| PNG | Decode first bytes: valid PNG header + IHDR chunk |
| JPG | Decode: valid JPEG markers (SOI, SOF, EOI) |
| WEBP | Valid RIFF/WEBP header |
| TXT | Non-empty; valid UTF-8 (or declared encoding) |
| CSV | Non-empty; parseable with consistent column count |
| HTML | Non-empty; contains `<html` or `<!DOCTYPE` |
| MP4 | Valid ftyp box (future) |

```rust
pub trait OutputValidator {
    fn validate(&self, path: &Path, format: FileFormat) -> ValidationResult;
}

pub enum ValidationResult {
    Valid { metadata: OutputMetadata },
    ValidWithWarnings { metadata: OutputMetadata, warnings: Vec<String> },
    Invalid { reason: String },
}
```

---

## 18. Dependency & Licensing Analysis

| Dependency | Version (current) | License | Bundleable? | Size | Notes |
|---|---|---|---|---|---|
| **Tauri 2** | 2.x | MIT/Apache-2.0 | ✅ Yes | ~5MB runtime | Core framework |
| **React 18** | 18.x | MIT | ✅ Yes | ~140KB | Frontend |
| **Tailwind CSS** | 3.x | MIT | ✅ Yes | Build-time only | Styling |
| **Zustand** | 4.x | MIT | ✅ Yes | ~3KB | State management |
| **LibreOffice** | 24.x | MPL 2.0 / LGPL 3 | ⚠️ Conditional | ~400MB | Must comply with MPL: provide source or link. Can bundle if license terms met. Consider requiring user-installed. |
| **Pandoc** | 3.x | GPL 2+ | ⚠️ GPL | ~100MB | GPL: bundling in proprietary app problematic. Options: (a) keep app FOSS, (b) require user-installed, (c) use as separate process (GPL permits this for some interpretations) |
| **FFmpeg** | 7.x | LGPL 2.1+ (LGPL build) | ⚠️ Conditional | ~80MB | LGPL build redistributable. Must not include GPL codecs. Dynamic linking preferred. |
| **Poppler** | 24.x | GPL 2+ | ⚠️ GPL | ~15MB | Same GPL concern as Pandoc. Alternative: `pdfium` (BSD-3) or Rust PDF libs |
| **wkhtmltopdf** | 0.12.6 | LGPL 3 | ⚠️ LGPL | ~40MB | Redistributable under LGPL terms. Project is unmaintained — evaluate alternatives. |
| **Tesseract** | 5.x | Apache 2.0 | ✅ Yes | ~30MB + data | Freely redistributable |
| **Tabula-java** | 1.x | MIT | ✅ Yes | ~15MB (needs JRE) | Needs JRE bundled or installed — significant concern |
| **`image` crate** | 0.25.x | MIT/Apache-2.0 | ✅ Yes | Compiled in | Pure Rust |
| **`lopdf`** | 0.33.x | MIT | ✅ Yes | Compiled in | Pure Rust PDF |
| **`printpdf`** | 0.7.x | MIT | ✅ Yes | Compiled in | Pure Rust PDF creation |
| **`syntect`** | 5.x | MIT | ✅ Yes | Compiled in | Syntax highlighting |
| **`calamine`** | 0.26.x | MIT | ✅ Yes | Compiled in | Read XLS/XLSX/ODS |
| **`rust_xlsxwriter`** | 0.80.x | MIT/Apache-2.0 | ✅ Yes | Compiled in | Write XLSX |
| **`csv`** | 1.3.x | MIT/Unlicense | ✅ Yes | Compiled in | CSV parsing |
| **`infer`** | 0.16.x | MIT | ✅ Yes | Compiled in | MIME/magic detection |
| **`docx-rs`** | 0.4.x | MIT | ✅ Yes | Compiled in | DOCX creation |
| **`pulldown-cmark`** | 0.12.x | MIT | ✅ Yes | Compiled in | Markdown parsing |

### Licensing Decisions Required Before Phase 2

1. **GPL tools (Poppler, Pandoc)**: Use as external processes (not linked). Ship app under permissive license. Require user to install these separately, or bundle with GPL compliance (provide source, GPL notice). **Recommended**: detect if installed; prompt user to install if missing; provide download links.

2. **LibreOffice**: Same approach — detect system installation. On Windows, check `C:\Program Files\LibreOffice`. Provide setup wizard to locate or download.

3. **Tabula-java**: Requires JRE. **Alternative**: use `pdfplumber`-equivalent in Rust or Python, or build a Rust table extractor. **Recommended**: defer to Phase 8; evaluate pure-Rust options first.

4. **wkhtmltopdf**: Unmaintained. **Alternative**: headless Chromium (via Tauri's WebView or bundled). **Recommended**: start with wkhtmltopdf for simplicity, plan Chromium migration.

5. **FFmpeg**: LGPL build is safe. Bundle LGPL-only build or detect system install.

---

## 19. Monorepo Structure

```
universal-file-converter/
├── apps/
│   └── desktop/
│       ├── src/                          # React frontend
│       │   ├── components/
│       │   │   ├── common/               # Button, Input, Modal, Card, etc.
│       │   │   ├── conversion/           # ConvertPanel, FormatSelector, OptionsPanel
│       │   │   ├── batch/                # BatchQueue, BatchProgress
│       │   │   ├── file/                 # FileCard, FileDrop, FileInfo
│       │   │   ├── history/              # HistoryList, HistoryItem
│       │   │   ├── preview/              # PreviewPanel, PdfPreview, ImagePreview
│       │   │   └── settings/             # SettingsPanel, SettingsGroup
│       │   ├── pages/
│       │   │   ├── HomePage.tsx
│       │   │   ├── ConvertPage.tsx
│       │   │   ├── BatchPage.tsx
│       │   │   ├── HistoryPage.tsx
│       │   │   ├── FavoritesPage.tsx
│       │   │   ├── SettingsPage.tsx
│       │   │   └── AboutPage.tsx
│       │   ├── hooks/
│       │   │   ├── useConversion.ts
│       │   │   ├── useFileDetection.ts
│       │   │   ├── useBatchQueue.ts
│       │   │   └── useSettings.ts
│       │   ├── stores/
│       │   │   ├── conversionStore.ts
│       │   │   ├── batchStore.ts
│       │   │   ├── historyStore.ts
│       │   │   └── settingsStore.ts
│       │   ├── services/
│       │   │   ├── ipc.ts                # Tauri invoke wrappers
│       │   │   └── formatMatrix.ts       # Client-side format data
│       │   ├── types/
│       │   │   ├── conversion.ts
│       │   │   ├── formats.ts
│       │   │   └── settings.ts
│       │   ├── utils/
│       │   │   ├── fileSize.ts
│       │   │   └── formatters.ts
│       │   ├── i18n/
│       │   │   ├── en.json
│       │   │   └── index.ts
│       │   ├── styles/
│       │   │   └── globals.css
│       │   ├── App.tsx
│       │   └── main.tsx
│       ├── src-tauri/
│       │   ├── src/
│       │   │   ├── main.rs
│       │   │   ├── lib.rs
│       │   │   ├── commands/             # Tauri IPC command handlers
│       │   │   │   ├── mod.rs
│       │   │   │   ├── convert.rs
│       │   │   │   ├── detect.rs
│       │   │   │   ├── batch.rs
│       │   │   │   ├── history.rs
│       │   │   │   ├── settings.rs
│       │   │   │   └── system.rs
│       │   │   ├── conversion/           # Core conversion logic
│       │   │   │   ├── mod.rs
│       │   │   │   ├── request.rs
│       │   │   │   ├── job.rs
│       │   │   │   ├── result.rs
│       │   │   │   └── error.rs
│       │   │   ├── detectors/            # Format detection
│       │   │   │   ├── mod.rs
│       │   │   │   ├── magic.rs
│       │   │   │   ├── extension.rs
│       │   │   │   ├── mime.rs
│       │   │   │   └── reconcile.rs
│       │   │   ├── planner/              # Conversion path planning
│       │   │   │   ├── mod.rs
│       │   │   │   ├── graph.rs
│       │   │   │   └── scoring.rs
│       │   │   ├── engines/              # Converter registry
│       │   │   │   ├── mod.rs
│       │   │   │   ├── registry.rs
│       │   │   │   └── manifest.rs
│       │   │   ├── converters/           # Individual converter implementations
│       │   │   │   ├── mod.rs
│       │   │   │   ├── libreoffice.rs
│       │   │   │   ├── pandoc.rs
│       │   │   │   ├── pdf_text.rs
│       │   │   │   ├── pdf_image.rs
│       │   │   │   ├── pdf_table.rs
│       │   │   │   ├── image_convert.rs
│       │   │   │   ├── image_to_pdf.rs
│       │   │   │   ├── code_to_pdf.rs
│       │   │   │   ├── html_to_pdf.rs
│       │   │   │   ├── ocr.rs
│       │   │   │   └── ffmpeg.rs
│       │   │   ├── process/              # External process runner
│       │   │   │   ├── mod.rs
│       │   │   │   ├── runner.rs
│       │   │   │   ├── timeout.rs
│       │   │   │   └── cleanup.rs
│       │   │   ├── filesystem/           # File I/O, temp dirs
│       │   │   │   ├── mod.rs
│       │   │   │   ├── temp.rs
│       │   │   │   ├── paths.rs
│       │   │   │   └── naming.rs
│       │   │   ├── security/             # Sanitization, validation
│       │   │   │   ├── mod.rs
│       │   │   │   ├── sanitize.rs
│       │   │   │   └── limits.rs
│       │   │   ├── validators/           # Output validation
│       │   │   │   ├── mod.rs
│       │   │   │   ├── pdf.rs
│       │   │   │   ├── office.rs
│       │   │   │   ├── image.rs
│       │   │   │   ├── text.rs
│       │   │   │   └── media.rs
│       │   │   ├── jobs/                 # Job queue, state machine
│       │   │   │   ├── mod.rs
│       │   │   │   ├── queue.rs
│       │   │   │   ├── state.rs
│       │   │   │   └── worker.rs
│       │   │   ├── history/              # Conversion history DB
│       │   │   │   ├── mod.rs
│       │   │   │   └── db.rs
│       │   │   └── settings/             # App settings
│       │   │       ├── mod.rs
│       │   │       └── config.rs
│       │   ├── Cargo.toml
│       │   ├── tauri.conf.json
│       │   └── build.rs
│       ├── package.json
│       ├── tsconfig.json
│       ├── vite.config.ts
│       └── tailwind.config.js
├── docs/
│   ├── PHASE-1-ARCHITECTURE.md
│   ├── CONVERSION-MATRIX.md
│   └── SECURITY.md
├── tests/
│   ├── fixtures/                        # Test input files
│   └── integration/                     # Conversion integration tests
├── scripts/
│   ├── bundle-engines.sh               # Script to prepare external engines
│   └── check-dependencies.sh
├── licenses/
│   └── THIRD-PARTY-NOTICES.md
└── README.md
```

---

## 20. Development Roadmap

| Phase | Scope | Estimated Effort | Dependencies |
|-------|-------|-----------------|-------------|
| **1** | Architecture & planning (this document) | Done | — |
| **2** | Tauri + React + TS + Rust project scaffold | 1 session | Phase 1 |
| **3** | Format detection (magic bytes, extension, MIME) | 1 session | Phase 2 |
| **4** | Converter trait, registry, engine abstraction | 1 session | Phase 2 |
| **5** | ProcessRunner, secure child process management | 1 session | Phase 4 |
| **6** | LibreOffice converter: DOCX/XLSX/PPTX ↔ PDF | 1-2 sessions | Phase 5 |
| **7** | PDF text/image extraction (lopdf, Poppler) | 1 session | Phase 5 |
| **8** | PDF → Excel/CSV table extraction | 1 session | Phase 7 |
| **9** | HTML → PDF (wkhtmltopdf) | 1 session | Phase 5 |
| **10** | Code/Markdown/Text → PDF (syntect, Pandoc) | 1 session | Phase 9 |
| **11** | Image conversions + Image → PDF | 1 session | Phase 4 |
| **12** | Batch conversion queue, parallel workers | 1 session | Phase 6-11 |
| **13** | Conversion planner: multi-step route planning | 1 session | Phase 12 |
| **14** | Preview panel, output comparison | 1 session | Phase 12 |
| **15** | History DB, favorites, settings persistence | 1 session | Phase 2 |
| **16** | CLI interface (shares backend) | 1 session | Phase 12 |
| **17** | Performance benchmarks, optimization | 1 session | Phase 12 |
| **18** | Security hardening, audit | 1 session | Phase 17 |
| **19** | Test suite (unit + integration + fixtures) | 1-2 sessions | Phase 12 |
| **20** | Windows installer (Tauri bundler / NSIS / WiX) | 1 session | Phase 19 |
| **21** | Final license audit, third-party notices | 1 session | Phase 20 |

---

## 21. Testing Strategy

### Unit Tests (Rust)
- Format detection: test each magic byte signature
- Path sanitization: traversal attempts, Unicode edge cases
- Conversion planning: graph traversal correctness
- Job state machine: valid transitions only
- File naming: collision avoidance, suffix generation

### Integration Tests
- Each converter: real input file → convert → validate output
- Multi-step chains: MD → HTML → PDF, PY → HTML → PDF
- Unsupported route detection: PDF → EXE returns error, not fake success
- Format mismatch: file with wrong extension
- Large file handling: 50MB+ PDF, 100+ page DOCX
- Concurrent batch: 10 parallel conversions

### Test Fixtures
Create minimal valid test files:
- `test.pdf` — 3 pages, text + table + image
- `test.docx` — headings, paragraphs, table, image, Unicode (English + Urdu + Arabic)
- `test.xlsx` — multiple sheets, formulas, dates, numbers
- `test.pptx` — 5 slides, text + images
- `test.csv` — 1000 rows, various data types
- `test.md` — headings, code blocks, links, images
- `test.html` — CSS, images, tables
- `test.py` — Python with various syntax elements
- `test.png`, `test.jpg`, `test.webp`, `test.tiff` — various dimensions
- `corrupt.pdf` — truncated file for error handling
- `wrong-ext.pdf` — actually a PNG named .pdf

### Fidelity Checks
- PDF → TXT: compare extracted text against known content
- DOCX → PDF: verify page count matches
- XLSX → CSV: verify row/column counts
- Image → Image: verify dimensions preserved

### Performance Benchmarks
- Small file (<1MB): target <5s
- Medium file (1-10MB): target <30s
- Large file (10-100MB): target <120s
- Batch of 10: target <60s (parallel)

---

## 22. Performance Strategy

### Startup
- Lazy engine detection: check availability on first use, cache result
- Converter registry: static initialization, no I/O at startup
- UI: skeleton loading, progressive rendering

### Conversion
- Stream processing where possible (don't load entire file into memory)
- Parallel batch workers: default `min(4, cpu_count / 2)`
- LibreOffice instance pooling: pre-warm one instance, reuse with separate profiles
- Cancel support: kill child process immediately on cancel

### Memory
- Image processing: stream decode/encode, don't hold decoded bitmap longer than needed
- PDF processing: page-by-page, not entire document
- Batch: limit concurrent jobs to prevent OOM

### Disk
- Temp cleanup immediately after each job
- Monitor temp folder size; warn at 1GB
- Compress intermediate files only if conversion takes >30s and disk is low

### Caching
- Engine availability: cached per session
- Format detection results: cached per file path + modification time
- Conversion plans: cached per (input_format, output_format) pair

---

## Pre-Phase-2 Verification Checklist

Before starting Phase 2, verify:

- [ ] **Rust toolchain**: `rustup` with stable Rust installed
- [ ] **Node.js**: v18+ with npm/pnpm
- [ ] **Tauri 2 CLI**: `cargo install tauri-cli --version ^2`
- [ ] **Tauri prerequisites**: WebView2 (Windows), system deps (Linux)
- [ ] **LibreOffice**: installed and `soffice --version` works (for Phase 6)
- [ ] **Pandoc**: installed and `pandoc --version` works (for Phase 10)
- [ ] **wkhtmltopdf**: binary available (for Phase 9)
- [ ] **Tesseract**: installed with language data (for OCR phases)
- [ ] **Poppler**: `pdftoppm`/`pdftotext` available (for Phase 7)
- [ ] **FFmpeg**: LGPL build available (for future media phase)
- [ ] **Licensing review**: confirmed approach for GPL deps (external process, not linked)
- [ ] **Crate availability**: all listed Rust crates exist at specified versions on crates.io
- [ ] **Target platform**: Windows 10/11 build environment available

Items 5-10 are needed for later phases, not Phase 2 itself. Phase 2 only needs Rust, Node.js, and Tauri CLI.

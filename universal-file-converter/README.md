# Universal File Converter

A professional desktop file conversion application built with **Tauri 2** (Rust backend + React/TypeScript frontend). Converts between 40+ format pairs across documents, images, spreadsheets, code, and web formats — all locally, with no cloud uploads.

## Features

- **40+ conversion routes** across documents, images, spreadsheets, presentations, code, and web formats
- **Multi-step conversion planning** — automatically chains converters when no direct route exists
- **Batch processing** — convert multiple files at once with per-item progress tracking
- **Format auto-detection** — magic bytes, extension mapping, MIME sniffing, CSV/Markdown heuristics
- **Conversion history & favorites** — track past conversions and save frequent format pairs
- **Dark/Light/System themes** — follows OS preference or manual override
- **Keyboard shortcuts** — `Ctrl+O` open file, `Ctrl+B` batch mode, `Ctrl+,` settings
- **Security-first design** — path sanitization, file size limits, process timeouts, no code execution during conversion

## Supported Formats

| Category       | Formats                                                        |
| -------------- | -------------------------------------------------------------- |
| Images         | PNG, JPG, WebP, BMP, TIFF, GIF, ICO                           |
| Documents      | PDF, DOCX, DOC, ODT, RTF, TXT                                 |
| Spreadsheets   | XLSX, XLS, ODS, CSV, TSV                                       |
| Presentations  | PPTX, PPT, ODP                                                 |
| Code           | Python, JavaScript, TypeScript, Java, C/C++, Go, Ruby, PHP, Rust, Swift, Kotlin, Shell |
| Web            | HTML, CSS, JSON, XML, SQL                                      |
| Markup         | Markdown                                                       |

## Architecture

```
apps/desktop/
├── src/                    # React frontend
│   ├── components/         # UI components (DropZone, FormatSelector, etc.)
│   ├── hooks/              # Custom hooks (useConversion, useFileDetection, useTheme)
│   ├── pages/              # 7 app pages (Home, Convert, Batch, History, Favorites, Settings, About)
│   ├── stores/             # Zustand state management
│   ├── services/           # IPC bridge and format matrix
│   └── types/              # TypeScript type definitions
├── src-tauri/
│   └── src/
│       ├── commands/       # 16 Tauri IPC command handlers
│       ├── converters/     # 11 converter implementations
│       ├── detectors/      # Format detection (magic bytes, extension, reconciliation)
│       ├── engines/        # External engine registry and manifest
│       ├── planner/        # BFS-based multi-step conversion planner
│       ├── jobs/           # Async job queue and worker
│       ├── process/        # ProcessRunner for external tools
│       ├── validators/     # Output format validation
│       ├── security/       # Path sanitization, filename safety
│       ├── settings/       # User preferences persistence
│       └── history/        # Conversion history storage
```

## Conversion Engines

| Engine            | Technology                | Handles                                      |
| ----------------- | ------------------------- | -------------------------------------------- |
| ImageConverter    | `image` crate             | PNG, JPG, WebP, BMP, TIFF, GIF, ICO          |
| ImageToPdf        | `lopdf`                   | Any image → PDF                              |
| MarkdownToHtml    | `pulldown-cmark`          | Markdown → HTML (GFM)                        |
| HtmlToPdf         | Pure Rust renderer        | HTML/CSS → PDF                               |
| CsvJsonConverter  | Built-in parser           | CSV ↔ JSON                                   |
| CsvToXlsx         | `rust_xlsxwriter`         | CSV → XLSX                                   |
| XlsxExtractor     | `calamine`                | XLSX/XLS/ODS → CSV/JSON                      |
| CodeHighlighter   | `syntect`                 | Source code → syntax-highlighted HTML         |
| TextToPdf         | `lopdf`                   | Text/code (25+ types) → PDF                  |
| PdfExtractor      | `lopdf`                   | PDF → plain text                             |
| LibreOffice       | External process          | Office document conversions                  |

## Tech Stack

**Frontend:** React 18, TypeScript, Vite 5, Tailwind CSS 3, Zustand 4, React Router 6, Lucide icons

**Backend:** Rust, Tauri 2, tokio (async runtime), serde (serialization)

**Key Rust crates:** `image` 0.25, `lopdf` 0.34, `calamine` 0.26, `rust_xlsxwriter` 0.80, `syntect` 5, `pulldown-cmark` 0.12

**Testing:** Vitest (frontend, 51 tests), Cargo test (backend, 109 tests)

## Installation

### Download Pre-built Packages

Download the latest release for your platform from the [Releases](https://github.com/muhammadsami875/Highlight2Text/releases) page:

| Platform | Package | Install Command |
| -------- | ------- | --------------- |
| Ubuntu/Debian | `Universal File Converter_0.1.0_amd64.deb` | `sudo dpkg -i Universal\ File\ Converter_0.1.0_amd64.deb` |
| Fedora/RHEL | `Universal File Converter-0.1.0-1.x86_64.rpm` | `sudo rpm -i Universal\ File\ Converter-0.1.0-1.x86_64.rpm` |
| Any Linux | `Universal File Converter_0.1.0_amd64.AppImage` | `chmod +x *.AppImage && ./Universal\ File\ Converter_0.1.0_amd64.AppImage` |
| Windows | `Universal File Converter_0.1.0_x64-setup.nsis.exe` | Run the installer |
| macOS | `Universal File Converter_0.1.0_aarch64.dmg` | Open the DMG and drag to Applications |

### Uninstall

```bash
# Debian/Ubuntu
sudo dpkg -r universal-file-converter

# Fedora/RHEL
sudo rpm -e universal-file-converter
```

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) (1.77+ stable)
- [Node.js](https://nodejs.org/) 18+
- System dependencies (Linux):
  ```bash
  # Ubuntu/Debian
  sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev librsvg2-dev libsoup-3.0-dev libssl-dev patchelf

  # Fedora
  sudo dnf install gtk3-devel webkit2gtk4.1-devel librsvg2-devel libsoup3-devel openssl-devel
  ```

### Development

```bash
cd apps/desktop
npm install
npm run tauri:dev
```

### Build Installers

```bash
cd apps/desktop
npm install

# Build all packages for your platform
npm run tauri:build

# Or build a specific package format
npm run tauri:build:deb       # Debian/Ubuntu .deb
npm run tauri:build:rpm       # Fedora/RHEL .rpm
npm run tauri:build:appimage  # Portable AppImage
```

Output packages are in `src-tauri/target/release/bundle/`.

### Testing

```bash
cd apps/desktop

# Frontend tests (51 tests)
npm test

# Backend tests (109 tests)
cd src-tauri && cargo test
```

## Security

- All file paths are canonicalized and validated against traversal attacks
- Filenames are sanitized (dangerous characters stripped, length enforced)
- File size limit: 2 GB per file
- External processes run with typed arguments (no shell command strings)
- Process timeouts enforced via tokio
- Output files are validated for format correctness before delivery
- Source code is never executed during conversion

## License

See [THIRD-PARTY-NOTICES.md](licenses/THIRD-PARTY-NOTICES.md) for third-party license information.

# Phase 2 — Project Scaffold

This phase sets up the Tauri 2 + React + TypeScript + Rust skeleton that the
remaining phases fill in. No business logic ships here.

## What exists after Phase 2

### Monorepo (pnpm + Cargo workspaces)
```
/package.json           pnpm workspace root
/pnpm-workspace.yaml
/Cargo.toml             Cargo workspace, shared deps pinned
/apps/desktop/          the Tauri app
/packages/              shared TS/Rust packages (empty for now)
/docs/                  design + phase docs
```

### Frontend (`apps/desktop`)
- Vite + React 18 + TypeScript strict + Tailwind + React Router.
- Shell with sidebar nav and pages: Home, Scanner, Editor, OCR, Documents, Settings.
- `services/ipc.ts`: typed Tauri `invoke` wrapper (single command so far).
- `stores/settingsStore.ts`: Zustand store, theme toggle.
- OCR page calls `get_app_info` so the IPC round-trip is smoke-testable end-to-end.

### Rust backend (`apps/desktop/src-tauri`)
- `lib.rs` wires `tracing`, Tauri dialog + fs plugins, and the handler list.
- One command: `get_app_info` → `{ name, version, offline, ocr_engine }`.
- Module stubs for every subsystem from the Phase 1 doc (`ocr`, `pdf`, `camera`,
  `document_detection`, `perspective`, `image_processing`, `projects`,
  `export`, `filesystem`, `security`, `settings`, `commands`).
- `capabilities/default.json` with the narrowest viable permission set.
- CSP in `tauri.conf.json` locks `connect-src` to IPC only — no network.

### Resources
- Empty `resources/{tessdata, binaries, fonts}/` with `.gitkeep` — populated in
  later phases (traineddata, Tesseract sidecar, Noto fonts) once licensing is
  confirmed per the Phase 1 checklist.

## Running (once dependencies are installed on a Windows dev box)
```powershell
pnpm install
pnpm --filter docsnap-desktop tauri dev
```

## Not in this phase
- No OpenCV, PDFium, Tesseract, nokhwa, or specta wiring yet.
- No file import, no detection, no OCR, no export — those are their own phases.
- No bundler icons yet (`icons/.gitkeep`); a Phase 20 task.

## Exit criteria for Phase 2
1. `pnpm install` succeeds on a Windows 11 dev box.
2. `pnpm --filter docsnap-desktop tauri dev` opens the DocSnap window and the
   OCR page renders the `get_app_info` payload (proof the IPC bridge works).
3. `pnpm --filter docsnap-desktop typecheck` passes with `--strict`.
4. `cargo check -p docsnap` passes on `stable-x86_64-pc-windows-msvc`.
5. Phase 1 verification checklist items 1 (Tauri 2 schema) and 6 (specta
   decision) are revisited — specta is in `Cargo.toml` but not yet wired; its
   introduction is Phase 3's first task so IPC types are shared from the moment
   real commands land.

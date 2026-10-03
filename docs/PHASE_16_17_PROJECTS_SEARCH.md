# Phases 16 & 17 — Project store + search

## Phase 16 — `.docsnap`
- ZIP container with `project.json` + `ocr/<pageId>.json`. Deflate.
- `Project` carries schema version (gated at load time), timestamps, and
  per-page recipes: rotation, corners, warp target dims, enhancement
  params, OCR result. No pixel bytes are persisted; renders are a
  pure function of recipe + source hash.
- Atomic save: write to `.docsnap.tmp`, then rename over the target
  (`std::fs::rename` is atomic on NTFS when both paths are on the same
  volume, which the library directory always is).
- `library_dir()` returns `%APPDATA%\DocSnap\projects` (or platform
  equivalent). `list_projects()` lists newest-first.
- Commands: `save_project`, `load_project`, `list_projects`.

## Phase 17 — Search
- Documents page combines project-name filtering with text search across
  OCR results of the currently loaded pages (per-line matches, scrollable).
- Searching across all stored projects without opening them is a Phase 20
  feature once we persist an on-disk FTS index; the current scope keeps
  everything local and lazy.

## Open items
- Source copying: a `.docsnap` currently references originals by canonical
  path. Opt-in "copy into project" lands in Phase 20 so sharing a project
  file is useful off-machine.
- Open-project loads pages via their *source_path*; previews / warps /
  enhancements are re-rendered on first use. If the source is missing, the
  page shows a broken-image state (reported in the open error row).

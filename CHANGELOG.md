# Changelog

## 0.1.1 — 2026-09-16

- Keep recent-file opening responsive on dense documents by preparing HTML
  outside the live document, yielding during assembly and deferring layout of
  offscreen chunks. A newer open request supersedes an unfinished earlier one.
- Show which file is opening and keep the outline in its own scroll area so
  recent files remain reachable.
- Find the active heading with a binary search and update only the previous
  and new outline selection, avoiding document-wide work on every scroll.
- Preserve headings named after reader controls, such as “Status”, “Document”
  and “Font reset”. Canonical Markdown anchors now resolve through a document
  heading map, with separate DOM identifiers for rendered headings.
- Keep outline navigation, local anchors and reading-context reload working
  with the separate heading identifiers.
- Add a regression test for control-name collisions and duplicate DOM IDs.
- Synchronize CLI, desktop and npm package versions to 0.1.1.

## 0.1.0 — 2026-09-16

Initial pre-1.0 Markdown reader.

- Shared Rust document engine with CommonMark/GFM parsing, unique headings,
  links, original code, metadata, bounded UTF-8 loading and directory defaults.
- Terminal reader with Unicode wrapping, styling, tables, nested/task lists,
  stdin, plain output, external pager and desktop launch commands.
- Vanilla TypeScript / Tauri 2 desktop reader with safe HTML, Rust syntax
  highlighting, selectable text, exact code copying and scoped local images.
- Outline navigation, document search, recent files, native open menu/dialog,
  drag/drop, system/light/dark themes, font shortcuts and zen mode.
- Directory-based file watching, debounced reload and reading-context retention.
- Single-instance opening and `.md`/`.markdown` bundle associations.
- Native View menu for find, outline, zen and reading-font commands.
- Local Rust and browser tests; no GitHub Actions or telemetry.
- Bounded reuse of identical highlighted code blocks, avoiding repeated
  syntax work without dropping large-code highlighting.
- Native Apple Silicon CLI on npm and locally validated macOS app/DMG bundles.

Performance, native artifact verification and distribution limits are recorded
in `docs/VALIDATION.md` and `docs/PERFORMANCE.md`. Crates.io publication and
signed cross-platform installers are not included in this version.

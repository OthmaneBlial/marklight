# Changelog

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

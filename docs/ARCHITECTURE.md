# Architecture

Marklight is a reader. Its three Rust crates and Tauri binary form one Cargo
workspace, with the CLI/core/render crates as default members.

- `marklight-core` owns loading, the pulldown-cmark event stream, heading IDs,
  original code, links, metadata, URL/path policy, TOML configuration and notify
  watchers. It has no UI or Tauri dependency.
- `marklight-render` consumes that event stream for width-aware terminal output
  or sanitized HTML. Syntect highlights bounded code blocks in Rust. The HTML
  renderer borrows large text events to avoid duplicating their allocations.
- `marklight` uses Clap for commands, terminal-size detection, stdin and an
  existing pager. It launches the desktop using a configured binary, sibling
  executable, or the installed platform application.
- `apps/desktop` has a vanilla TypeScript frontend, Tauri 2 native menu/dialogs,
  filesystem commands and events. There is no JavaScript Markdown parser and
  no backend server in the installed app.

## Native state

Rust maintains the current document, one directory watcher, configuration and
an image-token map under a mutex. Loading/rendering/dialog commands run on
blocking workers. Failed loads preserve the current document. The frontend
serializes document actions so rapid changes do not replace a newer read with
an older one. File events are debounced; watching the directory handles atomic
editor saves. Read/access events do not trigger a reload loop.

A reload returns newly rendered HTML. Before replacement, the frontend saves
an active heading and its vertical offset. It restores that offset when the
same heading exists, and falls back to the previous scroll offset otherwise.

## Rendering boundaries

There is one Markdown parse per load. The parser's event stream supplies both
renderers, heading/link extraction and original code payloads. GFM bare links
are added at the shared event layer, excluding code and existing links.

Raw HTML is separately reduced to basic formatting; generated HTML is then
allowlist sanitized. The reading DOM uses ordinary selectable text. Code-copy
buttons refer to original source code in the payload, not highlighted HTML.

Image requests use a custom Tauri URI protocol with opaque generation tokens.
Only referenced local raster files in the document directory tree are granted;
no unrestricted filesystem plugin is exposed to JavaScript. Tokens expire on
reload/open. Link commands verify that the destination occurs in the current
Markdown, then classify it into an anchor, external URL or local Markdown file.

## Runtime and development

The installed app embeds its frontend and uses the operating system WebView.
Vite is a development/build tool only. No Docker, database, telemetry, account,
cloud connection or runtime web server is part of the application.

The local test harness uses actual Rust-generated HTML and mocks Tauri IPC to
exercise frontend behavior in Chromium. That is complementary to Rust native
state tests and actual macOS package/runtime checks; it does not test Windows
or Linux packaging or their native dialogs.

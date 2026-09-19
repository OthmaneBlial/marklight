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
an image-token map under a mutex. Each opened document also owns a capability
directory handle for its image root; image reads use that handle after token
lookup, so replacing a path with a symlink cannot redirect a read outside the
opened folder. Loading/rendering/dialog commands run on
blocking workers. Failed loads preserve the current document. The frontend
serializes document actions so rapid changes do not replace a newer read with
an older one. File events are debounced; watching the directory handles atomic
editor saves. Read/access events do not trigger a reload loop.

The frontend keeps a separate, in-memory back/forward stack of at most 50
file, heading and scroll positions. Successful navigation records a visit;
failed opens do not. Persisted recents contain paths only and are unrelated
to the current session's back/forward stack. Link/open/reload failures leave
the last valid document visible and show a persistent alert with a retry
action. A reload response is ignored when a newer open request has superseded
it.

`apps/desktop/src/main.ts` owns the document session, history, outline window,
focus, preferences and native IPC calls. `search.ts` owns text-node matching and
mark replacement. Neither parses Markdown. The first-use example consists of
two checked-in files in `fixtures/markdown/`; Tauri bundles them as local
resources and the welcome button resolves the bundled path before using the
same open command as a user file.

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

The frontend assembles rendered content outside the live document. Dense
documents are grouped into chunks with deferred offscreen layout; their text
and DOM nodes remain present. Assembly yields between chunks and checks for a
newer open request. Canonical heading anchors map to separate DOM identifiers
so headings named after reader controls cannot overwrite those controls.
Active-heading lookup searches only headings in the visible document chunk.
Outlines with more than 400 headings keep at most 100 links in the DOM at a
time, with filtering and page controls to reach distant sections. Search
clears and marks one document chunk at a time, yielding between chunks and
discarding stale queries after user input or a document change. Marking a text
node builds one replacement fragment instead of editing the DOM once per match.

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

# Validation — v0.1.0

Validated locally on 2026-09-16: macOS 26.6, Apple M2, arm64, Rust stable 1.95,
Node 25.9. The project is pre-1.0. All development tests run locally; the source
repository contains **zero GitHub Actions workflows** and GitHub Actions are
disabled in its repository settings.

## Passed

| Area | Evidence |
|---|---|
| Workspace | `cargo fmt --check`; strict Clippy for all workspace targets and also all features; `cargo test --workspace`: 21 passed |
| Core | 9 tests: GFM/links/code originals, Unicode and duplicate heading IDs, paths and image containment, bounded loading/errors, TOML/recents, atomic-save watching |
| Rendering | 6 tests: terminal golden behavior/Unicode/controls, hostile HTML and URLs, scoped images, highlighted large code and distinct copy IDs |
| CLI | 4 executable integration tests: file/stdin/redirected output, directory priority, help/version/error codes, canonical desktop argument with spaces |
| Real pager | `python3 scripts/pager-smoke.py`: actual `less -R` in a controlling PTY; initial paging, `/Section 400` search and `q` with exit 0 |
| Frontend | TypeScript strict check and Vite production build; 7 Playwright tests using real Rust-rendered fixtures |
| Browser behavior | Exact code clipboard, ordinary multi-block selection, styled-text search/count/cycling, outline/reading-context reload, theme/font/zen/mobile, menu/drop IPC wiring, relative navigation/recents, hostile document without remote requests |
| Native app | Release `custom-protocol` build; embedded frontend opens offline at `tauri://localhost`, without a development server |
| Native selection/copy | macOS native WebView: code-copy button then pasteboard byte equality, including final newline; normal mouse selection across two paragraphs then Cmd+C |
| Native opening | Cmd+O opens the native filtered picker and selected GFM file; CLI forwarding opens another document in the same primary instance; `open -a Marklight.app basic.md` reaches OS file opening |
| Native menus | View → Zen produces the reading-only layout; Escape exits; native Increase Text Size shows 17 px and Reset restores 16 px |
| Native links/images | Local Markdown link opens its target at Section 20; referenced local raster image is present; remote image shows an unavailable note |
| Native watching | Atomic replacement inserts 20 paragraphs above the visible Section 20; new word count/content arrives and Section 20 retains its visual position |
| macOS packaging | App ZIP, simple DMG and CLI archive; DMG CRC verification, read-only mount, matching native binary SHA-256, Applications symlink, local ad-hoc resource seal verification; extracted CLI runs the actual GFM golden |
| npm | `marklight@0.1.0` published by `othmaneblial`; registry shasum matches the local tarball; clean isolated registry install, version/help, GFM golden, stdin and spaced path pass; `npm exec --package=marklight@0.1.0 -- marklight --version` passes |
| Site | Showcase/docs at the canonical `/marklight/` path return HTTP 200; local 390/1280 browser checks, 16 exact snippet copies, 72 local references, themes/images, no console errors, JS-disabled mobile layout |

Reproduction: `scripts/check.sh`, `scripts/check-npm.py`,
`scripts/pager-smoke.py` and `scripts/check-site.mjs`. See
[performance](PERFORMANCE.md) for measured timing and memory scope.

## Scope and remaining limitations

- Browser tests replace native IPC with a test adapter. They prove real Rust
  rendering and frontend interactions; they do not prove another OS or native
  permission/file dialog behavior. Native macOS checks above are separate.
- The 400 px browser layout fits horizontally. Browser screenshots in
  `docs/images/` are interface captures with that adapter, not native screenshots.
- Native drag/drop is implemented through Tauri and its frontend callback is
  tested. A real Finder drag/drop gesture was not independently recorded.
- Font/Zen key chords were verified in browser logic; native menu actions and
  registered equivalents were checked separately. Automated native key-chord
  activation for those actions was inconclusive.
- Semantic controls/headings and visible keyboard focus are present in the
  native accessibility tree. A full VoiceOver/NVDA or WCAG audit has not run.
- The macOS app has a **local ad-hoc seal**, not a Developer ID signature or
  notarization. Local signature integrity and successful launch do not establish
  Gatekeeper approval after download. No signing identity was created or reused.
- Intel Mac, Linux and Windows runtime/packages are unverified. No Xcode,
  simulator, cross-platform VM or additional Apple tooling was installed.
- The npm binary package deliberately supports macOS arm64 only. It includes
  the CLI, not the desktop. Crates.io packages are not published.
- All five generated 10 KB–10 MB documents reached native DOM/layout readiness;
  this does not prove smooth scrolling/search at every size.
- The desktop 500 ms startup target has not been met. Large document layout
  and memory costs exceed the small-fixture kernel timings; see performance.
- Markdown is loaded as a whole; there is no virtualization. Documents cap at
  32 MiB, images at 16 MiB and highlighted search matches at 10,000.

## Release artifacts

Local artifacts are in `artifacts/release/` (ignored by Git):

- `Marklight-0.1.0-macOS-arm64.app.zip`
- `Marklight-0.1.0-macOS-arm64.dmg`
- `marklight-0.1.0-darwin-arm64.tar.gz`
- `SHA256SUMS`

The npm tarball is `artifacts/npm/marklight-0.1.0.tgz` (four files, about 580 KB
compressed; about 1.3 MB unpacked). The native desktop executable is about
11 MB; compressed desktop packages are about 5 MB. No unverified binary for
another operating system is included.

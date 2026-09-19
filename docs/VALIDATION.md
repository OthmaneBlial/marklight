# Validation — v0.1.1 baseline and current development

## Development progress after v0.1.1 on 2026-09-19

At local source commit `91ff716`, `npm run build` and 12 Playwright scenarios
passed. The added regressions cover a 100-link outline window, heading filter,
position after reload, search replacement/close/reopen/file switch, and the
10,000-match display cap. A pathological 10,001-match single paragraph timed
out after 30 seconds with the prior marking algorithm; it completed in the
updated browser test. These are Chromium tests using the Rust-generated fixture
adapter except the explicit synthetic match-cap case.

A **local development build** of the macOS arm64 app opened the generated
100,000-byte Markdown file through the native picker. Its WebKit accessibility
tree showed 409 headings with outline pages `1–100` and `101–200`; native
search reported `1 / 409`, then `2 / 409` after Next. The actual WebKit window
was visually inspected. The app was closed and the prior outline preference
restored. This does not validate search response time at 1/10 MB, a screen
reader, the public 0.1.1 download, or a new release.

At commit `d76bb29`, `./scripts/check.sh` passed again: 21 Rust tests and 16
Playwright tests, including session back/forward with restored heading offset,
failed open/reload with Retry, a delayed stale reload, and a relative link that
becomes available later. The new link fixture is Rust rendered; the failure
timing in Playwright is mocked at the Tauri IPC boundary.

The release-mode local macOS arm64 build was then exercised in actual WebKit.
From a temporary Markdown file, a link to an absent sibling produced a
persistent alert while the first page remained readable. Creating the sibling
and clicking Retry opened it; Back returned to the first page. Renaming the
first file away produced a reload alert while its old text remained visible;
restoring the file triggered a successful reload and cleared the alert. A
separate native pass opened checked-in `huge.md`, jumped to Section 400 (48%
progress), opened `basic.md`, and used Back to restore Section 400 at the same
visible offset and 48% progress. Both test app instances were closed. Temporary
files were removed and the pre-test TOML configuration was restored with a
matching SHA-256. These local checks do not validate another OS or a public
download.

## Recheck on 2026-09-19 for roadmap baseline

At local HEAD `4e4fd53` (only `ROADMAP.md` after the 0.1.1 code tag),
`./scripts/check.sh` passed: npm clean install, TypeScript/Vite build,
`cargo fmt --check`, strict workspace Clippy, 21 Rust tests, Rust-generated
frontend fixtures and nine Playwright tests. Strict Clippy with all features,
`python3 scripts/pager-smoke.py`, `python3 scripts/check-npm.py` on the local
0.1.1 tarball, and `node scripts/check-site.mjs` also passed separately.
See [the 0.1.1 performance recheck](PERFORMANCE.md#recheck-of-011-on-2026-09-19)
and its [raw measurements](measurements-0.1.1-baseline.json).

Read-only GitHub and npm metadata checks on that date showed a public GitHub
release tagged `v0.1.1` with a macOS arm64 app ZIP, DMG, CLI archive and
`SHA256SUMS`, and an npm package `marklight@0.1.1` restricted to macOS arm64.
This recheck did **not** download those public assets into a clean environment,
exercise Gatekeeper or revalidate the site URL. The native interaction table
below remains the evidence from 2026-09-16, not a new native UI pass.

Validated locally on 2026-09-16: macOS 26.6, Apple M2, arm64, Rust stable 1.95,
Node 25.9. The project is pre-1.0. All development tests run locally; the source
repository contains **zero GitHub Actions workflows** and GitHub Actions are
disabled in its repository settings.

Native interaction and performance measurements were initially recorded for
0.1.0. The 0.1.1 corrective build additionally verifies control-name headings,
anchor navigation and recent-file switching with dense documents. It does not
repeat the original performance measurements.

## Passed

| Area | Evidence |
|---|---|
| Workspace | `cargo fmt --check`; strict Clippy for all workspace targets and also all features; `cargo test --workspace`: 21 passed |
| Core | 9 tests: GFM/links/code originals, Unicode and duplicate heading IDs, paths and image containment, bounded loading/errors, TOML/recents, atomic-save watching |
| Rendering | 6 tests: terminal golden behavior/Unicode/controls, hostile HTML and URLs, scoped images, highlighted large code and distinct copy IDs |
| CLI | 4 executable integration tests: file/stdin/redirected output, directory priority, help/version/error codes, canonical desktop argument with spaces |
| Real pager | `python3 scripts/pager-smoke.py`: actual `less -R` in a controlling PTY; initial paging, `/Section 400` search and `q` with exit 0 |
| Frontend | TypeScript strict check and Vite production build; 9 Playwright tests using real Rust-rendered fixtures |
| Browser behavior | Exact code clipboard, ordinary multi-block selection, styled-text search/count/cycling, outline/reading-context reload, theme/font/zen/mobile, menu/drop IPC wiring, relative navigation/recents, chunked document opening and newer-click priority, control-name heading collisions and unique DOM IDs, hostile document without remote requests |
| Native app | Release `custom-protocol` build; embedded frontend opens offline at `tauri://localhost`, without a development server |
| Native selection/copy | macOS native WebView: code-copy button then pasteboard byte equality, including final newline; normal mouse selection across two paragraphs then Cmd+C |
| Native opening | Cmd+O opens the native filtered picker and selected GFM file; CLI forwarding opens another document in the same primary instance; `open -a Marklight.app basic.md` reaches OS file opening |
| Native menus | View → Zen produces the reading-only layout; Escape exits; native Increase Text Size shows 17 px and Reset restores 16 px |
| Native 0.1.1 recents | Clicks open the dense 5,000,000- and 10,000,000-byte generated files, each followed by a successful recent-file return to `basic.md`; updated app is left on `gfm.md` at 16 px |
| Native 0.1.1 headings | `ui-collisions.md` retains Document/Status/Font reset/Sidebar; increasing to 17 px preserves heading text; outline Sidebar and local Return to Document focus their document headings |
| Native links/images | Local Markdown link opens its target at Section 20; referenced local raster image is present; remote image shows an unavailable note |
| Native watching | Atomic replacement inserts 20 paragraphs above the visible Section 20; new word count/content arrives and Section 20 retains its visual position |
| macOS packaging | App ZIP, simple DMG and CLI archive; DMG CRC verification, read-only mount, matching native binary SHA-256, Applications symlink, local ad-hoc resource seal verification; extracted CLI runs the actual GFM golden |
| npm | `marklight@0.1.1` published by `othmaneblial`; registry shasum matches the local tarball; clean isolated registry install, version/help, GFM golden, stdin and spaced path pass; `npm exec --package=marklight@0.1.1 -- marklight --version` passes |
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
- Markdown and its DOM remain loaded as a whole; offscreen layout is deferred
  in chunks rather than removing document nodes. Documents cap at
  32 MiB, images at 16 MiB and highlighted search matches at 10,000.

## Release artifacts

Local artifacts are in `artifacts/release/` (ignored by Git):

- `Marklight-0.1.1-macOS-arm64.app.zip`
- `Marklight-0.1.1-macOS-arm64.dmg`
- `marklight-0.1.1-darwin-arm64.tar.gz`
- `SHA256SUMS`

The npm tarball is `artifacts/npm/marklight-0.1.1.tgz` (four files, about 580 KB
compressed; about 1.3 MB unpacked). The native desktop executable is about
11 MB; compressed desktop packages are about 5 MB. No unverified binary for
another operating system is included.

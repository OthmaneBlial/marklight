# Installation

## Terminal reader

For macOS Apple Silicon (arm64), the npm package contains the native CLI:

```sh
npm install --global marklight
marklight README.md
npx marklight README.md
```

It has no install scripts or runtime dependencies and does not install the
desktop app. Other operating systems and architectures use Rust source builds.

Rust stable is required for source installation. The default workspace members
are CLI/core/render only, so terminal installation does not need Tauri, Node,
Xcode or a browser.

```sh
cargo install --git https://github.com/OthmaneBlial/marklight marklight
marklight README.md
marklight .
git show HEAD:README.md | marklight -
marklight README.md --plain --no-pager > readme.txt
```

From a checkout:

```sh
cargo install --path crates/marklight-cli
```

`cargo install marklight` is a planned registry path, not a verified installation
command for this release. Marklight's crates have not been published to crates.io.
Do not install an unrelated registry crate based on a matching name.

Large terminal documents use `$PAGER` (default `less -R`). Use `/query` to search,
`n`/`N` to cycle results and `q` to quit. Set `--no-pager` to print directly.
Piped output is automatically plain and never opens a pager. Use `--width 100`
for explicit wrapping and `--theme light`/`dark` for your terminal palette.
`NO_COLOR` disables ANSI styling.

## Desktop from source

The installed desktop app has no web server and embeds its frontend. Node is
needed only to build it. Use the [official Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/)
for your OS. From the repository root on macOS:

```sh
npm --prefix apps/desktop ci
npm --prefix apps/desktop run tauri -- build --features custom-protocol --bundles app
```

The result is under `target/release/bundle/`. On macOS the `.app` can be moved
into Applications. `./scripts/package.sh` builds the CLI and native bundles
locally and creates a simple macOS DMG plus application ZIP and CLI archive under
`artifacts/release/`. The standalone script `python3 scripts/bundle-macos.py`
packages existing builds. It uses `hdiutil` rather than automating Finder layout;
the default decorated Tauri DMG bundler failed on the validation host.
No GitHub Actions are used.

For Linux/Windows source builds, use `npm run tauri -- build --features
custom-protocol` from `apps/desktop`, with native Tauri prerequisites installed.
These native packages and operating systems have not been tested here.

## Desktop downloads

[GitHub Releases](https://github.com/OthmaneBlial/marklight/releases/latest)
provides the initial macOS Apple Silicon DMG and application ZIP, a standalone
CLI archive, and SHA-256 checksums. Drag Marklight.app to Applications. The app
is not Developer ID signed or notarized; macOS Gatekeeper can prevent opening
downloaded builds. A successful local launch does not prove downloaded-app
Gatekeeper approval. Check the source and checksum before approving an app.

The CLI discovers a sibling `marklight-desktop` executable, an installed
Marklight macOS application, or a `marklight-desktop` command on other systems.
To launch an uninstalled build explicitly:

```sh
export MARKLIGHT_DESKTOP="$PWD/target/release/bundle/macos/Marklight.app/Contents/MacOS/marklight-desktop"
marklight open README.md
marklight README.md --gui
```

## File associations

The Tauri bundle declares Marklight as a **Viewer** for `.md` and `.markdown`.
It does not silently change the user's default editor or application.

macOS: install the `.app`, right-click a Markdown file → Open With → Marklight.
To make double-click opening the default, Get Info → Open with → Marklight →
Change All. Finder opening reaches Tauri's `RunEvent::Opened`; startup events
are buffered until the frontend is ready. An already-running app receives the
new file. Command-line launches also use the single-instance plugin.

Windows: the installer registers extensions; use Settings → Apps → Default
apps or the Open with dialog to choose Marklight. A user's existing defaults
may prevent an installer from claiming the extension automatically.

Linux: associations depend on package format and desktop environment. A `.deb`
installs a desktop entry; AppImages may require manual integration. Choose
Marklight as the handler in your desktop's Open With settings. Do not assume
an AppImage changes MIME defaults automatically.

Windows/Linux packages are build targets and require native testing on those
systems. Current host package evidence and signing limits are recorded in
[validation](VALIDATION.md).

## Keyboard shortcuts

| Action | macOS | Windows/Linux |
|---|---|---|
| Open | Cmd+O | Ctrl+O |
| Find | Cmd+F | Ctrl+F |
| Larger / smaller text | Cmd++ / Cmd+- | Ctrl++ / Ctrl+- |
| Reset text size | Cmd+0 | Ctrl+0 |
| Outline | Cmd+Shift+T | Ctrl+Shift+T |
| Zen mode | Cmd+Shift+Z | Ctrl+Shift+Z |
| Next / previous search result | Enter / Shift+Enter | Enter / Shift+Enter |
| Close search / leave zen | Escape | Escape |

Text and code use normal selection. Cmd/Ctrl+C copies your selection; the code
Copy button copies original code including trailing newlines.

## Local preferences

The tiny TOML file stores theme, font size (12–28), outline/zen settings and at
most twelve recent local file paths. The default theme is System.

- macOS: `~/Library/Application Support/dev.Marklight.Marklight/config.toml`
- Linux: `$XDG_CONFIG_HOME/marklight/config.toml` (normally `~/.config/marklight`)
- Windows: `%APPDATA%\Marklight\Marklight\config\config.toml`

These paths use the `directories` crate's platform conventions. Use Clear
Recent in the sidebar to forget history. No database, account or network sync.

```toml
theme = "system"
font_size = 16
toc = true
zen_mode = false
recent = []
```

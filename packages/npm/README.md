# Marklight

A dedicated offline Markdown reader written in Rust. This npm package contains
the native terminal CLI for **macOS 12+ on Apple Silicon (arm64)**. It has no
JavaScript runtime dependencies, installation scripts or download hooks.

```sh
npm install --global marklight
marklight README.md
marklight .
printf '# Hello\n\nRead quietly.\n' | marklight -
marklight README.md --plain --no-pager
```

Or run without a global install:

```sh
npx marklight README.md
```

Read headings, lists, tables, code and links with automatic terminal wrapping.
Long documents use your pager; with `less`, `/` searches and `q` exits. Use
`marklight --help` for width and theme options. Pipe or redirect output for plain
text. Reading a document makes no network requests.

`marklight open README.md` / `--gui` launch the separately installed desktop app.
The npm package does **not** install the desktop app. Download the macOS desktop
from [GitHub Releases](https://github.com/OthmaneBlial/marklight/releases), or
follow the [source installation guide](https://github.com/OthmaneBlial/marklight/blob/main/docs/INSTALLATION.md).
Desktop builds are unsigned and not notarized in this initial release.

Linux, Windows and Intel Mac users can build the CLI from Rust source:

```sh
cargo install --git https://github.com/OthmaneBlial/marklight marklight
```

Other platforms are not runtime validated in v0.1.1. This package deliberately
rejects unsupported architectures rather than downloading an untested binary.
Upgrade with `npm install --global marklight@latest`.

[Source and docs](https://github.com/OthmaneBlial/marklight) · MIT license.

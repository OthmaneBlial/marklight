# Marklight

A dedicated Markdown reader, written in Rust. Read a file in your terminal,
or open it in a small Tauri desktop window. No editor, accounts, or telemetry.

Marklight is under active construction. See [the implementation plan](docs/PLAN.md)
for scope and verification. Installation and screenshots will be added as the
corresponding artifacts are validated.

## Development

Rust stable; desktop uses Tauri 2 and a vanilla TypeScript frontend.

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Tests run locally. This repository intentionally has no GitHub Actions.

MIT licensed.

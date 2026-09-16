#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
npm --prefix apps/desktop ci
cargo build --release -p marklight
if test "$(uname -s)" = Darwin; then
  npm --prefix apps/desktop run tauri -- build --features custom-protocol --bundles app
  python3 scripts/bundle-macos.py
else
  npm --prefix apps/desktop run tauri -- build --features custom-protocol
fi
printf 'CLI: %s/target/release/marklight\nDesktop bundles: %s/target/release/bundle/\n' "$PWD" "$PWD"

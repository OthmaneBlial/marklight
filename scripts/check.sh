#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
if test -d .github/workflows && test -n "$(find .github/workflows -type f -print)"; then
  echo 'Marklight intentionally has no GitHub Actions workflows.' >&2
  exit 1
fi
npm --prefix apps/desktop ci
npm --prefix apps/desktop run build
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -q -p marklight-render --example frontend_fixtures
npm --prefix apps/desktop test

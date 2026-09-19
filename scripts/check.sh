#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
printf 'Marklight local gate: %s (%s/%s)\n' "$(git rev-parse --short HEAD)" "$(uname -s)" "$(uname -m)"
if git diff --quiet && git diff --cached --quiet && test -z "$(git ls-files --others --exclude-standard)"; then
  printf '%s\n' 'Source tree: clean'
else
  printf '%s\n' 'Source tree: dirty (development result only)'
fi
rustc --version
cargo --version
node --version
npm --version
python3 --version
if test -d .github/workflows && test -n "$(find .github/workflows -type f -print)"; then
  echo 'Marklight intentionally has no GitHub Actions workflows.' >&2
  exit 1
fi
npm --prefix apps/desktop ci
npm --prefix apps/desktop run build
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo run -q -p marklight-render --example frontend_fixtures
npm --prefix apps/desktop test
node scripts/check-site.mjs
printf '%s\n' 'PASS: local source, frontend, Rust and site gate'

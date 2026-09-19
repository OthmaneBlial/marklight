#!/bin/sh
set -eu
cd "$(dirname "$0")/.."

if ! cargo audit --version >/dev/null 2>&1; then
  printf '%s\n' 'Install cargo-audit before running the dependency review.' >&2
  exit 2
fi

cargo audit
npm --prefix apps/desktop audit --audit-level=low

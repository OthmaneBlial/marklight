#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
if test "$(uname -s)" != Darwin || test "$(uname -m)" != arm64; then
  echo 'The initial npm binary package is built on macOS Apple Silicon only.' >&2
  exit 1
fi
cargo build --release -p marklight
mkdir -p packages/npm/bin artifacts/npm
cp target/release/marklight packages/npm/bin/marklight
cp LICENSE packages/npm/LICENSE
chmod 755 packages/npm/bin/marklight
task_artifacts="${MARKLIGHT_NPM_OUTPUT:-$PWD/artifacts/npm}"
mkdir -p "$task_artifacts"
task_artifacts="$(cd "$task_artifacts" && pwd)"
cd packages/npm
npm pack --pack-destination "$task_artifacts"

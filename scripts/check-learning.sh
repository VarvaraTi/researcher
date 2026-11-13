#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
export CARGO_HOME="$PWD/.local/cargo"
cargo fmt --all -- --check
cargo test --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings

# Optional local HTML renderer; the core learning executable does not need Node.
if command -v node >/dev/null 2>&1; then
  node --test tests/*.test.mjs
fi

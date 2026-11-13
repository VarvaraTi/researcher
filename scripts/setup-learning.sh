#!/bin/sh
# Build tools outside the plugin cache, then initialize only missing workspace data.
set -eu
plugin_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
[ "$#" -ge 1 ] || { echo 'Usage: setup-learning.sh WORKSPACE [--okf PATH] [--offline]' >&2; exit 1; }
learning_workspace=$1
shift
okf_source=
build_offline=
while [ "$#" -gt 0 ]; do
  case "$1" in
    --okf) [ "$#" -ge 2 ] || exit 1; okf_source=$2; shift 2 ;;
    --offline) build_offline=--offline; shift ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done
command -v cargo >/dev/null 2>&1 || { echo 'Rust/Cargo is required. Install a Rust toolchain, then rerun setup.' >&2; exit 1; }
mkdir -p "$learning_workspace"
learning_workspace=$(CDPATH= cd -- "$learning_workspace" && pwd)
case "$learning_workspace/" in
  "$plugin_root/"*) echo 'Choose a data workspace outside the installed plugin directory.' >&2; exit 1 ;;
esac
if [ -n "$okf_source" ]; then
  [ "$("$okf_source" version)" = 'okf 0.6.0 (OKF spec 0.2)' ] || { echo 'Expected OKF 0.6.0.' >&2; exit 1; }
fi
mkdir -p "$learning_workspace/.local/bin"
okf_target="$learning_workspace/.local/bin/okf"
if [ -e "$okf_target" ] || [ -L "$okf_target" ]; then
  [ ! -L "$okf_target" ] && [ -f "$okf_target" ] || { echo 'Invalid workspace OKF executable.' >&2; exit 1; }
  [ "$("$okf_target" version)" = 'okf 0.6.0 (OKF spec 0.2)' ] || { echo 'Existing OKF is not 0.6.0; resolve the version explicitly.' >&2; exit 1; }
elif [ -n "$okf_source" ]; then
  cp "$okf_source" "$okf_target"
  chmod 755 "$okf_target"
elif [ -n "$build_offline" ]; then
  echo 'Offline setup requires --okf PATH or an existing workspace OKF 0.6.0.' >&2
  exit 1
else
  sh "$plugin_root/scripts/setup-okf.sh" "$learning_workspace/.local/bin"
fi
export CARGO_HOME="${CARGO_HOME:-$learning_workspace/.local/cargo}"
cargo build --locked $build_offline --manifest-path "$plugin_root/Cargo.toml" \
  --target-dir "$learning_workspace/.local/researcher/build" --bins
for learning_binary in learning-rules learning-workspace; do
  learning_target="$learning_workspace/.local/bin/$learning_binary"
  [ ! -L "$learning_target" ] && [ ! -d "$learning_target" ] || { echo "Invalid target: $learning_target" >&2; exit 1; }
  cp "$learning_workspace/.local/researcher/build/debug/$learning_binary" "$learning_target"
done
"$learning_workspace/.local/bin/learning-workspace" "$learning_workspace" "$okf_target"

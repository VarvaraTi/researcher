#!/bin/sh
# Install the pinned OKF release locally; no Rust or administrator access needed.
set -eu
project_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
version=0.6.0
destination=${1:-"$project_root/.local/bin"}
[ "$#" -le 1 ] || { echo 'Usage: setup-okf.sh [BIN_DIRECTORY]' >&2; exit 1; }
case "$(uname -s)/$(uname -m)" in
  Darwin/arm64|Darwin/aarch64) target=aarch64-apple-darwin ;;
  Darwin/x86_64) target=x86_64-apple-darwin ;;
  Linux/arm64|Linux/aarch64) target=aarch64-unknown-linux-musl ;;
  Linux/x86_64) target=x86_64-unknown-linux-musl ;;
  *) echo 'Unsupported platform.' >&2; exit 1 ;;
esac
for dependency in curl tar shasum; do
  command -v "$dependency" >/dev/null 2>&1 || { echo "Missing: $dependency" >&2; exit 1; }
done
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
asset="okf-$version-$target.tar.gz"
base="https://github.com/abatyuk/okf/releases/download/v$version"
for name in "$asset" SHA256SUMS; do
  curl --fail --location --show-error --connect-timeout 15 --max-time 180 \
    --proto '=https' --proto-redir '=https' "$base/$name" -o "$work/$name"
done
expected=$(awk -v name="$asset" '$2 == name {print $1}' "$work/SHA256SUMS")
actual=$(shasum -a 256 "$work/$asset" | awk '{print $1}')
[ -n "$expected" ] && [ "$expected" = "$actual" ] || { echo 'Checksum mismatch or missing checksum.' >&2; exit 1; }
tar -xzf "$work/$asset" -C "$work" okf
[ -f "$work/okf" ] && [ ! -L "$work/okf" ] || { echo 'Invalid executable in archive.' >&2; exit 1; }
chmod 755 "$work/okf"
[ "$("$work/okf" version)" = "okf $version (OKF spec 0.2)" ] || { echo 'Unexpected OKF version.' >&2; exit 1; }
mkdir -p "$destination"
[ ! -L "$destination/okf" ] && [ ! -d "$destination/okf" ] || { echo 'Refusing to overwrite a symlink or directory.' >&2; exit 1; }
cp "$work/okf" "$destination/okf"
chmod 755 "$destination/okf"
"$destination/okf" version
printf 'Installed in %s\nRun: export PATH="%s:$PATH"\n' "$destination" "$destination"

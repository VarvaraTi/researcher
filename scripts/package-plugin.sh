#!/bin/sh
# Explicit allowlist: never package personal runtime, caches or the upstream checkout.
set -eu
plugin_source=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
package_parent="$plugin_source/.local/plugin-package"
mkdir -p "$package_parent"
package_stage=$(mktemp -d "$package_parent/.researcher-XXXXXX")
trap 'rm -rf "$package_stage"' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
cd "$plugin_source"
# Check all inputs before copying. Symlinks are not accepted in the distribution.
package_inputs='.codex-plugin skills schemas templates policies/defaults crates/learning-rules/src crates/learning-rules/Cargo.toml Cargo.toml Cargo.lock docs/learning-operations.md docs/learning-contracts.md docs/adr docs/plugin.md learning-spec.md learning-intent.md tests/scenarios/learning-flow.md OKF-SETUP.md scripts/setup-learning.sh scripts/setup-okf.sh scripts/read-pdf.sh scripts/read-pdf.swift scripts/update-course-map.mjs scripts/lib/course-map.mjs scripts/ingest-book.mjs scripts/lib/book-ingestion.mjs scripts/retrieve-book.mjs scripts/lib/library-retrieval.mjs'
for package_input in $package_inputs; do
  [ -e "$package_input" ] || { echo "Missing package input: $package_input" >&2; exit 1; }
  [ -z "$(find "$package_input" -type l -print)" ] || { echo "Symlink in package input: $package_input" >&2; exit 1; }
done
tar -cf "$package_stage/payload.tar" $package_inputs
mkdir "$package_stage/researcher"
tar -xf "$package_stage/payload.tar" -C "$package_stage/researcher"
# Keep the previous generated package available until the replacement is ready.
if [ -e "$package_parent/researcher" ]; then
  mv "$package_parent/researcher" "$package_stage/previous"
fi
mv "$package_stage/researcher" "$package_parent/researcher"
printf 'Packaged plugin: %s/researcher\n' "$package_parent"

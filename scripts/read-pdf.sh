#!/bin/sh
# macOS PDFKit text extraction; no OCR or package installation.
set -eu
if [ "$(uname -s)" != Darwin ] || ! command -v swift >/dev/null 2>&1; then
  echo 'read-pdf requires macOS and Swift (Xcode Command Line Tools).' >&2
  exit 1
fi
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
pdf_workspace=$(CDPATH= cd -- "${LEARNING_WORKSPACE:-$PWD}" && pwd -P)
pdf_cache="$pdf_workspace/.local/pdf-swift-cache"
mkdir -p "$pdf_cache"
swift -module-cache-path "$pdf_cache" "$script_dir/read-pdf.swift" -- "$@"

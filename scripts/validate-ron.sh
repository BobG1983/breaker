#!/usr/bin/env bash
set -euo pipefail

# Check all .ron files have type annotations
if [ -d assets ]; then
  UNANNOTATED=$(find assets -name '*.ron' -exec sh -c 'head -5 "$1" | grep -qF "/* @[" || echo "$1"' _ {} \;)
  if [ -n "$UNANNOTATED" ]; then
    echo "ERROR: RON files missing type annotation (/* @[crate::path::Type] */):"
    echo "$UNANNOTATED"
    exit 1
  fi
fi
echo "All RON files have type annotations."

# Validate annotated files against Rust types
if command -v ron-lsp &>/dev/null; then
  ron-lsp check . 2>&1
else
  echo "WARNING: ron-lsp not installed, skipping RON type validation"
fi

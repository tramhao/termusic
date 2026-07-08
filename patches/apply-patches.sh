#!/usr/bin/env bash
# Apply cargo registry patches after `cargo build` downloads dependencies.
# Run from the termusic project root.
# Lost on `cargo clean` — re-run after each clean.

set -euo pipefail

REGISTRY_DIR="$HOME/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

apply_patch() {
  local crate_version="$1"
  local patch_file="$2"
  local target="$REGISTRY_DIR/$crate_version"

  if [ ! -d "$target" ]; then
    echo "$crate_version not found in registry. Run 'cargo build' first."
    return 1
  fi
  if [ ! -f "$patch_file" ]; then
    echo "Patch file not found: $patch_file"
    return 1
  fi

  patch -d "$target" -p0 --batch --forward -r - < "$patch_file" 2>/dev/null || true
  echo "Patched $crate_version"
}

apply_patch "tuirealm-orx-tree-0.4.0" "$SCRIPT_DIR/tuirealm-orx-tree.patch"
apply_patch "souvlaki-0.8.3" "$SCRIPT_DIR/souvlaki-0.8.3.patch"
apply_patch "tui-realm-stdlib-4.1.0" "$SCRIPT_DIR/tui-realm-stdlib-4.1.0.patch"

echo "All patches applied."

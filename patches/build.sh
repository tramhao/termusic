#!/usr/bin/env bash
# Patched build for termusic.
# Fetches dependencies, extracts + patches the three modified crates,
# then builds the project.
# Run from the termusic project root.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REGISTRY_SRC="$HOME/.cargo/registry/src"
REGISTRY_CACHE="$HOME/.cargo/registry/cache"

# Find the registry hash dir (stable across cargo updates on same index URL)
HASH_DIR=""
for d in "$REGISTRY_SRC"/index.crates.io-*; do
    if [ -d "$d" ]; then
        HASH_DIR="$(basename "$d")"
        break
    fi
done
if [ -z "$HASH_DIR" ]; then
    HASH_DIR="index.crates.io-1949cf8c6b5b557f"
fi

REGISTRY_DIR="$REGISTRY_SRC/$HASH_DIR"
CACHE_DIR="$REGISTRY_CACHE/$HASH_DIR"

echo "→ Registry: $REGISTRY_DIR"

# 1. Fetch all dependencies (download .crate files, no compilation)
echo "→ Fetching dependencies..."
cargo fetch

# 2. Extract patched crates to source dir if not already extracted
extract_if_missing() {
    local crate="$1"
    local version="$2"
    local target="$REGISTRY_DIR/$crate-$version"
    local cache_file="$CACHE_DIR/$crate-$version.crate"

    if [ -d "$target" ]; then
        echo "  ✓ $crate-$version already extracted"
        return
    fi

    echo "  → Extracting $crate-$version..."
    if [ ! -f "$cache_file" ]; then
        echo "  ✗ $cache_file not found. Try running 'cargo fetch' again."
        exit 1
    fi

    mkdir -p "$target"
    tar xzf "$cache_file" -C "$target" 2>/dev/null
    # Cargo checks for this marker to confirm clean extraction
    echo '{"v":1}' >"$target/.cargo-ok"
    echo "  ✓ Extracted $crate-$version"
}

extract_if_missing "tuirealm-orx-tree" "0.4.0"
extract_if_missing "tui-realm-stdlib" "4.1.0"
extract_if_missing "souvlaki" "0.8.3"

# 3. Apply patches
apply_patch() {
    local crate_version="$1"
    local patch_file="$2"
    local target="$REGISTRY_DIR/$crate_version"

    if [ ! -f "$patch_file" ]; then
        echo "  ✗ Patch file not found: $patch_file"
        exit 1
    fi

    patch -d "$target" -p0 --batch --forward -r - <"$patch_file" 2>/dev/null || true
    echo "  ✓ Patched $crate_version"
}

echo "→ Applying patches..."
apply_patch "tuirealm-orx-tree-0.4.0" "$SCRIPT_DIR/tuirealm-orx-tree.patch"
apply_patch "souvlaki-0.8.3" "$SCRIPT_DIR/souvlaki-0.8.3.patch"
apply_patch "tui-realm-stdlib-4.1.0" "$SCRIPT_DIR/tui-realm-stdlib-4.1.0.patch"

# 4. Clean the patched crates' artifacts so they get recompiled
echo "→ Cleaning patched crate artifacts..."
cargo clean -p tuirealm-orx-tree 2>/dev/null || true
cargo clean -p tui-realm-stdlib 2>/dev/null || true
cargo clean -p souvlaki 2>/dev/null || true

# 5. Build
echo "→ Building termusic..."
cargo build --release

echo "✓ Done! Binary at target/release/termusic"

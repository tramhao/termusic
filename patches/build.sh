#!/usr/bin/env bash
# Patched build for termusic.
# Always extracts fresh from .crate cache, applies patches, then builds.
# Works regardless of prior state — no cargo clean needed.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REGISTRY_SRC="$HOME/.cargo/registry/src"
REGISTRY_CACHE="$HOME/.cargo/registry/cache"

# Find the registry hash dir
HASH_DIR=""
for d in "$REGISTRY_SRC"/index.crates.io-*; do
    [ -d "$d" ] && { HASH_DIR="$(basename "$d")"; break; }
done
HASH_DIR="${HASH_DIR:-index.crates.io-1949cf8c6b5b557f}"

REGISTRY_DIR="$REGISTRY_SRC/$HASH_DIR"
CACHE_DIR="$REGISTRY_CACHE/$HASH_DIR"

echo "→ Registry: $REGISTRY_DIR"

# 1. Fetch all dependencies
echo "→ Fetching dependencies..."
cargo fetch

# 2. Clean fingerprints BEFORE re-extracting, so cargo sees changed sources
echo "→ Cleaning patched crate fingerprints..."
find target -path "*/.fingerprint/tui-realm-stdlib*" -exec rm -rf {} + 2>/dev/null || true
find target -path "*/.fingerprint/tuirealm*" -exec rm -rf {} + 2>/dev/null || true
find target -path "*/deps/tui_realm_stdlib*" -exec rm -rf {} + 2>/dev/null || true
find target -path "*/deps/libtui_realm_stdlib*" -exec rm -rf {} + 2>/dev/null || true
find target -path "*/deps/tuirealm*" -exec rm -rf {} + 2>/dev/null || true

# 3. ALWAYS extract fresh from .crate cache (rm -rf first)
echo "→ Extracting fresh from cache..."
for spec in "tuirealm-orx-tree:0.4.0" "tui-realm-stdlib:4.1.0" "souvlaki:0.8.3"; do
    crate="${spec%%:*}"
    version="${spec##*:}"
    target="$REGISTRY_DIR/$crate-$version"
    cache_file="$CACHE_DIR/$crate-$version.crate"

    if [ ! -f "$cache_file" ]; then
        echo "  ✗ $cache_file not found. Run 'cargo fetch' first."
        exit 1
    fi

    rm -rf "$target"
    mkdir -p "$target"
    tar xzf "$cache_file" -C "$target" --strip-components=1 2>/dev/null
    echo '{"v":1}' >"$target/.cargo-ok"
    echo "  ✓ $crate-$version (fresh)"
done

# 4. Apply all patches (always on clean originals)
echo "→ Applying patches..."
patch -d "$REGISTRY_DIR/tuirealm-orx-tree-0.4.0" -p0 --batch -r - \
    <"$SCRIPT_DIR/tuirealm-orx-tree.patch" 2>/dev/null && echo "  ✓ tuirealm-orx-tree" || echo "  ✗ tuirealm-orx-tree failed"
patch -d "$REGISTRY_DIR/souvlaki-0.8.3" -p0 --batch -r - \
    <"$SCRIPT_DIR/souvlaki-0.8.3.patch" 2>/dev/null && echo "  ✓ souvlaki" || echo "  ✗ souvlaki failed"
patch -d "$REGISTRY_DIR/tui-realm-stdlib-4.1.0" -p0 --batch -r - \
    <"$SCRIPT_DIR/tui-realm-stdlib-4.1.0.patch" 2>/dev/null && echo "  ✓ tui-realm-stdlib" || echo "  ✗ tui-realm-stdlib failed"

# 5. Build
echo "→ Building termusic..."
RUSTFLAGS="-C link-arg=-L/opt/homebrew/lib -C link-arg=-Wl,-rpath,/opt/homebrew/lib" \
    cargo build --release --features mpv

echo "✓ Done! Binary at target/release/termusic"

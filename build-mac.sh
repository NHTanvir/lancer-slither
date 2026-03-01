#!/usr/bin/env bash
# macOS build → build/lancer-slither + lancer-slither.app bundle.
#
# Requires:
#   - rustup (https://rustup.rs)
#   - cargo-bundle (cargo install cargo-bundle) — for the .app
#
# Usage:
#   ./build-mac.sh           # release binaries to build/
#   ./build-mac.sh --bundle  # also produce build/lancer-slither.app

set -euo pipefail

REPO="$(cd "$(dirname "$0")" && pwd)"
BUILD="$REPO/build"

echo "================================================"
echo "  lancer-slither macOS build"
echo "================================================"
echo ""
echo "  Repo   : $REPO"
echo "  Output : $BUILD/"
echo ""

echo "[1/3] Building release workspace..."
cargo build --release

echo "[2/3] Copying binaries..."
mkdir -p "$BUILD"
for B in \
    lancer-slither \
    lancer-slither-cli \
    lancer-slither-mutate \
    lancer-slither-prop \
    lancer-slither-check-upgradeability \
    lancer-slither-read-storage \
    lancer-slither-find-paths \
    lancer-slither-format \
    lancer-slither-flat \
    lancer-slither-check-erc \
    lancer-slither-interface \
    lancer-slither-doctor \
    lancer-slither-documentation
do
    if [ -f "$REPO/target/release/$B" ]; then
        cp "$REPO/target/release/$B" "$BUILD/"
    fi
done

if [ "${1:-}" = "--bundle" ]; then
    echo "[3/3] Creating .app bundle via cargo-bundle..."
    if ! command -v cargo-bundle >/dev/null 2>&1; then
        echo "  cargo-bundle not installed. Install with: cargo install cargo-bundle"
        exit 1
    fi
    pushd "$REPO/crates/lancer-slither" >/dev/null
    cargo bundle --release
    popd >/dev/null
    if [ -d "$REPO/target/release/bundle/osx/lancer-slither.app" ]; then
        rsync -a "$REPO/target/release/bundle/osx/lancer-slither.app" "$BUILD/"
    fi
fi

echo ""
echo "================================================"
echo "  Done!  Binaries in $BUILD/"
echo "================================================"
ls -1 "$BUILD/"

#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "$0")" && pwd)"
BUILD="$REPO/build"

echo "================================================"
echo "  lancer-slither Build"
echo "================================================"
echo ""
echo "  Repo   : $REPO"
echo "  Output : $BUILD/"
echo ""

echo "[1/2] Building release binaries (workspace)..."
cargo build --release

echo "[2/2] Copying binaries to build/..."
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

echo ""
echo "================================================"
echo "  Done!  Binaries in $BUILD/"
echo "================================================"
ls -1 "$BUILD/"

if [ "${1:-}" = "--install" ]; then
    APPS="$HOME/.local/share/applications"
    mkdir -p "$APPS"
    cat > "$APPS/lancer-slither.desktop" <<EOF
[Desktop Entry]
Name=lancer-slither
Comment=Solidity static analyzer (Lancer)
Exec=$BUILD/lancer-slither
Terminal=false
Type=Application
Categories=Development;Security;
EOF
    echo "Desktop entry installed: $APPS/lancer-slither.desktop"
fi

#!/usr/bin/env bash
# macOS release build for lancer-frida.
# Output: build/lancer-frida  (+ optional lancer-frida.app via cargo-bundle)
set -euo pipefail

REPO="$(cd "$(dirname "$0")" && pwd)"
BUILD="$REPO/build"
BIN="lancer-frida"

echo "================================================"
echo "  lancer-frida Build (macOS)"
echo "================================================"
echo ""
echo "  Repo   : $REPO"
echo "  Output : $BUILD/$BIN"
echo ""

echo "[1/3] Building release binary..."
cargo build --release

echo "[2/3] Copying to build/..."
mkdir -p "$BUILD"
cp "$REPO/target/release/$BIN" "$BUILD/"

echo "[3/3] Bundling lancer-frida.app (optional)..."
if command -v cargo-bundle >/dev/null 2>&1; then
    cargo bundle --release
    APP="$REPO/target/release/bundle/osx/lancer-frida.app"
    if [ -d "$APP" ]; then
        rm -rf "$BUILD/lancer-frida.app"
        cp -R "$APP" "$BUILD/"
        echo "      Bundled: $BUILD/lancer-frida.app"
    fi
else
    echo "      cargo-bundle not installed — skipping .app bundle."
    echo "      Install with: cargo install cargo-bundle"
fi

echo ""
echo "================================================"
echo "  Done!  Binary ready: $BUILD/$BIN"
echo "================================================"
echo ""
echo "  Runtime dependency: pip install frida-tools"
echo "  (provides frida, frida-ps, frida-trace, frida-ls-devices, ...)"

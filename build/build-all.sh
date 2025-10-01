#!/bin/bash
# Master build script - builds for all platforms
# This will build macOS, Windows, and Linux binaries

set -e

echo "🚀 Building hyper-mcp for all platforms..."
echo ""

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Run each platform build script
echo "═══════════════════════════════════════"
bash "$SCRIPT_DIR/build-macos.sh"
echo ""

echo "═══════════════════════════════════════"
bash "$SCRIPT_DIR/build-linux.sh"
echo ""

echo "═══════════════════════════════════════"
bash "$SCRIPT_DIR/build-windows.sh"
echo ""

echo "═══════════════════════════════════════"
echo "🎉 All platform builds completed!"
echo ""
echo "Build summary:"
echo "  macOS:   target/release-macos/{aarch64,x86_64}/"
echo "  Linux:   target/release-linux/{arm64,amd64}/"
echo "  Windows: target/release-windows/x86_64/"

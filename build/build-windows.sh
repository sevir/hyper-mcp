#!/bin/bash
# Build script for Windows (x86_64)
# Generates x86_64 release binary for Windows

set -e

echo "🪟 Building hyper-mcp for Windows..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get the project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Build for Windows x86_64
echo -e "${BLUE}Building for Windows (x86_64)...${NC}"
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu

# Create output directory
mkdir -p "$PROJECT_ROOT/target/release-windows/x86_64"

# Copy binary to organized folder
cp "$PROJECT_ROOT/target/x86_64-pc-windows-gnu/release/hyper-mcp.exe" \
   "$PROJECT_ROOT/target/release-windows/x86_64/hyper-mcp.exe"

echo -e "${GREEN}✅ Windows build completed successfully!${NC}"
echo ""
echo "Binary location:"
echo "  - x86_64: $PROJECT_ROOT/target/release-windows/x86_64/hyper-mcp.exe"
echo ""
echo "Individual target:"
echo "  - x86_64: $PROJECT_ROOT/target/x86_64-pc-windows-gnu/release/hyper-mcp.exe"

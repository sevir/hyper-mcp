#!/bin/bash
# Build script for macOS (Apple Silicon and Intel)
# Generates aarch64 (Apple Silicon) and x86_64 (Intel) release binaries

set -e

echo "🍎 Building hyper-mcp for macOS..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get the project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$PROJECT_ROOT"

# Build for Apple Silicon (aarch64)
echo -e "${BLUE}Building for macOS Apple Silicon (aarch64)...${NC}"
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Build for Intel (x86_64)
echo -e "${BLUE}Building for macOS Intel (x86_64)...${NC}"
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# Create output directories
mkdir -p "$PROJECT_ROOT/target/release-macos/aarch64"
mkdir -p "$PROJECT_ROOT/target/release-macos/x86_64"

# Copy binaries to organized folders
cp "$PROJECT_ROOT/target/aarch64-apple-darwin/release/hyper-mcp" \
   "$PROJECT_ROOT/target/release-macos/aarch64/hyper-mcp"

cp "$PROJECT_ROOT/target/x86_64-apple-darwin/release/hyper-mcp" \
   "$PROJECT_ROOT/target/release-macos/x86_64/hyper-mcp"

echo -e "${GREEN}✅ macOS builds completed successfully!${NC}"
echo ""
echo "Binaries location:"
echo "  - Apple Silicon: $PROJECT_ROOT/target/release-macos/aarch64/hyper-mcp"
echo "  - Intel:         $PROJECT_ROOT/target/release-macos/x86_64/hyper-mcp"
echo ""
echo "Individual targets:"
echo "  - Apple Silicon: $PROJECT_ROOT/target/aarch64-apple-darwin/release/hyper-mcp"
echo "  - Intel:         $PROJECT_ROOT/target/x86_64-apple-darwin/release/hyper-mcp"

#!/bin/bash
# Build script for Linux (AMD64 only)
# Generates amd64 (x86_64) release binary
# Note: ARM64 build is disabled

set -e

echo "🐧 Building hyper-mcp for Linux..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get the project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Build for Linux ARM64 (aarch64) - DISABLED
# echo -e "${BLUE}Building for Linux ARM64 (aarch64)...${NC}"
# rustup target add aarch64-unknown-linux-gnu
# cargo build --release --target aarch64-unknown-linux-gnu

# Build for Linux AMD64 (x86_64)
echo -e "${BLUE}Building for Linux AMD64 (x86_64)...${NC}"
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu

# Create output directories
# mkdir -p "$PROJECT_ROOT/target/release-linux/arm64"
mkdir -p "$PROJECT_ROOT/target/release-linux/amd64"

# Copy binaries to organized folders
# cp "$PROJECT_ROOT/target/aarch64-unknown-linux-gnu/release/hyper-mcp" \
#    "$PROJECT_ROOT/target/release-linux/arm64/hyper-mcp"

cp "$PROJECT_ROOT/target/x86_64-unknown-linux-gnu/release/hyper-mcp" \
   "$PROJECT_ROOT/target/release-linux/amd64/hyper-mcp"

echo -e "${GREEN}✅ Linux builds completed successfully!${NC}"
echo ""
echo "Binaries location:"
# echo "  - ARM64: $PROJECT_ROOT/target/release-linux/arm64/hyper-mcp"
echo "  - AMD64: $PROJECT_ROOT/target/release-linux/amd64/hyper-mcp"
echo ""
echo "Individual targets:"
# echo "  - ARM64: $PROJECT_ROOT/target/aarch64-unknown-linux-gnu/release/hyper-mcp"
echo "  - AMD64: $PROJECT_ROOT/target/x86_64-unknown-linux-gnu/release/hyper-mcp"

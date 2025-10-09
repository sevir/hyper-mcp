#!/bin/bash
# Build script for Linux (AMD64 and ARM64)
# Generates amd64 (x86_64) and arm64 (aarch64) release binaries

set -e

echo "🐧 Building hyper-mcp for Linux..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get the project root directory
PROJECT_ROOT="$(pwd)"
cd "$PROJECT_ROOT"


# Build for Linux AMD64 (x86_64)
echo -e "${BLUE}Building for Linux AMD64 (x86_64)...${NC}"
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu

# Create output directories
mkdir -p "$PROJECT_ROOT/target/release-linux/amd64"

# Copy binaries to organized folders

cp "$PROJECT_ROOT/target/x86_64-unknown-linux-gnu/release/hyper-mcp" \
   "$PROJECT_ROOT/target/release-linux/amd64/hyper-mcp"

echo -e "${GREEN}✅ Linux builds completed successfully!${NC}"
echo ""
echo "Binaries location:"
echo "  - AMD64: $PROJECT_ROOT/target/release-linux/amd64/hyper-mcp"
echo ""
echo "Individual targets:"
echo "  - AMD64: $PROJECT_ROOT/target/x86_64-unknown-linux-gnu/release/hyper-mcp"

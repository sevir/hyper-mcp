# Build Scripts Documentation for hyper-mcp

## Overview
Created a comprehensive build system for cross-platform compilation of hyper-mcp binaries.

## Structure
- **Location**: `/www/MCP/hyper-mcp/build/`
- **Scripts created**:
  - `build-macos.sh` - Builds for macOS (aarch64 and x86_64)
  - `build-linux.sh` - Builds for Linux (arm64 and amd64)
  - `build-windows.sh` - Builds for Windows (x86_64)
  - `build-all.sh` - Master script to build all platforms
  - `README.md` - Complete documentation

## Target Platforms
1. **macOS**:
   - aarch64-apple-darwin (Apple Silicon)
   - x86_64-apple-darwin (Intel)
   
2. **Linux**:
   - aarch64-unknown-linux-gnu (ARM64)
   - x86_64-unknown-linux-gnu (AMD64)
   
3. **Windows**:
   - x86_64-pc-windows-gnu

## Output Organization
Each script creates organized output directories:
- `target/release-macos/{aarch64,x86_64}/`
- `target/release-linux/{arm64,amd64}/`
- `target/release-windows/x86_64/`

Original Rust target directories are preserved in their standard locations.

## Usage
```bash
# Build for specific platform
./build/build-macos.sh
./build/build-linux.sh
./build/build-windows.sh

# Build for all platforms
./build/build-all.sh
```

## Features
- Automatic target installation via rustup
- Color-coded output for better visibility
- Organized output directories for easy distribution
- Comprehensive error handling with `set -e`
- Clear success messages with binary locations

## Prerequisites
- Rust toolchain (rustup)
- Cross-compilation tools for non-native targets
- MinGW-w64 for Windows builds on Linux/macOS
- gcc-aarch64-linux-gnu for ARM64 Linux builds

## Date Created
October 1, 2025

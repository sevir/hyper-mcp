# Build Scripts

This directory contains build scripts for cross-compiling `hyper-mcp` for different platforms and architectures.

## Available Scripts

### 🍎 macOS Build (`build-macos.sh`)
Builds release binaries for macOS:
- **aarch64** (Apple Silicon: M1, M2, M3, etc.)
- **x86_64** (Intel processors)

**Output directories:**
- `target/release-macos/aarch64/hyper-mcp`
- `target/release-macos/x86_64/hyper-mcp`

**Original target directories:**
- `target/aarch64-apple-darwin/release/hyper-mcp`
- `target/x86_64-apple-darwin/release/hyper-mcp`

### 🪟 Windows Build (`build-windows.sh`)
Builds release binary for Windows:
- **x86_64** (64-bit Intel/AMD)

**Output directory:**
- `target/release-windows/x86_64/hyper-mcp.exe`

**Original target directory:**
- `target/x86_64-pc-windows-gnu/release/hyper-mcp.exe`

### 🐧 Linux Build (`build-linux.sh`)
Builds release binaries for Linux:
- **arm64** (aarch64: Raspberry Pi, ARM servers, etc.)
- **amd64** (x86_64: standard 64-bit Intel/AMD)

**Output directories:**
- `target/release-linux/arm64/hyper-mcp`
- `target/release-linux/amd64/hyper-mcp`

**Original target directories:**
- `target/aarch64-unknown-linux-gnu/release/hyper-mcp`
- `target/x86_64-unknown-linux-gnu/release/hyper-mcp`

### 🚀 Build All (`build-all.sh`)
Runs all platform build scripts in sequence. This is the easiest way to build for all supported platforms at once.

## Usage

### Build for a specific platform:

```bash
# macOS
./build/build-macos.sh

# Windows
./build/build-windows.sh

# Linux
./build/build-linux.sh
```

### Build for all platforms:

```bash
./build/build-all.sh
```

## Prerequisites

### Required Tools
- **Rust toolchain** (install from [rustup.rs](https://rustup.rs/))
- **cargo** (included with Rust)

### Target Installation
The scripts will automatically install the required Rust targets using `rustup target add`. The following targets are used:

- `aarch64-apple-darwin` (macOS Apple Silicon)
- `x86_64-apple-darwin` (macOS Intel)
- `x86_64-pc-windows-gnu` (Windows)
- `aarch64-unknown-linux-gnu` (Linux ARM64)
- `x86_64-unknown-linux-gnu` (Linux AMD64)

### Cross-Compilation Notes

#### Building Windows binaries on Linux/macOS
You may need to install MinGW-w64:
- **macOS**: `brew install mingw-w64`
- **Ubuntu/Debian**: `sudo apt-get install mingw-w64`

#### Building Linux ARM64 binaries
You may need a cross-compiler:
- **Ubuntu/Debian**: `sudo apt-get install gcc-aarch64-linux-gnu`
- Configure cargo to use the cross-compiler by adding to `~/.cargo/config.toml`:
  ```toml
  [target.aarch64-unknown-linux-gnu]
  linker = "aarch64-linux-gnu-gcc"
  ```

## Output Structure

After running the build scripts, the binaries will be organized in the following structure:

```
target/
├── release-macos/
│   ├── aarch64/
│   │   └── hyper-mcp
│   └── x86_64/
│       └── hyper-mcp
├── release-linux/
│   ├── arm64/
│   │   └── hyper-mcp
│   └── amd64/
│       └── hyper-mcp
└── release-windows/
    └── x86_64/
        └── hyper-mcp.exe
```

Original Rust target directories are preserved in `target/{target-triple}/release/`.

## Troubleshooting

### Linker errors
If you encounter linker errors, ensure you have the appropriate cross-compilation tools installed for the target platform.

### Target not found
Run `rustup target list` to see available targets and `rustup target list --installed` to see which ones you have installed.

### Permission denied
Make sure the scripts are executable:
```bash
chmod +x build/*.sh
```

## Additional Resources

- [Rust Cross-compilation Guide](https://rust-lang.github.io/rustup/cross-compilation.html)
- [Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html)

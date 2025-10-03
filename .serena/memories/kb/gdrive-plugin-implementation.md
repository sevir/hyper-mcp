# Google Drive/Docs Plugin Implementation

## Overview

Implemented a Google Drive and Google Docs integration plugin for the hyper-mcp project. The plugin provides MCP tools for file management and document operations using Google's REST APIs.

## Implementation Details

### Location
`examples/plugins/gdrive/`

### Architecture
- **Language**: Rust
- **Compilation Target**: wasm32-unknown-unknown (WebAssembly)
- **Framework**: extism-pdk (Plugin Development Kit)
- **Authentication**: OAuth2 Bearer Token (pre-configured)

### Key Components

1. **lib.rs**: Main plugin implementation with 7 tools:
   - `list_files`: List Google Drive files with optional folder filtering
   - `get_file`: Get detailed file information
   - `create_document`: Create new Google Documents
   - `read_document`: Read document content
   - `append_to_document`: Append text to documents
   - `insert_text`: Insert text at specific positions
   - `search_files`: Search files by name

2. **pdk.rs**: Plugin Development Kit bindings (copied from existing plugins)

3. **Cargo.toml**: Dependencies include:
   - base64 = "0.22"
   - base64-serde = "0.8"
   - extism-pdk = "1.4.0"
   - serde + serde_json
   - urlencoding = "2.1"

### API Endpoints Used

- **Google Drive API v3**: `https://www.googleapis.com/drive/v3`
  - List files, get file info, search files
  
- **Google Docs API v1**: `https://docs.googleapis.com/v1`
  - Create documents, read content, batch updates

### Authentication Approach

**Challenge**: Full OAuth2 flow (with browser redirect and token storage) is incompatible with WASM plugin architecture:
- No file system access for token storage
- No ability to handle browser redirects
- Limited to HTTP requests within allowed hosts

**Solution**: Token-based authentication
- Users obtain OAuth2 access token externally (via OAuth2 Playground or official client libraries)
- Token passed as environment variable: `GOOGLE_ACCESS_TOKEN`
- Plugin uses Bearer token authentication for all API requests

**Required OAuth2 Scopes**:
- `https://www.googleapis.com/auth/drive`
- `https://www.googleapis.com/auth/documents`

### Configuration Example

```json
{
  "plugins": [
    {
      "name": "gdrive",
      "path": "oci://ghcr.io/sevir/hyper-mcp/plugin-gdrive:latest",
      "runtime_config": {
        "allowed_hosts": [
          "www.googleapis.com",
          "docs.googleapis.com"
        ],
        "env_vars": {
          "GOOGLE_ACCESS_TOKEN": "ya29.a0..."
        }
      }
    }
  ]
}
```

### Build Process

1. **Local Build**:
   ```bash
   rustup target add wasm32-unknown-unknown
   cargo build --release --target wasm32-unknown-unknown
   ```

2. **Docker Build**:
   - Multi-stage Dockerfile
   - Stage 1: Rust builder with wasm32 target
   - Stage 2: Scratch image with only the compiled WASM file

3. **Container Registry**:
   ```bash
   docker build -t ghcr.io/sevir/hyper-mcp/plugin-gdrive:latest .
   docker push ghcr.io/sevir/hyper-mcp/plugin-gdrive:latest
   ```

## Limitations and Trade-offs

### Current Limitations
1. **Token Refresh**: No automatic token refresh (tokens expire after ~1 hour)
2. **Binary Operations**: No file upload/download support
3. **Advanced Features**: Missing:
   - Rich text formatting
   - Sharing/permissions management
   - Comments
   - Revision history
   - Google Sheets/Slides integration

### Design Trade-offs
- **Simplicity vs Functionality**: Chose simpler token-based auth over complex OAuth2 flow
- **Scope**: Focused on common operations (read, write, search) vs comprehensive API coverage
- **Token Management**: External token refresh vs built-in refresh logic

## Comparison with Reference Implementations

### a-bonus/google-docs-mcp
- Full MCP server (not a plugin)
- Handles OAuth2 flow with local token storage
- More comprehensive (formatting, tables, comments, Drive management)
- Uses Node.js + TypeScript

### dev-ithitchhiker/mcp-google-docs
- Full MCP server (not a plugin)
- Supports Docs, Sheets, Slides, Drive
- Python implementation with fastmcp
- OAuth2 with local token storage

### This Plugin
- WASM plugin (not standalone server)
- Pre-configured token authentication
- Focused on core operations
- Rust + WebAssembly implementation
- No local storage capabilities

## Future Enhancements

Potential improvements:
1. Service account authentication support
2. Token refresh proxy service
3. Advanced formatting operations
4. Google Sheets basic operations
5. File permissions management
6. Comment operations
7. Binary file operations (with base64 encoding)

## Testing Recommendations

1. Obtain valid OAuth2 token from Google OAuth2 Playground
2. Test each operation individually
3. Monitor token expiration (typically 1 hour)
4. Verify allowed_hosts configuration
5. Check API rate limits

## Documentation

Complete README.md includes:
- Prerequisites and OAuth2 setup
- Configuration examples
- All 7 operations with parameter details
- Security considerations
- Error handling
- Troubleshooting guide

## Lessons Learned

1. **WASM Plugin Constraints**: File system and complex authentication flows are challenging in WASM context
2. **API Design**: Google's REST APIs are well-designed but require careful scope management
3. **Error Handling**: Comprehensive error handling is crucial for API integrations
4. **Documentation**: Clear setup instructions are essential when OAuth2 is involved
5. **Pragmatism**: Sometimes a simpler approach (token-based) is better than a perfect but impractical one (full OAuth2 flow)

## Build Status

✅ Successfully builds without errors
✅ All dependencies resolved
✅ WASM compilation successful
⚠️ Not runtime-tested (requires valid OAuth2 token)

## Date

Implemented: October 3, 2025

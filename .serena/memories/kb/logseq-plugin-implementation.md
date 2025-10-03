# Logseq Plugin Implementation for hyper-mcp

## Date
October 3, 2025

## Summary
Successfully implemented a Rust-based Logseq plugin for the hyper-mcp project that integrates with Logseq's local HTTP API to provide knowledge base access and manipulation capabilities.

## Implementation Details

### Plugin Location
`examples/plugins/logseq/`

### Key Features Implemented

1. **search_pages** - Search for pages by name or content using Datalog queries
2. **get_page_blocks** - Retrieve hierarchical block structures from pages
3. **get_block** - Get specific blocks with optional children recursively
4. **get_page_references** - Find all references to a page/tag with configurable depth
5. **get_page_content** - Extract complete page content with metadata

### Technology Stack
- **Language**: Rust
- **Target**: wasm32-wasip1 (WebAssembly)
- **Framework**: extism-pdk 1.4.1
- **Dependencies**: serde, serde_json for JSON handling

### Configuration
- `LOGSEQ_API_KEY` (required): API token from Logseq
- `LOGSEQ_BASE_URL` (optional): Default http://localhost:12315

### API Integration

The plugin uses Logseq's HTTP API (default port 12315) with the following methods:
- `logseq.Editor.getPage` - Page information retrieval
- `logseq.Editor.getBlock` - Block content with children
- `logseq.DB.datascriptQuery` - Datalog queries for searching

### Implementation Reference

Based on Python Gist: https://gist.github.com/digiogithub/0608dc6132ea5a18ca5249c076d951a1

The Rust implementation adapts the Python reference to use:
- extism-pdk for WebAssembly plugin development
- Native HTTP client for API calls
- Serde for JSON serialization/deserialization
- Markdown formatting for structured output

### File Structure
```
examples/plugins/logseq/
├── Cargo.toml          # Rust dependencies and build config
├── Dockerfile          # Multi-stage build for WebAssembly
├── README.md           # Usage documentation
└── src/
    ├── lib.rs          # Main plugin implementation
    └── pdk.rs          # MCP protocol types
```

### Build Process

```bash
cargo build --target wasm32-wasip1 --release
```

Output: `target/wasm32-wasip1/release/plugin.wasm`

### Docker Build

```bash
docker build -t ghcr.io/sevir/hyper-mcp/plugin-logseq:latest .
docker push ghcr.io/sevir/hyper-mcp/plugin-logseq:latest
```

### Key Design Decisions

1. **Hierarchical Output**: Blocks are formatted with indentation to show parent-child relationships
2. **Markdown Formatting**: All responses use markdown for readability
3. **Recursive References**: Optional depth parameter for following reference chains
4. **Default Values**: Sensible defaults (include_children: true, max_depth: 1)

### Integration with hyper-mcp

The plugin follows the standard hyper-mcp plugin pattern:
- Exports `list_tools()` for tool discovery
- Exports `call_tool()` for execution
- Uses standard MCP protocol types
- Configurable via runtime environment variables

### Testing Considerations

To test the plugin:
1. Ensure Logseq is running with HTTP API enabled
2. Get API token from Logseq settings
3. Configure plugin with token and base URL
4. Test with sample queries on your knowledge base

### Future Enhancements

Potential improvements:
- Add block creation/modification tools
- Support for tags and properties queries
- Advanced Datalog query builder
- Batch operations for multiple pages
- Export capabilities (PDF, HTML, etc.)

## Completion Status

✅ Plugin implementation complete
✅ Documentation (README.md) complete  
✅ Docker configuration complete
✅ Successfully builds to WebAssembly
✅ Follows hyper-mcp plugin patterns
✅ Configuration via environment variables
✅ All 5 tools implemented and documented

# Logseq Plugin

A hyper-mcp plugin that provides integration with Logseq's local HTTP API for reading and analyzing your knowledge base.

## Overview

This plugin allows you to interact with your Logseq knowledge base through its HTTP API. You can search pages, retrieve blocks, explore references, and build comprehensive views of your connected notes.

## Features

- Search pages by name or content
- Retrieve hierarchical block structures
- Get specific blocks with optional children
- Explore page references recursively
- Extract complete page content with metadata
- Build knowledge graphs by following references
- Explore a topic end-to-end: search a page, extract its references, and read all related pages in one call (`explore_page`)

## Prerequisites

Before using this plugin, you need to:

1. **Enable Logseq HTTP API**: 
   - Open Logseq application
   - Go to Settings → Features → Developer mode
   - Enable "HTTP APIs server"
   - The default endpoint is `http://localhost:12315`

2. **Get API Token**:
   - In the same settings, copy the API token
   - Click "Activate" to start the API server

## Configuration

The plugin requires the following configuration:

- `LOGSEQ_API_KEY`: (Required) Your Logseq API token
- `LOGSEQ_BASE_URL`: (Optional) Base URL for Logseq API (default: `http://localhost:12315`)

## Usage

```json
{
  "plugins": [
    {
      "name": "logseq",
      "path": "oci://ghcr.io/sevir/hyper-mcp/plugin-logseq:latest",
      "runtime_config": {
        "allowed_hosts": ["localhost:12315"],
        "env_vars": {
          "LOGSEQ_API_KEY": "your-api-key-here",
          "LOGSEQ_BASE_URL": "http://localhost:12315"
        }
      }
    }
  ]
}
```

## Available Tools

### 1. search_pages

Search for pages in Logseq by name or content.

**Parameters:**
- `query` (string, required): Search query for finding pages

**Example:**
```json
{
  "name": "search_pages",
  "arguments": {
    "query": "project"
  }
}
```

### 2. get_page_blocks

Get all blocks from a specific page with hierarchical structure.

**Parameters:**
- `page_name` (string, required): The name of the page to get blocks from

**Example:**
```json
{
  "name": "get_page_blocks",
  "arguments": {
    "page_name": "My Project"
  }
}
```

### 3. get_block

Get a specific block by ID with optional children.

**Parameters:**
- `block_id` (string, required): The ID or UUID of the block
- `include_children` (boolean, optional): Whether to include child blocks (default: true)

**Example:**
```json
{
  "name": "get_block",
  "arguments": {
    "block_id": "682cfd19-3c3f-427c-a0be-c5a3a197ea20",
    "include_children": true
  }
}
```

### 4. get_page_references

Get all blocks that reference a specific page or tag, with recursive depth support.

**Parameters:**
- `page_name` (string, required): The name of the page or tag to find references for
- `max_depth` (integer, optional): Maximum depth for recursive retrieval (default: 1, 0 for unlimited)

**Example:**
```json
{
  "name": "get_page_references",
  "arguments": {
    "page_name": "important-topic",
    "max_depth": 2
  }
}
```

**Use Cases:**
- Explore connected notes and knowledge graphs
- Find all mentions of a concept across your knowledge base
- Build comprehensive views of related information
- Analyze how topics are interconnected

### 5. get_page_content

Get the complete content of a page including metadata and all blocks.

**Parameters:**
- `page_name` (string, required): The name of the page to retrieve

**Example:**
```json
{
  "name": "get_page_content",
  "arguments": {
    "page_name": "Meeting Notes"
  }
}
```

### 6. explore_page

AI-friendly knowledge exploration in a single call: resolve a search term to the
best matching page, read its full content, extract the pages it references, and
recursively read those related pages up to a given depth. Returns one
consolidated markdown document — ideal for using your Logseq notes as a
navigable knowledge source.

**Parameters:**
- `query` (string, required): Page name or search term. Resolved to the best matching page (exact match first, then case-insensitive search).
- `depth` (integer, optional): How many reference hops to follow from the starting page (default: 1, recommended max: 2).
- `max_pages` (integer, optional): Maximum number of pages to read in total, to keep the result bounded (default: 10).
- `include_backlinks` (boolean, optional): Also follow pages that reference the starting page, not only outgoing references (default: false).

**Example:**
```json
{
  "name": "explore_page",
  "arguments": {
    "query": "Knowledge Management",
    "depth": 1,
    "max_pages": 10,
    "include_backlinks": false
  }
}
```

**What it does, step by step:**
1. Resolves `query` to a real page name.
2. Reads the page content as a hierarchical block tree (`getPageBlocksTree`).
3. Extracts the pages referenced by that page (outgoing `[[links]]` and `#tags`), and optionally the backlinks.
4. Reads each related page, following references breadth-first up to `depth`, skipping already-visited pages and stopping at `max_pages`.

**Use Cases:**
- "Give me everything related to X" in one request.
- Let an AI navigate connected notes without orchestrating multiple tool calls.
- Build a consolidated context document around a topic.

## Building and Publishing

### Build the Plugin

```bash
cd examples/plugins/logseq

# Build for WebAssembly
cargo build --target wasm32-wasip1 --release

# The plugin will be at: target/wasm32-wasip1/release/plugin.wasm
```

### Build Docker Image

```bash
# Build the image
docker build -t ghcr.io/sevir/hyper-mcp/plugin-logseq:latest .

# Push to GitHub Container Registry
docker push ghcr.io/sevir/hyper-mcp/plugin-logseq:latest
```

## Logseq API Reference

This plugin uses the following Logseq API methods:

- `logseq.Editor.getPage` - Retrieve page information
- `logseq.Editor.getBlock` - Get block content with children
- `logseq.Editor.getPageBlocksTree` - Get a page's blocks as a real parent/child tree
- `logseq.DB.datascriptQuery` - Query the Logseq database with Datalog (references, backlinks, search)

For more information about the Logseq API, see:
- [Logseq Plugin API Documentation](https://logseq.github.io/plugins/)
- [Logseq HTTP API Server Documentation](https://fossies.org/linux/logseq/resources/docs/api_server.html)

## Example Workflows

### 1. Knowledge Base Exploration

Search for a topic, then explore all references:

```json
// First, search for pages about "AI"
{
  "name": "search_pages",
  "arguments": { "query": "AI" }
}

// Then get references to understand connections
{
  "name": "get_page_references",
  "arguments": {
    "page_name": "AI",
    "max_depth": 2
  }
}
```

### 2. Extract Structured Content

Get complete content with full hierarchy:

```json
{
  "name": "get_page_content",
  "arguments": {
    "page_name": "Project Plan"
  }
}
```

### 3. Deep Dive into Specific Blocks

When you find an interesting block ID:

```json
{
  "name": "get_block",
  "arguments": {
    "block_id": "block-uuid-here",
    "include_children": true
  }
}
```

## Troubleshooting

### API Connection Issues

If you get connection errors:

1. Verify Logseq is running
2. Check that HTTP API server is enabled in settings
3. Confirm the API token is correct
4. Ensure the base URL matches your Logseq instance

### No Results Found

If searches return no results:

1. Check that the page or block exists in Logseq
2. Verify the spelling of page names (case-insensitive)
3. For references, ensure the page is actually referenced somewhere

### Permission Errors

The plugin needs network access to `localhost:12315`. Ensure this is allowed in your runtime configuration.

## Security Notes

- The API token should be kept secure and not shared
- The HTTP API only accepts requests from localhost by default
- All API requests must include the Bearer token in the Authorization header

## License

MIT License

## Tasks

### build:images

Build docker images of plugins

interactive: true

```bash
docker build -t ghcr.io/sevir/hyper-mcp/plugin-logseq:latest .
```

### push:images

Push docker images of plugins

interactive: true

```bash

docker push ghcr.io/sevir/hyper-mcp/plugin-logseq:latest
cosign sign --yes ghcr.io/sevir/hyper-mcp/plugin-logseq:latest
MANIFEST_TAG="ghcr.io/sevir/hyper-mcp/plugin-logseq:latest"
docker manifest create "$MANIFEST_TAG" "ghcr.io/sevir/hyper-mcp/plugin-logseq:latest" && docker manifest push "$MANIFEST_TAG" || echo "Skipping manifest for logseq"

``` 
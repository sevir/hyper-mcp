# Logseq Plugin

A hyper-mcp plugin that provides integration with Logseq's local HTTP API for reading and analyzing your knowledge base.

## Overview

This plugin allows you to interact with your Logseq knowledge base through its HTTP API. You can search pages, retrieve blocks, explore references, and build comprehensive views of your connected notes.

## Features

- Search across all pages and blocks for content
- Retrieve hierarchical block structures
- Get specific blocks with optional children
- Explore page references recursively
- Extract complete page content with metadata
- Build knowledge graphs by following references

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

Search for content across all Logseq pages and blocks.

**Parameters:**
- `query` (string, required): Search query for finding content

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
- `logseq.DB.datascriptQuery` - Query the Logseq database with Datalog

For more information about the Logseq API, see:
- [Logseq Plugin API Documentation](https://logseq.github.io/plugins/)
- [Logseq HTTP API Server Documentation](https://fossies.org/linux/logseq/resources/docs/api_server.html)

## Example Workflows

### 1. Knowledge Base Exploration

Search for a topic, then explore all references:

```json
// First, search for content about "AI"
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
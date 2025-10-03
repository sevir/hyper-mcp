# Brave Search Plugin

A hyper-mcp plugin that provides web search capabilities using the Brave Search API.

## Overview

This plugin allows you to perform web searches using Brave's independent search API. It returns formatted search results including titles, URLs, snippets, and discussion results when available.

## Prerequisites

Before using this plugin, you need to:

1. **Get Brave Search API Access**: Go to [Brave Search API Dashboard](https://api-dashboard.search.brave.com/)
2. **Create an Account** and get your API subscription token
3. **Choose a Plan**: Brave offers free tier with limited searches and paid plans for higher volumes

## Configuration

The plugin requires the following configuration:

- `BRAVE_API_KEY`: (Required) Your Brave Search API subscription token

## Usage

```json
{
  "plugins": [
    {
      "name": "brave-search",
      "path": "oci://ghcr.io/sevir/hyper-mcp/plugin-brave-search:latest",
      "runtime_config": {
        "allowed_hosts": ["api.search.brave.com"],
        "env_vars": {
          "BRAVE_API_KEY": "your-brave-api-key-here"
        }
      }
    }
  ]
}
```

## Available Operations

### Basic Search

```json
{
  "name": "brave_search",
  "arguments": {
    "query": "rust programming language"
  }
}
```

### Advanced Search with Options

```json
{
  "name": "brave_search",
  "arguments": {
    "query": "machine learning tutorials",
    "count": 15,
    "country": "US",
    "search_lang": "en",
    "safesearch": "moderate"
  }
}
```

## Parameters

### Required Parameters

- `query` (string): The search query string

### Optional Parameters
- `count` (optional): Number of results to return (1-20, default: 10)
- `offset` (optional): Pagination offset (default: 0)
- `country` (optional): Country code for search results (e.g., "US", "GB", "DE")
- `search_lang` (optional): Language code for search query (e.g., "en", "es", "fr")
- `ui_lang` (optional): Language code for user interface (e.g., "en", "es", "fr")
- `safesearch` (optional): SafeSearch setting ("strict", "moderate", "off")
- `freshness` (optional): Time-based freshness filter ("pd", "pw", "pm", "py")
- `result_filter` (optional): Filter for specific result types (comma-separated)

#### Example Usage

```json
{
  "name": "brave_search",
  "arguments": {
    "query": "Rust programming language",
    "api_key": "your_brave_api_token_here",
    "count": 5,
    "country": "US",
    "safesearch": "moderate"
  }
}
```

#### Response Format

The tool returns formatted text containing:

1. Original query information
2. Search results with:
   - Title
   - URL
   - Description
   - Page age (when available)
3. Discussion results (when available)

## API Documentation

This plugin uses the Brave Search Web Search API. For more details, see:
- [Brave Search API Documentation](https://api-dashboard.search.brave.com/app/documentation/web-search/query)
- [Brave Search API Response Objects](https://api-dashboard.search.brave.com/app/documentation/web-search/responses)

## Error Handling

The plugin handles various error scenarios:

- Invalid or missing API key
- Network connectivity issues
- API rate limiting
- Invalid parameters
- Malformed API responses

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Dependencies

- `extism-pdk`: Plugin development framework
- `serde`: JSON serialization/deserialization
- `serde_json`: JSON handling
- `urlencoding`: URL encoding for query parameters

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the same license as the hyper-mcp project.

## Tasks

### build:images

Build docker images of plugins

interactive: true

```bash
docker build -t ghcr.io/sevir/hyper-mcp/plugin-brave-search:latest .
```

### push:images

Push docker images of plugins

interactive: true

```bash

docker push ghcr.io/sevir/hyper-mcp/plugin-brave-search:latest
cosign sign --yes ghcr.io/sevir/hyper-mcp/plugin-brave-search:latest
MANIFEST_TAG="ghcr.io/sevir/hyper-mcp/plugin-brave-search:latest"
docker manifest create "$MANIFEST_TAG" "ghcr.io/sevir/hyper-mcp/plugin-brave-search:latest" && docker manifest push "$MANIFEST_TAG" || echo "Skipping manifest for brave-search"

``` 
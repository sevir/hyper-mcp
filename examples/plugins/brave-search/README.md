# Brave Search MCP Plugin

A Model Context Protocol (MCP) plugin that provides web search capabilities using the Brave Search API.

## Features

- Perform web searches using Brave Search API
- Configurable search parameters (count, country, language, safe search)
- Returns formatted search results with titles, URLs, and descriptions
- Includes discussion results when available
- Supports pagination and result filtering

## Prerequisites

- A Brave Search API subscription token from [Brave Search API Dashboard](https://api-dashboard.search.brave.com/)

## Installation

1. Ensure you have Rust installed (2024 edition recommended)
2. Clone the hyper-mcp repository
3. Navigate to the brave-search plugin directory:
   ```bash
   cd examples/plugins/brave-search
   ```
4. Build the plugin:
   ```bash
   cargo build --release
   ```

## Configuration

The plugin requires a Brave Search API subscription token. You can obtain one from the [Brave Search API Dashboard](https://api-dashboard.search.brave.com/).

## Usage

### Tool: `brave_search`

Performs a web search using the Brave Search API.

#### Parameters

- `query` (required): The search query string
- `api_key` (required): Your Brave Search API subscription token
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
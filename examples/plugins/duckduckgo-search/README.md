# DuckDuckGo Search Plugin

A hyper-mcp plugin that provides search capabilities using DuckDuckGo's Instant Answer API.

## Overview

This plugin allows you to perform searches using DuckDuckGo's public Instant Answer API. It returns instant answers, definitions, abstracts, and related topics without requiring any API key.

## Features

- No API key required - uses DuckDuckGo's public API
- Returns instant answers, definitions, and related topics
- Supports various output formats and response formatting options
- Provides abstracts, topics, and related information
- Safe search enabled by default

## Configuration

No configuration required - DuckDuckGo's API is free and open.

## Usage

```json
{
  "plugins": [
    {
      "name": "duckduckgo-search",
      "path": "oci://ghcr.io/sevir/hyper-mcp/plugin-duckduckgo-search:latest",
      "runtime_config": {
        "allowed_hosts": ["api.duckduckgo.com"]
      }
    }
  ]
}
```

## Available Operations

### Basic Search

```json
{
  "name": "duckduckgo_search",
  "arguments": {
    "query": "What is the capital of France?"
  }
}
```

### Advanced Search with Options

```json
{
  "name": "duckduckgo_search",
  "arguments": {
    "query": "machine learning definition",
    "format": "json",
    "pretty": true,
    "no_html": true
  }
}
```

## Parameters

### Required Parameters

- `query` (string): The search query string

### Optional Parameters

- `format` (string): Response format ("json" or "xml", default: "json")
- `pretty` (boolean): Pretty-print the JSON response (default: false)
- `no_html` (boolean): Remove HTML from text (default: false)
- `no_redirect` (boolean): Do not follow redirects (default: false)
- `skip_disambig` (boolean): Skip disambiguation (default: false)

```json
{
  "name": "duckduckgo_search",
  "arguments": {
    "query": "What is the capital of France?",
    "pretty": true
  }
}
```

#### Response Format

The tool returns formatted text containing:

1. **Query and Heading**: The original query and DuckDuckGo's interpretation
2. **Instant Answer**: Direct answers from DuckDuckGo's knowledge base
3. **Definition**: Dictionary definitions when applicable
4. **Related Topics**: Categorized related information and links
5. **Search Results**: Traditional search results when available

## API Documentation

This plugin uses DuckDuckGo's Instant Answer API. For more details, see:
- [DuckDuckGo Instant Answer API Documentation](https://duckduckgo.com/api)
- [DuckDuckGo API Parameters](https://api.duckduckgo.com/)

## Response Types

DuckDuckGo's API can return different types of responses:

- **Instant Answers**: Direct answers to factual questions
- **Definitions**: Dictionary definitions for terms
- **Related Topics**: Categorized links and information
- **Search Results**: Traditional web search results
- **Redirects**: When queries should redirect to specific pages

## Examples

### Simple Query
```json
{
  "query": "python programming"
}
```

### Definition Query
```json
{
  "query": "define photosynthesis",
  "pretty": true
}
```

### Technical Query
```json
{
  "query": "rust language features",
  "no_html": true
}
```

## Error Handling

The plugin handles various error scenarios:

- Invalid query parameters
- Network connectivity issues
- API response parsing errors
- Malformed JSON responses

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
- `chrono`: Date/time handling
- `base64-serde`: Base64 encoding support

## Privacy

This plugin uses DuckDuckGo's public API, which:
- Does not track users
- Does not store personal information
- Provides privacy-focused search results
- No API key or authentication required

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the same license as the hyper-mcp project.
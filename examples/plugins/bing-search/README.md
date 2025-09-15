# Bing Search Plugin

A hyper-mcp plugin that provides Bing Web Search functionality using the Microsoft Bing Search API.

## Overview

This plugin allows you to perform web searches using Microsoft's Bing Search API. It returns formatted search results including titles, URLs, snippets, and metadata such as last crawled dates and related searches.

## Prerequisites

Before using this plugin, you need to:

1. **Create an Azure Account**: Go to the [Azure Portal](https://portal.azure.com/)

2. **Create a Bing Search Resource**:
   - Search for "Bing Search v7" in the Azure Marketplace
   - Create a new Bing Search resource
   - Copy the API key from the "Keys and Endpoint" section

3. **Note your endpoint**: The default endpoint is `https://api.bing.microsoft.com/v7.0/search`

## Configuration

The plugin requires these parameters for each search:

- `api_key`: Your Bing Search API subscription key
- `query`: The search query string

## Usage

### Basic Search

```json
{
  "name": "bing_search",
  "arguments": {
    "query": "rust programming language",
    "api_key": "YOUR_SUBSCRIPTION_KEY_HERE"
  }
}
```

### Advanced Search with Options

```json
{
  "name": "bing_search",
  "arguments": {
    "query": "machine learning tutorials",
    "api_key": "YOUR_SUBSCRIPTION_KEY_HERE",
    "count": 20,
    "mkt": "en-US",
    "safe_search": "Moderate",
    "freshness": "Week",
    "set_lang": "en"
  }
}
```

## Parameters

### Required Parameters

- `query` (string): The search query
- `api_key` (string): Your Bing Search API subscription key

### Optional Parameters

- `count` (integer): Number of results to return (1-50, default: 10)
- `offset` (integer): Number of results to skip for pagination (default: 0)
- `mkt` (string): Market code (e.g., "en-US", "en-GB", "es-ES")
- `safe_search` (string): SafeSearch filter ("Off", "Moderate", "Strict")
- `freshness` (string): Date filter ("Day", "Week", "Month") or date range ("2024-01-01..2024-01-31")
- `response_filter` (string): Comma-separated response types ("Webpages", "Images", "News", etc.)
- `set_lang` (string): UI language code (e.g., "en", "es", "fr")

## Response Format

The plugin returns search results in a formatted text format including:

- Total number of results
- Search URL for full results on Bing
- Individual search results with:
  - Title
  - URL
  - Display URL
  - Snippet/description
  - Last crawled date
- Spelling suggestions (if applicable)
- Related searches

## Rate Limits and Quotas

Bing Search API has the following limits:

- **Free tier**: 1,000 queries/month, 3 calls/second
- **S1 tier**: 3 calls/second, higher monthly limits available
- **S2-S6 tiers**: Higher throughput limits

## Error Handling

The plugin handles various error conditions:

- Missing or invalid required parameters
- API authentication failures (invalid subscription key)
- Network errors
- Invalid API responses
- Rate limit exceeded

## Examples

### Search for Programming Tutorials

```json
{
  "name": "bing_search",
  "arguments": {
    "query": "python web development tutorials",
    "api_key": "your-subscription-key",
    "count": 15,
    "mkt": "en-US"
  }
}
```

### Recent News Search

```json
{
  "name": "bing_search",
  "arguments": {
    "query": "artificial intelligence news",
    "api_key": "your-subscription-key",
    "freshness": "Day",
    "count": 10
  }
}
```

### Safe Search for Educational Content

```json
{
  "name": "bing_search",
  "arguments": {
    "query": "science experiments for kids",
    "api_key": "your-subscription-key",
    "safe_search": "Strict",
    "mkt": "en-US"
  }
}
```

### International Search

```json
{
  "name": "bing_search",
  "arguments": {
    "query": "restaurantes en Madrid",
    "api_key": "your-subscription-key",
    "mkt": "es-ES",
    "set_lang": "es"
  }
}
```

## Building

To build this plugin:

```bash
cargo build --release
```

## Docker

You can also build and run using Docker:

```bash
# Build the Docker image
docker build -t bing-search-plugin .

# Run the container
docker run -p 3000:3000 bing-search-plugin
```

## Troubleshooting

### Common Issues

1. **401 Unauthorized**: Check that your subscription key is correct and active
2. **429 Too Many Requests**: You've exceeded your rate limit
3. **400 Bad Request**: Check your parameters for validity
4. **403 Forbidden**: Your subscription may not have access to this API

### Getting Help

- [Bing Search API Documentation](https://learn.microsoft.com/en-us/bing/search-apis/bing-web-search/)
- [Azure Bing Search Service](https://azure.microsoft.com/en-us/services/cognitive-services/bing-web-search-api/)
- [Azure Support](https://azure.microsoft.com/en-us/support/)

## Pricing

Bing Search API pricing varies by tier:

- **F1 (Free)**: 1,000 queries/month
- **S1**: $3/1,000 queries
- **S2-S6**: Higher volume discounts available

Check the [Azure Pricing Calculator](https://azure.microsoft.com/en-us/pricing/calculator/) for current pricing.

## License

This plugin is part of the hyper-mcp project and follows the same license terms.
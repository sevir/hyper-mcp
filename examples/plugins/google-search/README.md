# Google Search Plugin

A hyper-mcp plugin that provides Google Custom Search functionality using the Google Custom Search JSON API.

## Overview

This plugin allows you to perform web searches using Google's Custom Search JSON API. It returns formatted search results including titles, URLs, snippets, and metadata.

## Prerequisites

Before using this plugin, you need to:

1. **Create a Google Cloud Project**: Go to the [Google Cloud Console](https://console.cloud.google.com/)

2. **Enable the Custom Search JSON API**:
   - Go to "APIs & Services" > "Library"
   - Search for "Custom Search JSON API"
   - Click "Enable"

3. **Create API Credentials**:
   - Go to "APIs & Services" > "Credentials"
   - Click "Create Credentials" > "API Key"
   - Copy the generated API key

4. **Create a Custom Search Engine**:
   - Go to [Programmable Search Engine](https://cse.google.com/)
   - Click "New search engine"
   - Configure your search engine (you can search the entire web or specific sites)
   - Get your Search Engine ID from the "Setup" tab

## Configuration

The plugin requires these parameters for each search:

- `api_key`: Your Google Custom Search API key
- `search_engine_id`: Your Custom Search Engine ID (cx parameter)
- `query`: The search query string

## Usage

### Basic Search

```json
{
  "name": "google_search",
  "arguments": {
    "query": "rust programming language",
    "api_key": "YOUR_API_KEY_HERE",
    "search_engine_id": "YOUR_SEARCH_ENGINE_ID_HERE"
  }
}
```

### Advanced Search with Options

```json
{
  "name": "google_search",
  "arguments": {
    "query": "machine learning tutorials",
    "api_key": "YOUR_API_KEY_HERE",
    "search_engine_id": "YOUR_SEARCH_ENGINE_ID_HERE",
    "num": 5,
    "safe": "active",
    "lr": "lang_en",
    "gl": "us",
    "date_restrict": "m6"
  }
}
```

## Parameters

### Required Parameters

- `query` (string): The search query
- `api_key` (string): Your Google Custom Search API key
- `search_engine_id` (string): Your Custom Search Engine ID

### Optional Parameters

- `num` (integer): Number of results to return (1-10, default: 10)
- `start` (integer): Starting index for results (1-91, default: 1)
- `safe` (string): SafeSearch setting ("active" or "off")
- `lr` (string): Language restriction (e.g., "lang_en" for English)
- `gl` (string): Geolocation (two-letter country code, e.g., "us")
- `cr` (string): Country restriction
- `date_restrict` (string): Date restriction (e.g., "d7" for past 7 days, "m6" for past 6 months)
- `site_search` (string): Limit results to specific site
- `search_type` (string): Search type ("image" for image search only)

## Response Format

The plugin returns search results in a formatted text format including:

- Total number of results
- Search time
- Individual search results with:
  - Title
  - URL
  - Display link
  - Snippet/description
- Spelling suggestions (if applicable)

## Error Handling

The plugin handles various error conditions:

- Missing or invalid required parameters
- API authentication failures
- Network errors
- Invalid API responses

## Rate Limits and Quotas

Google Custom Search JSON API has the following limits:

- 100 search queries per day for the free tier
- 10,000 queries per day for paid plans
- Maximum 10 results per query
- Maximum 100 results total (across multiple pages)

## Building

To build this plugin:

```bash
cargo build --release
```

## Docker

You can also build and run using Docker:

```bash
# Build the Docker image
docker build -t google-search-plugin .

# Run the container
docker run -p 3000:3000 google-search-plugin
```

## Examples

### Search for Programming Tutorials

```json
{
  "name": "google_search",
  "arguments": {
    "query": "python web development tutorials",
    "api_key": "AIzaSy...",
    "search_engine_id": "0123456789abcdef",
    "num": 5,
    "safe": "active"
  }
}
```

### Image Search

```json
{
  "name": "google_search",
  "arguments": {
    "query": "rust logo",
    "api_key": "AIzaSy...",
    "search_engine_id": "0123456789abcdef",
    "search_type": "image",
    "num": 8
  }
}
```

### Site-Specific Search

```json
{
  "name": "google_search",
  "arguments": {
    "query": "documentation",
    "api_key": "AIzaSy...",
    "search_engine_id": "0123456789abcdef",
    "site_search": "docs.rs"
  }
}
```

## Troubleshooting

### Common Issues

1. **API_KEY_INVALID**: Check that your API key is correct and enabled
2. **SEARCH_ENGINE_ID_INVALID**: Verify your Custom Search Engine ID
3. **QUOTA_EXCEEDED**: You've reached your daily quota limit
4. **INVALID_REQUEST**: Check your parameters for validity

### Getting Help

- [Custom Search JSON API Documentation](https://developers.google.com/custom-search/v1/overview)
- [Programmable Search Engine Help](https://support.google.com/programmable-search/)
- [Google Cloud Console](https://console.cloud.google.com/)

## License

This plugin is part of the hyper-mcp project and follows the same license terms.
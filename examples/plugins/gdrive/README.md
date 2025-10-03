# Google Drive & Docs Plugin

A hyper-mcp plugin that provides integration with Google Drive and Google Docs APIs for file management and document operations.

## Overview

This plugin allows you to interact with Google Drive files and Google Docs documents through the MCP protocol. It provides tools for listing files, searching, creating documents, reading document content, and modifying documents.

## Features

- **File Management**: List, search, and get information about Google Drive files
- **Document Creation**: Create new Google Documents
- **Document Reading**: Read content from Google Documents
- **Document Editing**: Append text or insert text at specific positions
- **OAuth2 Token-based Authentication**: Uses pre-configured access tokens

## Prerequisites

Before using this plugin, you need to:

1. **Create a Google Cloud Project**: Go to [Google Cloud Console](https://console.cloud.google.com/)
2. **Enable APIs**: Enable the Google Drive API and Google Docs API
3. **Create OAuth 2.0 Credentials**: Create OAuth 2.0 client credentials
4. **Get Access Token**: Obtain an OAuth 2.0 access token through the standard OAuth2 flow

### OAuth2 Token Generation

This plugin requires a pre-configured OAuth2 access token. You can obtain this through various methods:

1. **Using Google OAuth2 Playground**: [https://developers.google.com/oauthplayground/](https://developers.google.com/oauthplayground/)
   - Select "Drive API v3" and "Docs API v1" scopes
   - Authorize and exchange authorization code for tokens
   - Copy the access_token

2. **Using the Google APIs Client Library**: Follow Google's official documentation for your preferred language

3. **Required Scopes**:
   - `https://www.googleapis.com/auth/drive`
   - `https://www.googleapis.com/auth/documents`

## Configuration

The plugin requires the following configuration:

- `GOOGLE_ACCESS_TOKEN`: (Required) Your Google OAuth2 access token

## Usage

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
          "GOOGLE_ACCESS_TOKEN": "your-google-access-token-here"
        }
      }
    }
  ]
}
```

## Available Operations

### List Files

List files from Google Drive with optional folder filtering.

```json
{
  "name": "list_files",
  "arguments": {
    "folder_id": "optional-folder-id",
    "page_size": 10
  }
}
```

### Search Files

Search for files by name in Google Drive.

```json
{
  "name": "search_files",
  "arguments": {
    "query": "project report",
    "page_size": 20
  }
}
```

### Get File Information

Get detailed information about a specific file.

```json
{
  "name": "get_file",
  "arguments": {
    "file_id": "1abc...xyz"
  }
}
```

### Create Document

Create a new Google Document.

```json
{
  "name": "create_document",
  "arguments": {
    "title": "My New Document"
  }
}
```

### Read Document

Read the content of a Google Document.

```json
{
  "name": "read_document",
  "arguments": {
    "document_id": "1abc...xyz"
  }
}
```

### Append to Document

Append text to the end of a Google Document.

```json
{
  "name": "append_to_document",
  "arguments": {
    "document_id": "1abc...xyz",
    "text": "This text will be appended to the document."
  }
}
```

### Insert Text

Insert text at a specific position in a Google Document.

```json
{
  "name": "insert_text",
  "arguments": {
    "document_id": "1abc...xyz",
    "text": "This text will be inserted.",
    "index": 1
  }
}
```

## Parameters

### list_files
- `folder_id` (optional): Folder ID to list files from
- `page_size` (optional): Number of files to return (default: 10, max: 100)

### search_files
- `query` (required): Search query string
- `page_size` (optional): Number of results (default: 10, max: 100)

### get_file
- `file_id` (required): The ID of the file

### create_document
- `title` (optional): Title of the new document (default: "Untitled Document")

### read_document
- `document_id` (required): The ID of the document to read

### append_to_document
- `document_id` (required): The ID of the document
- `text` (required): Text to append

### insert_text
- `document_id` (required): The ID of the document
- `text` (required): Text to insert
- `index` (optional): Position to insert (default: 1 for beginning)

## API Documentation

This plugin uses the following Google APIs:

- [Google Drive API v3](https://developers.google.com/drive/api/v3/reference)
- [Google Docs API v1](https://developers.google.com/docs/api/reference/rest)

## Token Refresh

**Important Note**: OAuth2 access tokens typically expire after 1 hour. For production use, you should:

1. Implement a token refresh mechanism external to this plugin
2. Regularly update the `GOOGLE_ACCESS_TOKEN` environment variable
3. Consider using service account credentials for server-to-server scenarios
4. Monitor for 401 Unauthorized responses indicating expired tokens

## Security Considerations

- **Token Storage**: Access tokens should be stored securely
- **Token Scope**: Use minimum required scopes for your use case
- **Token Expiration**: Implement proper token refresh handling
- **Credentials**: Never commit access tokens to version control

## Limitations

- This plugin uses pre-configured OAuth2 tokens (no automatic refresh)
- Token refresh must be handled externally
- Limited to basic file and document operations
- No support for:
  - File upload/download (binary content)
  - Advanced formatting operations
  - Sheets or Slides operations
  - Sharing and permissions management

## Examples

### List Files in a Folder

```json
{
  "name": "list_files",
  "arguments": {
    "folder_id": "1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs"
  }
}
```

### Create and Edit a Document

```json
// Step 1: Create a document
{
  "name": "create_document",
  "arguments": {
    "title": "Meeting Notes"
  }
}

// Step 2: Add content (use document_id from step 1)
{
  "name": "append_to_document",
  "arguments": {
    "document_id": "document-id-from-step-1",
    "text": "Meeting started at 10:00 AM\n\nAttendees: John, Jane, Bob"
  }
}
```

### Search and Read Documents

```json
// Step 1: Search for documents
{
  "name": "search_files",
  "arguments": {
    "query": "project"
  }
}

// Step 2: Read a found document
{
  "name": "read_document",
  "arguments": {
    "document_id": "document-id-from-search"
  }
}
```

## Error Handling

The plugin handles various error scenarios:

- Missing or invalid access token
- Expired tokens (401 responses)
- Network connectivity issues
- Invalid file/document IDs
- API rate limiting

## Building the Plugin

### Prerequisites

- Rust 1.70+ 
- Docker (for containerized builds)

### Local Build

```bash
cd examples/plugins/gdrive
cargo build --release --target wasm32-unknown-unknown
```

### Docker Build

```bash
docker build -t gdrive-plugin .
```

### Build and Push to GitHub Container Registry

```bash
# Build the image
docker build -t ghcr.io/sevir/hyper-mcp/plugin-gdrive:latest .

# Push to registry
docker push ghcr.io/sevir/hyper-mcp/plugin-gdrive:latest
```

## Troubleshooting

### "GOOGLE_ACCESS_TOKEN configuration is required"

Ensure you've set the `GOOGLE_ACCESS_TOKEN` environment variable in your plugin configuration.

### "API request failed with status 401"

Your access token has expired. Generate a new token using the OAuth2 flow.

### "API request failed with status 403"

Your token doesn't have the required scopes. Ensure you've authorized:
- `https://www.googleapis.com/auth/drive`
- `https://www.googleapis.com/auth/documents`

### "Missing or invalid required parameter"

Check that you're providing all required parameters for the operation you're trying to perform.

## License

This plugin is part of the hyper-mcp project.

## Contributing

Contributions are welcome! Please ensure:
- Code follows Rust best practices
- All tests pass
- Documentation is updated

## Future Enhancements

Potential improvements for future versions:

- Automatic token refresh capability
- Service account authentication support
- Binary file upload/download
- Advanced formatting options
- Google Sheets integration
- Google Slides integration
- Permissions and sharing management
- Comment management
- Revision history access

## Tasks

### build:images

Build docker images of plugins

interactive: true

```bash
docker build -t ghcr.io/sevir/hyper-mcp/plugin-gdrive:latest .
```

### push:images

Push docker images of plugins

interactive: true

```bash

docker push ghcr.io/sevir/hyper-mcp/plugin-gdrive:latest
cosign sign --yes ghcr.io/sevir/hyper-mcp/plugin-gdrive:latest
MANIFEST_TAG="ghcr.io/sevir/hyper-mcp/plugin-gdrive:latest"
docker manifest create "$MANIFEST_TAG" "ghcr.io/sevir/hyper-mcp/plugin-gdrive:latest" && docker manifest push "$MANIFEST_TAG" || echo "Skipping manifest for gdrive"

``` 
# YouTrack Plugin for hyper-mcp

A WebAssembly plugin for hyper-mcp that provides integration with JetBrains YouTrack for issue tracking and agile project management.

## Features

This plugin implements the same tools as the [YouTrack-mcp Python server](https://github.com/Digio-Campus/Youtrack-mcp), providing:

- **getTasksInformation**: Get comprehensive information about all in-progress tasks from an agile board
- **getIssueById**: Retrieve detailed information about a specific issue with all comments and metadata
- **createIssue**: Create new issues in YouTrack projects with custom fields support
- **updateIssue**: Update existing issues including summary, description, and custom fields
- **getCustomFields**: Discover available custom fields and their valid values for a project

## Configuration

### Environment Variables

- `YOUTRACK_BASE_URL`: Your YouTrack instance URL (e.g., `https://your-instance.youtrack.cloud/api`)
- `YOUTRACK_API_TOKEN`: Your YouTrack API token (permanent token)

### Getting a YouTrack API Token

1. Log in to your YouTrack instance
2. Click on your profile icon in the top right
3. Go to **Profile Settings** → **Authentication** → **Tokens**
4. Click **New token...**
5. Give it a name (e.g., "hyper-mcp integration")
6. Select the appropriate scope (e.g., "YouTrack")
7. Copy the generated token

## Usage

Add the plugin to your `config.json` or `config.yaml`:

### JSON Configuration

```json
{
    "plugins": [
        {
            "name": "youtrack",
            "path": "oci://ghcr.io/sevir/youtrack-plugin:latest",
            "runtime_config": {
                "allowed_hosts": [
                    "*.youtrack.cloud"
                ],
                "env_vars": {
                    "YOUTRACK_BASE_URL": "https://your-instance.youtrack.cloud/api",
                    "YOUTRACK_API_TOKEN": "perm:your-token-here"
                }
            }
        }
    ]
}
```

### YAML Configuration

```yaml
plugins:
  - name: youtrack
    path: oci://ghcr.io/sevir/youtrack-plugin:latest
    runtime_config:
      allowed_hosts:
        - "*.youtrack.cloud"
      env_vars:
        YOUTRACK_BASE_URL: "https://your-instance.youtrack.cloud/api"
        YOUTRACK_API_TOKEN: "perm:your-token-here"
```

## Available Tools

### getTasksInformation

Get information about all in-progress tasks from an agile board.

**Parameters:**
- `name` (required): The name of the agile board to query
- `num_comments` (optional, default: 1): Number of recent comments to include per task (0 for none)

**Example:**
```json
{
    "name": "Sprint 2024-08",
    "num_comments": 2
}
```

**Returns:**
A markdown-formatted report containing:
- Summary of total tasks in progress
- Table with task details (ID, summary, assignee, state, estimated/spent time, last update, comments)
- Recent comments for each task

### getIssueById

Get detailed information about a specific issue.

**Parameters:**
- `issue_id` (required): The issue ID (e.g., "DEMO-123" or internal ID like "3-3")

**Example:**
```json
{
    "issue_id": "DEMO-123"
}
```

**Returns:**
A detailed markdown report including:
- Basic information (ID, summary, state, priority, etc.)
- Full description
- All custom field values
- Complete comment history with timestamps

### createIssue

Create a new issue in a YouTrack project.

**Parameters:**
- `project` (required): Project short name or ID
- `summary` (required): Issue title/summary
- `description` (optional): Issue description
- `fields` (optional): JSON object with custom field values

**Example:**
```json
{
    "project": "DEMO",
    "summary": "Implement new feature",
    "description": "This feature needs to...",
    "fields": {
        "Priority": {"name": "High"},
        "Type": {"name": "Feature"}
    }
}
```

**Note:** Use `getCustomFields` first to discover available fields for your project.

### updateIssue

Update an existing issue.

**Parameters:**
- `issue_id` (required): The issue ID to update
- `summary` (optional): New summary
- `description` (optional): New description
- `fields` (optional): Custom fields to update

**Example:**
```json
{
    "issue_id": "DEMO-123",
    "summary": "Updated: Implement new feature",
    "fields": {
        "State": {"name": "In Progress"}
    }
}
```

### getCustomFields

Get all custom fields available for a project.

**Parameters:**
- `project` (required): Project short name or ID

**Example:**
```json
{
    "project": "DEMO"
}
```

**Returns:**
A markdown table listing all custom fields with:
- Field name
- Field type
- Possible values (for enum/state fields)

## Building from Source

### Prerequisites

- Rust 1.88+ with `wasm32-wasip1` target
- Docker (for containerized builds)

### Build Steps

```bash
# Install WASM target
rustup target add wasm32-wasip1

# Build the plugin
cargo build --release --target wasm32-wasip1

# The plugin will be at target/wasm32-wasip1/release/plugin.wasm
```

### Build with Docker

```bash
docker build -t youtrack-plugin .
```

### Publishing to Registry

```bash
# Build and push to GitHub Container Registry
docker build -t ghcr.io/sevir/youtrack-plugin:latest .
docker push ghcr.io/sevir/youtrack-plugin:latest
```

## Comparison with Python Implementation

This Rust/WASM plugin provides the same functionality as the [Python YouTrack-mcp server](https://github.com/Digio-Campus/Youtrack-mcp) with these benefits:

- **Performance**: Faster execution due to compiled WebAssembly
- **Portability**: Runs anywhere WASM is supported (cloud, edge, mobile, IoT)
- **Security**: Sandboxed execution with fine-grained permissions
- **Distribution**: Easy deployment via container registries
- **No Runtime**: No Python interpreter required

## Troubleshooting

### "Configuration error: YOUTRACK_BASE_URL configuration is required"

Make sure your `YOUTRACK_BASE_URL` is set correctly in the plugin configuration. It should include `/api` at the end (e.g., `https://your-instance.youtrack.cloud/api`).

### "Issue not found or access denied"

1. Verify your API token has the correct permissions
2. Check that the issue ID is correct
3. Ensure you have access to the project/issue

### "Board not found"

Make sure the board name exactly matches the name in YouTrack. Board names are case-sensitive.

### "Failed to fetch custom fields"

Verify:
1. The project short name/ID is correct
2. Your API token has admin permissions if accessing project settings
3. The project exists and you have access to it

## License

Apache-2.0

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Related Projects

- [hyper-mcp](https://github.com/tuananh/hyper-mcp) - The MCP server this plugin runs on
- [YouTrack-mcp](https://github.com/Digio-Campus/Youtrack-mcp) - Python implementation with the same tools
- [YouTrack REST API](https://www.jetbrains.com/help/youtrack/devportal/api-reference.html) - Official YouTrack API documentation

## Tasks

### build:images

Build docker images of plugins

interactive: true

```bash
docker build -t ghcr.io/sevir/hyper-mcp/plugin-youtrack:latest .
```

### push:images

Push docker images of plugins

interactive: true

```bash

docker push ghcr.io/sevir/hyper-mcp/plugin-youtrack:latest
cosign sign --yes ghcr.io/sevir/hyper-mcp/plugin-youtrack:latest
MANIFEST_TAG="ghcr.io/sevir/hyper-mcp/plugin-youtrack:latest"
docker manifest create "$MANIFEST_TAG" "ghcr.io/sevir/hyper-mcp/plugin-youtrack:latest" && docker manifest push "$MANIFEST_TAG" || echo "Skipping manifest for youtrack"

``` 
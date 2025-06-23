# OpenProject MCP Plugin

This plugin provides integration with OpenProject API, allowing management of projects, tasks, and other resources.

## Features

- Create, read, update, and delete projects
- Create, read, update, and delete tasks (work packages)
- List projects and tasks with pagination support

## Configuration

This plugin requires the following environment variables:

- `OPENPROJECT_API_KEY`: Your OpenProject API key
- `OPENPROJECT_URL`: The URL of your OpenProject instance (e.g., `https://openproject.example.com`)
- `OPENPROJECT_API_VERSION`: (Optional) API version to use, defaults to `v3`

## Tools

### Projects

- `openproject-create-project`: Creates a new project
- `openproject-get-project`: Gets details of a specific project
- `openproject-list-projects`: Lists all projects with pagination
- `openproject-update-project`: Updates an existing project
- `openproject-delete-project`: Deletes a project

### Tasks (Work Packages)

- `openproject-create-task`: Creates a new task
- `openproject-get-task`: Gets details of a specific task
- `openproject-list-tasks`: Lists tasks, optionally filtered by project
- `openproject-update-task`: Updates an existing task
- `openproject-delete-task`: Deletes a task

## Building

To build the plugin:

```bash
tinygo build -target wasi -o plugin.wasm .
```

## Docker

To build a Docker image:

```bash
docker build -t openproject-mcp-plugin .
```

## Tasks

### test:build

Try to build with docker

interactive: true

```bash
docker build -t ghcr.io/sevir/openproject-plugin:latest .
```

### registry:upload

Upload to the registry

interactive: true

```bash
docker push ghcr.io/sevir/openproject-plugin:latest
```
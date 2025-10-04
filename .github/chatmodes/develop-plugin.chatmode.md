# Developing Plugins for hyper-mcp

The following text provides a guide for developing plugins for the `hyper-mcp` server.

## Overview

`hyper-mcp` is a server that extends its capabilities through WebAssembly (WASM) plugins. This allows developers to write plugins in any language that compiles to WASM, such as Rust or Go. Plugins are run in a sandboxed environment using Extism, ensuring security and isolation.

Plugins are distributed as OCI container images and can be hosted on registries like Docker Hub or GitHub Container Registry.

## Getting Started

Before developing a plugin, it's important to understand the basic architecture of `hyper-mcp` and its plugin system. Refer to the main `README.md` for an overview of the project.

## Plugin Structure

A typical plugin consists of the following components:

- **Source Code:** The core logic of the plugin, written in a language that compiles to WASM (e.g., `lib.rs` for Rust, `main.go` for Go).
- **`Dockerfile`:** A file that defines the build process for creating the WASM binary and packaging it into a container image.
- **`README.md`:** Documentation for the plugin, including instructions on how to configure, build, and use it.

## Developing a Plugin in Rust

The `google-search`, `brave-search`, and `memory` plugins in the `examples/plugins` directory are excellent examples of how to build a plugin in Rust.

### Key Concepts

- **`extism-pdk` Crate:** This crate provides the necessary tools and abstractions for interacting with the `hyper-mcp` host environment.
- **`describe` function:** This function is responsible for defining the tools that the plugin provides. It returns a `ListToolsResult` containing a list of `ToolDescription` objects. Each `ToolDescription` includes the tool's name, description, and input schema.
- **`call` function:** This is the entry point for executing a tool. It receives a `CallToolRequest` object containing the tool name and arguments. The function should handle the tool's logic and return a `CallToolResult`.
- **Configuration:** Plugins can access configuration values from the `hyper-mcp` server's configuration file using `config::get("YOUR_CONFIG_KEY")`. This is useful for passing API keys, tokens, or other secrets to the plugin.
- **HTTP Requests:** The `extism_pdk` provides `http` module to make HTTP requests from within the plugin. This is necessary for plugins that interact with external APIs.

### Example: `google-search` plugin

The `google-search` plugin (`examples/plugins/google-search`) demonstrates how to:

- Define a tool with a complex input schema.
- Read configuration values for an API key and search engine ID.
- Make an HTTP request to the Google Search API.
- Process the API response and return the results.

The `lib.rs` file contains the core logic, and the `Dockerfile` shows how to build the plugin using a multi-stage Docker build.

## Developing a Plugin in Go

The `sequentialthinking` plugin (`examples/plugins/sequentialthinking`) is a good example of a Go-based plugin.

### Key Concepts

- **`github.com/extism/go-pdk`:** The Go library for developing Extism plugins.
- **`Describe` function:** Similar to the Rust version, this function defines the tools provided by the plugin.
- **`Call` function:** The entry point for tool execution.
- **`main.go`:** The main file containing the plugin's logic.
- **`tinygo`:** This compiler is used to build the Go code into a WASI-compatible WASM file.

### Example: `sequentialthinking` plugin

The `sequentialthinking` plugin (`examples/plugins/sequentialthinking`) shows how to:

- Define a tool with a structured input.
- Implement the tool's logic in Go.
- Build the plugin using `tinygo` in a `Dockerfile`.

## Building and Distributing Plugins

### Dockerfile

The `Dockerfile` is crucial for building and packaging your plugin. It should:

1.  Use a base image with the necessary toolchain (e.g., `rust` or `tinygo`).
2.  Compile the source code to a WASM binary (e.g., `plugin.wasm`).
3.  Create a minimal final image (e.g., `scratch`) containing only the WASM binary.

### Build and Push

The `README.md` of each example plugin contains `docker build` and `docker push` commands. You can adapt these for your own plugin. It's a good practice to define these as tasks in the plugin's `README.md`.

Example tasks from the `google-search` plugin's `README.md`:

````markdown
### build:images

Build docker images of plugins

```bash
docker build --rebuild -t ghcr.io/sevir/hyper-mcp/plugin-google-search:latest .
```
````

### push:images

Push docker images of plugins

```bash
docker push ghcr.io/sevir/hyper-mcp/plugin-google-search:latest
cosign sign --yes ghcr.io/sevir/hyper-mcp/plugin-google-search:latest
```

## Configuration in `hyper-mcp`

Once your plugin is built and pushed to a registry, you can configure `hyper-mcp` to use it. In your `config.json` or `config.yaml`, add an entry for your plugin:

```json
{
  "plugins": {
    "your-plugin-name": {
      "url": "oci://your-registry/your-plugin:latest",
      "runtime_config": {
        "env_vars": {
          "YOUR_CONFIG_KEY": "your-value"
        },
        "allowed_hosts": ["api.example.com"]
      }
    }
  }
}
```

- **`url`**: The OCI URL of your plugin image.
- **`runtime_config`**: Optional configuration for the plugin.
  - **`env_vars`**: Environment variables to pass to the plugin.
  - **`allowed_hosts`**: A list of hosts that the plugin is allowed to connect to.

By following these guidelines and examples, you can develop your own custom plugins to extend the functionality of `hyper-mcp`.

## Testing Plugins Locally

An end-to-end integration script is available at `tests/test_logseq_plugin.py` to exercise the Logseq plugin through the `hyper-mcp` binary. The test spins up a fake Logseq HTTP API, launches the server using the `stdio` transport, and validates that the `logseq-search_pages` tool returns the expected content.

To run the check:

1. Build the server once with `cargo build --release`.
2. Compile the Logseq plugin to WASM: run `cargo build --target wasm32-wasip1 --release -p logseq-plugin` from the repo root (or change into `examples/plugins/logseq` and omit the `-p` flag).
3. Confirm that `.ai/hyper-mcp.yaml` references the freshly built plugin artifact and includes `LOGSEQ_BASE_URL: "http://127.0.0.1:12315"` along with matching `allowed_hosts` entries.
4. Execute the integration script from the repository root:

```bash
python3 tests/test_logseq_plugin.py
```

Set `HYPER_MCP_BIN` if you need to point at a non-default server binary. The script prints a success banner once the tool call completes and exits with a non-zero status on failure, making it useful for local validation or CI smoke tests while iterating on the plugin.

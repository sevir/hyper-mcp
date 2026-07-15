# Serper Search Plugin

A hyper-mcp plugin that searches the web through the [Serper API](https://serper.dev/) and returns concise, formatted MCP text results.

## Features and prerequisites

- Google web search through Serper's `POST https://google.serper.dev/search` endpoint.
- Formatted answer-box, knowledge-graph, and organic-result output instead of raw JSON.
- A Serper account and API key are required.
- The API key is supplied through runtime configuration; it is never a tool argument.

## Configuration

The plugin reads one required Extism configuration value:

- `SERPER_API_KEY`: Serper API key. Store it in a secret manager or resolve it from the host environment; do not commit a real key.

The plugin only needs outbound access to `google.serper.dev`. Example host configuration:

```yaml
plugins:
  serper:
    url: oci://ghcr.io/sevir/serper-plugin:latest
    runtime_config:
      allowed_hosts:
        - google.serper.dev
      env_vars:
        SERPER_API_KEY: ${SERPER_API_KEY}
```

For local development, replace `url` with `file:///absolute/path/to/plugin.wasm`.

## Available tools

### `serper_web_search`

Searches the web for a query and returns formatted text.

Parameters:

- `q` (string, required): non-empty search query.

Invocation:

```json
{
  "name": "serper_web_search",
  "arguments": {
    "q": "Rust WebAssembly"
  }
}
```

The response may include an answer box, knowledge-graph information, and numbered organic results with titles, URLs, and snippets. Empty result sets are reported as text. Validation failures, missing configuration, transport failures, HTTP/API failures, and invalid JSON responses are returned as user-facing MCP error results.

## API and limitations

- Serper controls quotas, pricing, supported search options, and response availability. See the [Serper API documentation](https://serper.dev/docs) and [Serper API reference](https://serper.dev/api).
- The plugin currently sends only the required `q` field and uses Serper's web-search endpoint.
- Search results depend on Serper and Google availability and may change over time.

## Development

Prerequisites: Rust 1.90 and the `wasm32-wasip1` target.

From this directory:

```bash
rustup target add wasm32-wasip1
cargo fmt --check
cargo clippy --target wasm32-wasip1 -- -D warnings
cargo test
cargo build --release --target wasm32-wasip1
```

The WASM artifact is `target/wasm32-wasip1/release/plugin.wasm`.

## OCI image

The Dockerfile uses Rust 1.90 and installs the lockfile-resolved `cargo-auditable` dependencies. This avoids the newer Cargo dependencies that require Rust 1.91 or later.

```bash
docker build -t ghcr.io/sevir/serper-plugin:latest .
docker push ghcr.io/sevir/serper-plugin:latest
cosign sign --yes ghcr.io/sevir/serper-plugin:latest
```

Use signature verification for deployed images. Do not disable verification except for short-lived local development.

## Tasks

### build:images

```bash
docker build -t ghcr.io/sevir/serper-plugin:latest .
```

### push:images

```bash
docker push ghcr.io/sevir/serper-plugin:latest
cosign sign --yes ghcr.io/sevir/serper-plugin:latest
```

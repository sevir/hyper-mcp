# SerpApi Google Search Plugin

A Rust/WASM hyper-mcp plugin that searches Google through the official [SerpApi Google Search API](https://serpapi.com/search-api) and returns readable MCP text instead of raw JSON.

## Features

- Google search through `GET https://serpapi.com/search` with `engine=google`.
- Required query parameter `q` and documented safe search options.
- API key available only through runtime configuration; it is never accepted as a tool argument or printed in output.
- User-facing MCP errors for missing configuration, invalid arguments, HTTP failures, SerpApi API errors, malformed JSON, and unusable responses.
- Least-privilege network access: only `serpapi.com` is required.

## Prerequisites

- A SerpApi account and API key.
- Rust 1.90 or newer with the `wasm32-wasip1` target for local builds.
- A hyper-mcp host configured with the plugin's runtime settings.

## Configuration

Set `SERPAPI_API_KEY` through hyper-mcp `runtime_config.env_vars`. Prefer resolving it from the host environment rather than writing a secret in a configuration file:

```yaml
plugins:
  serpapi:
    url: oci://ghcr.io/sevir/serpapi-plugin:latest
    runtime_config:
      allowed_hosts:
        - serpapi.com
      env_vars:
        SERPAPI_API_KEY: ${SERPAPI_API_KEY}
```

For local development, replace `url` with `file:///absolute/path/to/plugin.wasm`. The host configuration key must be `serpapi` or another valid key using only letters, numbers, and single underscores.

The plugin reads the key only with `extism_pdk::config::get("SERPAPI_API_KEY")`. Do not pass an API key in a tool invocation. Never commit or log the key.

## Tool

### `serpapi_google_search`

Searches Google through SerpApi and returns the search status, metadata, organic results, and an answer box when present.

Required:

- `q` (string): non-empty Google search query.

Optional documented SerpApi/Google parameters:

- `location` (string): SerpApi location, such as `Austin, Texas, United States`.
- `google_domain` (string): Google domain, such as `google.com`.
- `gl` (string): two-letter country code.
- `hl` (string): two-letter language code.
- `device` (string): `desktop`, `mobile`, or `tablet`.
- `safe` (string): `active` or `off`.
- `filter` (string): `0` or `1`.
- `time_period` (string): SerpApi `tbs` value, such as `qdr:d`, `qdr:w`, or `qdr:m`.
- `udm` (string): Google vertical mode, such as `2` for images.
- `nfpr` (string): `0` or `1` to disable/enable auto-correction.
- `start` (integer): result offset from `0` through `1000`.
- `num` (integer): number of results from `1` through `100`.

Example:

```json
{
  "name": "serpapi_google_search",
  "arguments": {
    "q": "Rust WASM MCP plugins",
    "location": "Austin, Texas, United States",
    "hl": "en",
    "num": 10,
    "safe": "active"
  }
}
```

The response is readable text similar to:

```text
SerpApi Google Search Results
Status: Success
Total results: 123000

Organic results:
1. Example result
   URL: https://example.com
   A result snippet.
```

Raw JSON is never returned. Invalid optional values are rejected rather than silently sent to the API. SerpApi's current API limits, billing, result availability, and ranking behavior apply.

## Errors and limitations

- `SERPAPI_API_KEY` must be present in runtime configuration.
- Requests are sent only to `https://serpapi.com/search`.
- HTTP status failures and SerpApi error responses are returned as MCP errors with a bounded message.
- Successful responses must contain `organic_results`; an unexpected response shape is reported as a parse/response error.
- Only the response fields useful for readable search output are rendered: status, result count, search time, organic result title/URL/snippet/date, and answer-box text.

## Development

From this directory:

```bash
rustup target add wasm32-wasip1
cargo fmt --check
cargo clippy --target wasm32-wasip1 -- -D warnings
cargo test
cargo build --release --target wasm32-wasip1
```

The WASM artifact is `target/wasm32-wasip1/release/plugin.wasm`. `src/pdk.rs` is generated ABI support matching `plugin-schema.yaml`; do not hand-edit generated PDK bindings.

## OCI image

Build the scratch image. It installs lockfile-resolved `cargo-auditable` dependencies so Rust 1.90 remains compatible, and its final filesystem contains only `/plugin.wasm`:

```bash
docker build -t ghcr.io/sevir/serpapi-plugin:latest .
```

Publish only after authorization, then sign the image with Cosign:

```bash
docker push ghcr.io/sevir/serpapi-plugin:latest
cosign sign --yes ghcr.io/sevir/serpapi-plugin:latest
```

Verify the signed image through the repository's normal OCI verification workflow. Do not disable signature verification for final validation.

## Tasks

### build:images

```bash
docker build -t ghcr.io/sevir/serpapi-plugin:latest .
```

### push:images

```bash
docker push ghcr.io/sevir/serpapi-plugin:latest
cosign sign --yes ghcr.io/sevir/serpapi-plugin:latest
```

## API documentation

- [SerpApi Google Search API](https://serpapi.com/search-api)
- [SerpApi Google Search API reference](https://serpapi.com/google-search-api)

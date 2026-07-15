# SearchAPI.io Google Search

Rust WASM plugin for Google Search through the official SearchAPI.io `GET /api/v1/search` endpoint. Results are returned as readable text rather than raw JSON.

## Requirements

- A SearchAPI.io account and API key.
- The API key supplied at runtime as `SEARCHAPI_API_KEY`.
- Outbound access only to `www.searchapi.io`.

## Configuration

Use a host plugin configuration key with underscores, and resolve the secret from the host environment:

```yaml
plugins:
  searchapi_google:
    url: oci://ghcr.io/sevir/searchapi-google-plugin:latest
    runtime_config:
      allowed_hosts:
        - www.searchapi.io
      env_vars:
        SEARCHAPI_API_KEY: ${SEARCHAPI_API_KEY}
```

The API key is never accepted as a tool argument. For local development, replace `url` with `file:///absolute/path/to/plugin.wasm`.

## Tool

### `searchapi_google_search`

Searches Google with `engine=google`.

Required:

- `q` (string): non-empty search query.

Optional validated parameters:

- `page` (integer, 1–100): result page.
- `gl` (string): Google country code, such as `us`.
- `hl` (string): Google interface language code, such as `en`.
- `device` (string): device type, such as `desktop`, `mobile`, or `tablet`.
- `time_period` (string): `last_1_minute`, `last_5_minutes`, `last_15_minutes`, `last_30_minutes`, `last_hour`, `last_day`, `last_week`, `last_month`, or `last_year`.
- `safe` (string): `active`, `blur`, or `off`.

Example:

```json
{
  "name": "searchapi_google_search",
  "arguments": {
    "q": "Rust WASM plugins",
    "gl": "us",
    "hl": "en",
    "time_period": "last_month"
  }
}
```

The response includes the query, answer-box text when available, and each organic result's title, URL, and snippet. Missing configuration, invalid arguments, transport failures, non-2xx API responses, and invalid JSON are returned as user-facing MCP tool errors.

## Development

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

The Dockerfile installs lockfile-resolved `cargo-auditable` dependencies so Rust 1.90 remains compatible.

```bash
docker build -t ghcr.io/sevir/searchapi-google-plugin:latest .
docker push ghcr.io/sevir/searchapi-google-plugin:latest
cosign sign --yes ghcr.io/sevir/searchapi-google-plugin:latest
```

The scratch image contains only `/plugin.wasm`.

## Tasks

### build:images

```bash
docker build -t ghcr.io/sevir/searchapi-google-plugin:latest .
```

### push:images

```bash
docker push ghcr.io/sevir/searchapi-google-plugin:latest
cosign sign --yes ghcr.io/sevir/searchapi-google-plugin:latest
```

## API documentation

- [SearchAPI.io Google Search API](https://www.searchapi.io/docs/google)
- [SearchAPI.io API reference](https://www.searchapi.io/api/v1/search)
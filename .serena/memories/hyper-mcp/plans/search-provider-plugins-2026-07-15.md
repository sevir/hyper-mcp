---
created_at: 2026-07-15T09:50:31.759825448Z
updated_at: 2026-07-15T09:50:31.759825448Z
tags:
    - plan
    - plugins
    - rust
    - search
---
# Search provider plugin development plan

## Goal

Modernize the existing `serper` Rust plugin and add two documented Rust plugins:

1. `searchapi-google` for SearchAPI.io Google Search.
2. `serpapi` for SerpApi Google Search.

## Constraints

- Preserve unrelated working-tree files: `.pando.toml` and `.pando/` are untracked and out of scope.
- Use the `dev` workflow and Rust `wasm32-wasip1` target.
- Follow the mandatory `describe` / `call` ABI in `plugin-schema.yaml`.
- All source comments and documentation must be English.
- API keys are runtime configuration values read with `extism_pdk::config::get`, never tool arguments, logs, examples containing real values, or committed secrets.
- Use least-privilege `allowed_hosts`: `google.serper.dev`, `www.searchapi.io`, and `serpapi.com` respectively.
- Plugins should return MCP `ContentType::Text` formatted result summaries, not raw provider JSON.

## Workstreams

### Serper modernization

- Review current API documentation and existing source.
- Keep the documented Serper endpoint unless official documentation requires a change.
- Validate query and optional arguments.
- Surface provider HTTP/API errors as user-facing MCP error results.
- Parse the successful response and format organic results, result metadata, answer-box/knowledge-panel content where available, related questions, and related searches in readable text matching the established `google-search` style.
- Update manifest and README only when a concrete maintenance/API need is identified.

### SearchAPI.io Google plugin

- Create `examples/plugins/searchapi-google` with Cargo manifest/lockfile, WASI Dockerfile, Rust source, generated PDK support, and README.
- Implement a `google_search` tool using `GET https://www.searchapi.io/api/v1/search` with `engine=google`, `q`, and `SEARCHAPI_API_KEY` runtime configuration.
- Expose safe, documented Google options with strict validation and format common Google organic/result metadata as text.

### SerpApi plugin

- Create `examples/plugins/serpapi` with the same Rust/WASI packaging and a dedicated README.
- Implement a `google_search` tool using `GET https://serpapi.com/search.json`, `engine=google`, `q`, and `SERPAPI_API_KEY` runtime configuration.
- Expose documented optional Google parameters with strict validation and format common Google organic/result metadata as text.

## Validation

- Run `cargo fmt --check`, `cargo clippy --target wasm32-wasip1 -- -D warnings`, `cargo test`, and `cargo build --release --target wasm32-wasip1` for all three plugins.
- Inspect diagnostics and `git diff` / `git status`; leave unrelated files untouched.
- Reconcile the implementation and READMEs with the official provider documentation before completion.

---
created_at: 2026-07-15T09:54:23.462880244Z
updated_at: 2026-07-15T09:54:23.462880244Z
tags:
    - decision
    - plugins
    - rust
    - search
---
# Search provider plugin implementation decision

Implemented the plan in [[hyper-mcp/plans/search-provider-plugins-2026-07-15.md]].

## Decisions

- The existing `serper` plugin remains on Serper's documented `POST https://google.serper.dev/search` endpoint and still reads `SERPER_API_KEY` from Extism runtime configuration. Its successful output is now formatted MCP text rather than raw JSON.
- `searchapi-google` uses SearchAPI.io's documented `GET https://www.searchapi.io/api/v1/search` endpoint with `engine=google`. It sends its runtime-only `SEARCHAPI_API_KEY` as a Bearer authorization header, which avoids placing credentials in the request URL. Current SearchAPI documentation marks `num` as phased out, so it is intentionally not exposed.
- `serpapi` uses SerpApi's documented canonical `GET https://serpapi.com/search` endpoint with `engine=google` and runtime-only `SERPAPI_API_KEY`.
- Each plugin has a dedicated English README, WASI Docker packaging containing only `/plugin.wasm`, input validation, formatted text output, API error handling, and least-privilege allowed-host guidance.

## Validation

All three changed/new plugins passed format checks, clippy with warnings denied for `wasm32-wasip1`, unit tests, and release WASM builds. Root validation remains recorded with the worktree review.

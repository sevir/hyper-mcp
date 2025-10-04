# Hyper-MCP Logseq Integration Test Notes

## Date
October 5, 2025

## Scope
Captures the workflow and technical gotchas uncovered while creating the end-to-end Python integration test for the Logseq plugin in the `hyper-mcp` repository.

## Key Artifacts
- **Test Harness**: `tests/test_logseq_plugin.py`
- **Runtime Config**: `.ai/hyper-mcp.yaml`
- **Documentation Update**: `.github/chatmodes/develop-plugin.chatmode.md` ("Testing Plugins Locally" section)

## What the Script Does
- Spins up a fake Logseq HTTP API server with deterministic responses for `logseq-search_pages`.
- Launches the `hyper-mcp` binary over the stdio transport and speaks the MCP JSON-RPC protocol (version `2024-11-05`).
- Advertises standard client capabilities (logging, prompts, tools) during `initialize`.
- Discovers the Logseq plugin via `list_tools` and executes `logseq-search_pages`.
- Asserts that the tool reply matches the mocked page title `"Prácticas en empresa 25"`, then performs graceful shutdown of both the MCP server and the fake HTTP server.

## Handshake / Protocol Lessons
- MCP stdio transport expects **newline-delimited JSON**; buffering a full JSON object without the trailing newline prevents the server from parsing requests.
- The initial `initialize` request must include a `capabilities` object (even if empty) to avoid schema validation errors.
- Use `"mcp_version": "2024-11-05"` to match the server's supported protocol version.

## Runtime Configuration Requirements
- `.ai/hyper-mcp.yaml` must point the Logseq plugin to the locally built WASM artifact and expose the fake API endpoint:
  - `LOGSEQ_BASE_URL: "http://127.0.0.1:12315"`
  - `allowed_hosts` must include `"127.0.0.1"`, `"localhost"`, and matching port variants so the plugin's HTTP client can reach the stub server.
- Build prerequisites before running the test:
  1. `cargo build --release` (server binary).
  2. `cargo build --target wasm32-wasip1 --release -p logseq-plugin` (plugin artifact).

## How to Run the Test
```bash
python3 tests/test_logseq_plugin.py
```
The script prints a success banner when the tool call returns the mocked payload and exits non-zero on failure, making it CI-friendly.

## Troubleshooting Checklist
- **Connection closes immediately**: verify newline framing and that the server binary path is correct.
- **HTTP error from plugin**: ensure `allowed_hosts` covers the fake server domain/port.
- **Schema errors**: confirm the `initialize` payload includes `capabilities` and `protocol_version` fields.
- **Tool missing in `list_tools`**: check `.ai/hyper-mcp.yaml` points to the freshly compiled WASM file and that the plugin build succeeded.

## Status
✅ Integration script passes against the mocked Logseq API and documents the required build/configuration steps for future plugin authors.

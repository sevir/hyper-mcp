# Logseq plugin PDK type update (2025-10-05)

## Summary
- Updated `examples/plugins/logseq/src/pdk.rs` to align the plugin's Extism bindings with other generated plugins.
- Added `extism_pdk::{FromBytes, ToBytes}` derives and `#[encoding(Json)]` to all request/response structs and enums so they convert cleanly between WASM host and plugin.
- Made `CallToolRequest.method` optional (matching other plugins) to avoid deserialization failures when the host omits it.
- Introduced a regression test verifying that a request without `method` still deserializes and preserves the tool arguments.
- Confirmed the plugin builds for `wasm32-wasip1` and passes unit tests.

## Notes
- The new unit test lives in `examples/plugins/logseq/src/pdk.rs` and checks `CallToolRequest` parsing.
- Running the plugin against a live Logseq instance still requires `LOGSEQ_API_KEY`/`LOGSEQ_BASE_URL` and an accessible HTTP API.
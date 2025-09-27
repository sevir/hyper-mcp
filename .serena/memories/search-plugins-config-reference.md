# Search Plugins Configuration Reference

This memory contains the configuration variables required for each search plugin after the recent update.

## Configuration Variables by Plugin

| Plugin | Configuration Variables | Container Path |
|--------|------------------------|----------------|
| bing-search | `BING_API_KEY` | `oci://ghcr.io/sevir/hyper-mcp/plugin-bing-search:latest` |
| google-search | `GOOGLE_API_KEY`, `GOOGLE_SEARCH_ENGINE_ID` | `oci://ghcr.io/sevir/hyper-mcp/plugin-google-search:latest` |
| brave-search | `BRAVE_API_KEY` | `oci://ghcr.io/sevir/hyper-mcp/plugin-brave-search:latest` |
| perplexity-search | `PERPLEXITY_API_KEY` | `oci://ghcr.io/sevir/hyper-mcp/plugin-perplexity-search:latest` |
| duckduckgo-search | None (public API) | `oci://ghcr.io/sevir/hyper-mcp/plugin-duckduckgo-search:latest` |
| gitlab | `GITLAB_TOKEN`, `GITLAB_URL` (optional) | `oci://ghcr.io/tuananh/gitlab-plugin:latest` |

## Pattern Used
All plugins follow the same pattern as GitLab:
1. Configuration function: `get_{plugin}_config()` using `config::get()`
2. Error handling for missing configuration
3. Removed API keys from function parameters
4. Updated tool descriptions to remove API key requirements
5. Standardized README documentation format

## Benefits Achieved
- Centralized configuration management
- Better security (no API keys in function calls)
- Consistent user experience across all plugins
- Standardized documentation
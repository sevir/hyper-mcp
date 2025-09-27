# Search Plugins Configuration Update Summary

## Overview
Updated all search plugins in the hyper-mcp project to use configuration variables (similar to the GitLab plugin) instead of requiring API keys as parameters in each function call.

## Plugins Updated

### 1. bing-search
- **Configuration required**: `BING_API_KEY`
- **Container path**: `oci://ghcr.io/sevir/hyper-mcp/plugin-bing-search:latest`
- **Changes**:
  - Added `get_bing_config()` function to retrieve config using `config::get()`
  - Removed `api_key` parameter from function signature
  - Updated tool description to remove `api_key` from required parameters
  - Updated README.md with new configuration format

### 2. google-search
- **Configuration required**: `GOOGLE_API_KEY`, `GOOGLE_SEARCH_ENGINE_ID`
- **Container path**: `oci://ghcr.io/sevir/hyper-mcp/plugin-google-search:latest`
- **Changes**:
  - Added `get_google_config()` function returning both API key and search engine ID
  - Removed `api_key` and `search_engine_id` parameters from function signature
  - Updated tool description to remove both parameters from required list
  - Updated README.md with new configuration format

### 3. brave-search
- **Configuration required**: `BRAVE_API_KEY`
- **Container path**: `oci://ghcr.io/sevir/hyper-mcp/plugin-brave-search:latest`
- **Changes**:
  - Added `get_brave_config()` function to retrieve API key
  - Removed `api_key` parameter from function signature
  - Updated tool description and README.md

### 4. perplexity-search
- **Configuration required**: `PERPLEXITY_API_KEY`
- **Container path**: `oci://ghcr.io/sevir/hyper-mcp/plugin-perplexity-search:latest`
- **Changes**:
  - Added `get_perplexity_config()` function to retrieve API key
  - Removed `api_key` parameter from function signature
  - Updated tool description and README.md

### 5. duckduckgo-search
- **Configuration required**: None (uses public API)
- **Container path**: `oci://ghcr.io/sevir/hyper-mcp/plugin-duckduckgo-search:latest`
- **Changes**:
  - Updated README.md to follow new format (no code changes needed as it doesn't use API keys)

## Configuration Format
All READMEs now include the standardized configuration format:

```json
{
  "plugins": [
    {
      "name": "plugin-name",
      "path": "oci://ghcr.io/sevir/hyper-mcp/plugin-{plugin-name}:latest",
      "runtime_config": {
        "allowed_hosts": ["api-host.com"],
        "env_vars": {
          "API_KEY_NAME": "your-api-key-here"
        }
      }
    }
  ]
}
```

## Benefits
1. **Improved Security**: API keys are now stored in configuration rather than passed as parameters
2. **Better User Experience**: Users don't need to provide API keys with every function call
3. **Consistency**: All plugins now follow the same pattern as the GitLab plugin
4. **Documentation**: All READMEs standardized with clear configuration examples and OCI container paths

## Testing
All plugins compile successfully after changes, confirming the refactoring was completed without introducing build errors.
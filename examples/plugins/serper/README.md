# serper

A plugin that performs Google web search using the [Serper](https://serper.dev) API and returns the raw JSON response for the given query string.

## Requirements

- Set `SERPER_API_KEY` in your config to your Serper API key.

## Usage

Call with:
```json
{
  "plugins": [
    {
      "name": "serper",
      "path": "oci://ghcr.io/tuananh/serper-plugin:latest",
      "runtime_config": {
        "env_vars": {
          "SERPER_API_KEY": "<your-serper-api-key>"
        },
        "allowed_hosts": ["google.serper.dev"] 
      }
    }
  ]
}
```

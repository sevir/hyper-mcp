# New feature development plan

This is a plan for developing a new feature in the hyper-mcp project, adding and developing new plugins.

In the path `examples/plugins/context7` there is an example of a plugin in Rust that can be used as a starting point. In the README.md in the root path of the project there are instructions on how to build with a Dockerfile. 

- [x] Implement a new plugin in Rust for search in Google Search using Custom Search JSON API. Search in internet and Context7 for libraries and how to do it. The folder `examples/plugins/google-search` contains the scaffold of the plugin. Also write a README.md file with instructions on how to use it into this path.
- [x] Implement a new plugin in Rust for search using Bing Search API. The folder `examples/plugins/bing-search` contains the scaffold of the plugin. Also write a README.md file with instructions on how to use it into this path.
- [x] Implement a new plugin in Rust for search using Brave Search API, the OpenAI specs is in https://github.com/automation-ai-labs/mcp-link/raw/refs/heads/main/examples/brave.yaml. The folder `examples/plugins/brave-search` contains the scaffold of the plugin. Also write a README.md file with instructions on how to use it into this path.
- [x] Implement a new plugin in Rust for search using DuckDuckGo Search API, the OpenAI specs is in https://github.com/automation-ai-labs/mcp-link/raw/refs/heads/main/examples/duckduckgo.yaml. The folder `examples/plugins/duckduckgo-search` contains the scaffold of the plugin. Also write a README.md file with instructions on how to use it into this path.
- [x] Implement a new plugin in Rust for search in Perplexity Search. Search in internet and Context7 for libraries and how to do it. The folder `examples/plugins/perplexity-search` contains the scaffold of the plugin. Also write a README.md file with instructions on how to use it into this path.
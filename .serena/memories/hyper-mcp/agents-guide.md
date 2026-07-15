---
created_at: 2026-07-15T07:33:20.550380055Z
updated_at: 2026-07-15T07:33:20.550380055Z
tags:
    - architecture
    - development
    - plugins
    - documentation
---
# hyper-mcp AGENTS guide

Created root `AGENTS.md` for Sevir's `hyper-mcp` fork. It records the `dev` branch workflow, Rust host architecture, plugin ABI (`describe` and `call`), supported loading schemes, Extism runtime permissions, and configuration naming constraints.

It includes fast-reference plugin patterns for Rust (`think`, `perplexity-search`, `logseq`, `holded`, `youtrack`, XTP examples, WASI native dependencies) and Go (`crypto-price`, `github`, `openproject`, `sequentialthinking`), plus directory structure, build commands, Docker packaging requirements, README checklist, local testing, CI, GHCR publication, Cosign signing, and release image naming.

Important publishing distinction: the automated release workflow publishes `ghcr.io/sevir/<directory-name>-plugin:<tag|latest>`, while older manually added Sevir plugins use `ghcr.io/sevir/hyper-mcp/plugin-<plugin-name>:latest`; retain existing names and use the release convention for new plugins. The required OCI artifact path is `/plugin.wasm`.
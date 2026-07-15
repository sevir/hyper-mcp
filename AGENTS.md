# AGENTS.md

## Repository identity and workflow

- This repository is Sevir's fork: `github.com/sevir/hyper-mcp`; the upstream remote is `tuananh/hyper-mcp`.
- Work on branch `dev`. CI runs for pushes and pull requests targeting `dev`; do not use `main` as the integration target.
- Use English for source code, tool descriptions, and repository documentation.
- Preserve unrelated working-tree changes. In particular, inspect `git status --short` before editing or staging.
- The root server is Rust 1.90 (`rust-toolchain.toml`, edition 2024). Plugins are independent projects beneath `examples/plugins/<plugin-name>/`; build them from their own directory or with `--manifest-path`.

## Architecture

`hyper-mcp` is an MCP server implemented with `rmcp`. It discovers each plugin's tools by calling two mandatory WASM exports:

- `describe`: returns `ListToolsResult` containing tool name, description, and JSON input schema.
- `call`: receives `CallToolRequest` (`params.name`, optional `params.arguments`) and returns `CallToolResult`.

Runtime flow:

1. `src/config.rs` loads JSON, YAML, or TOML configuration and validates plugin names.
2. `src/plugins.rs` fetches WASM from `file://`, `http(s)://`, `oci://`, or `s3://`.
3. OCI images must contain the WASM module at `/plugin.wasm`; OCI modules are cached locally.
4. Extism instantiates the module with the configured network, filesystem, memory, and configuration permissions.
5. `describe` results are exposed as namespaced MCP tools: `<plugin-config-name>-<tool-name>`.
6. Tool calls are routed to the matching plugin's `call` export.

Read before changing host behavior:

- `src/main.rs`: CLI, transports (`stdio`, `sse`, `streamable-http`).
- `src/config.rs`: plugin config format, naming rules, runtime configuration.
- `src/plugins.rs`: WASM loading, Extism manifest permissions, tool routing/namespacing.
- `src/oci.rs`: OCI pull, cache, and signature verification.
- `plugin-schema.yaml`: precise ABI schema for `call` and `describe`.
- `RUNTIME_CONFIG.md`: runtime permissions, auth, and secret-management guidance.

## Plugin configuration contract

Use a valid configuration key: alphanumeric characters and single underscores only; it cannot start/end with `_`, contain `__`, hyphens, spaces, or special characters. Directory, crate, OCI image, and displayed names may use hyphens, but the **host plugin configuration key** cannot.

```yaml
plugins:
  example_plugin:
    url: oci://ghcr.io/sevir/hyper-mcp/plugin-example-plugin:latest
    runtime_config:
      allowed_hosts:
        - api.example.com
      allowed_paths:
        - /optional/host/path
      env_vars:
        EXAMPLE_API_KEY: ${EXAMPLE_API_KEY}
      memory_limit: 128Mi
      skip_tools:
        - internal_.*
```

Rules:

- `url` also accepts legacy alias `path`.
- `allowed_hosts` is mandatory for outbound plugin HTTP access; grant only the API hosts actually required, never `*` unless explicitly justified.
- `allowed_paths` grants the WASM module access to exact host paths. Avoid it unless necessary.
- `env_vars` are passed to Extism as plugin config values, not OS environment variables. Rust reads them with `extism_pdk::config::get`; Go reads them with `pdk.GetConfig`.
- `${NAME}` in an `env_vars` value resolves from the host environment. Never commit secrets to configuration examples, READMEs, tests, or source.
- `memory_limit` is parsed by the host (for example `128Mi`).
- `skip_tools` uses full-match regexes and filters exposed and callable tools.
- Local development uses `file:///absolute/path/to/plugin.wasm`.

OCI images are signature-verified by default. Images published through the supported workflow are signed with Cosign. Do not recommend `--insecure-skip-signature` outside short-lived local development.

## Creating a plugin

1. Choose an existing reference matching the use case and language.
2. Create `examples/plugins/<plugin-name>/` with source, dependency manifest/lockfile, `Dockerfile`, and a dedicated `README.md`.
3. Implement `describe` and `call` exactly as defined in `plugin-schema.yaml`.
4. Define accurate JSON schemas: `type: "object"`, `properties`, and `required` (use `[]` when no parameter is required).
5. Validate all arguments and return a structured `CallToolResult` with `isError: true` and text content for recoverable user/configuration errors.
6. Keep secrets in runtime configuration and declare minimum `allowed_hosts` / `allowed_paths` in the usage example.
7. Build the WASM module, then build an OCI image containing **only** `/plugin.wasm` in its scratch final stage.
8. Update the plugin README in the same change; document configuration, every tool, build, test, OCI publication, and a runnable host configuration.

The root `README.md` documents XTP initialization (`xtp plugin init --schema-file plugin-schema.yaml`). Some Rust examples contain `xtp.toml`, `prepare.sh`, and generated `src/pdk.rs`; use the existing pattern if working with XTP. Do not hand-edit generated PDK bindings (`pdk.rs` / `pdk.gen.go`).

## Rust plugins (preferred)

References:

- Minimal local/tool schema: `examples/plugins/think/`.
- External API, runtime configuration, HTTP, comprehensive README: `examples/plugins/perplexity-search/`.
- Production-quality Sevir API integration and README tasks: `examples/plugins/logseq/`, `examples/plugins/holded/`, `examples/plugins/youtrack/`.
- Generated XTP setup: `examples/plugins/google-search/` and `examples/plugins/duckduckgo-search/`.
- WASI native dependencies: `examples/plugins/sqlite/` and `examples/plugins/memory/`.

Expected structure:

```text
examples/plugins/<plugin-name>/
├── Cargo.toml
├── Cargo.lock                 # commit when generated
├── Dockerfile
├── README.md
└── src/
    ├── lib.rs                 # business logic: call(), describe(), tool handlers
    └── pdk.rs                 # generated when using XTP; do not edit
```

`Cargo.toml` requirements:

```toml
[lib]
name = "plugin"
crate-type = ["cdylib"]

[dependencies]
extism-pdk = "..."
serde = { version = "...", features = ["derive"] }
serde_json = "..."
```

Implementation conventions:

- Dispatch in `call(input)` on `input.params.name`.
- `describe()` returns stable tool names without the plugin prefix; the host adds it.
- Use `extism_pdk::config::get("KEY")` for runtime secrets/configuration.
- Use Extism's `http` API for outbound requests; the matching host must be granted in `allowed_hosts`.
- Return `ContentType::Text` and user-oriented error text for normal validation/API failures.
- Compile for `wasm32-wasip1`; do not use legacy `wasm32-wasi` or `wasm32-unknown-unknown` for new plugins. A few legacy README commands are stale; the Dockerfiles and CI establish `wasm32-wasip1` as the current target.

Commands from the plugin directory:

```bash
rustup target add wasm32-wasip1
cargo fmt --check
cargo clippy --target wasm32-wasip1 -- -D warnings
cargo test
cargo build --release --target wasm32-wasip1
# output: target/wasm32-wasip1/release/plugin.wasm
```

Use this Dockerfile pattern, adapting only the Rust version if the plugin requires it:

```dockerfile
FROM rust:1.90-slim AS builder
LABEL org.opencontainers.image.source=https://github.com/sevir/hyper-mcp

RUN rustup target add wasm32-wasip1 && \
    rustup component add rust-std --target wasm32-wasip1 && \
    cargo install cargo-auditable

WORKDIR /workspace
COPY . .
RUN cargo fetch
RUN cargo auditable build --release --target wasm32-wasip1

FROM scratch
WORKDIR /
COPY --from=builder /workspace/target/wasm32-wasip1/release/plugin.wasm /plugin.wasm
```

For C/native dependencies, follow `sqlite`/`memory` and configure a WASI SDK and `CC_wasm32_wasip1` rather than assuming native compilation works.

## Go plugins

References:

- Simple pattern and build instructions: `examples/plugins/crypto-price/`.
- Larger multi-tool organization: `examples/plugins/github/` and `examples/plugins/openproject/`.
- Stateful tool: `examples/plugins/sequentialthinking/`.

Expected structure:

```text
examples/plugins/<plugin-name>/
├── go.mod
├── go.sum
├── Dockerfile
├── README.md
├── main.go                    # Call(), Describe(), dispatch
└── pdk.gen.go                 # generated binding; do not edit
```

Conventions:

- Use `github.com/extism/go-pdk`.
- Export `Call` and `Describe`; retain `func main() {}` in the generated binding/project.
- Dispatch on `input.Params.Name`; names must correspond to those returned by `Describe`.
- Retrieve configuration using `pdk.GetConfig("KEY")` and perform outbound HTTP through the PDK.
- Build with TinyGo and WASI:

```bash
tinygo build -target wasi -o plugin.wasm .
docker build -t ghcr.io/sevir/hyper-mcp/plugin-<plugin-name>:latest .
```

Docker final stage must copy `plugin.wasm` to `/plugin.wasm`; see `crypto-price/Dockerfile`.

## README requirements for every plugin

Use `logseq/README.md`, `holded/README.md`, or `youtrack/README.md` as the most complete templates. Keep all information current with source and Dockerfile.

Required sections:

1. Title and concise purpose.
2. Features and external prerequisites (account, API/API version, permissions, local service where relevant).
3. Configuration: exact config keys, required/optional status, secret handling, minimum `allowed_hosts`, and an `oci://` usage example.
4. Available tools: one subsection per tool with purpose, parameter name/type/requiredness/default, and JSON invocation example.
5. Behavior, errors, limitations, and API links where useful.
6. Development: toolchain prerequisites, WASM build command and expected artifact.
7. OCI build and publication commands, including Cosign signing.
8. A `## Tasks` section using `build:images` and `push:images`, matching the current image name.

Do not show real tokens in README examples. Ensure tool names, required fields, defaults, configuration keys, target triple, and registry image exactly match the code/Dockerfile.

## Build, test, and local validation

Root server commands:

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test --workspace --all-features
cargo build
cargo build --release
```

`just fmt -- --check` and `just clippy` are available convenience commands. CI runs root clippy, formatting, workspace tests, root build, and a WASI build of `google-search`; still build every changed plugin explicitly.

For a local integration test:

1. Build the server and plugin WASM.
2. Point a local config at the module with `file:///absolute/path/to/plugin.wasm`.
3. Grant only the required runtime permissions.
4. Run `target/debug/hyper-mcp --config-file /absolute/path/to/config.yaml` (or `cargo run -- --config-file ...`) and inspect tools through an MCP client.

`tests/test_logseq_plugin.py` is an end-to-end reference. It requires `.ai/hyper-mcp.yaml`, a built server binary, a freshly built Logseq module, and validates a namespaced tool through stdio. Run it only when its prerequisites are present:

```bash
python3 tests/test_logseq_plugin.py
```

## GitHub and GHCR publication

Development changes go to `dev` and are reviewed/merged according to the repository workflow. Do not publish from a feature branch without explicit authorization.

Preferred repository-release path:

- Push a version tag (`v*`) only when creating a release.
- `.github/workflows/release.yml` builds every `examples/plugins/*/` directory, pushes GHCR images, and keylessly signs them with Cosign.
- The release workflow's canonical image format is `ghcr.io/<repository-owner>/<directory-name>-plugin:<tag>` and `:latest`. For this fork it is typically `ghcr.io/sevir/<plugin-name>-plugin:<tag>`.

Existing manually added Sevir plugins often use `ghcr.io/sevir/hyper-mcp/plugin-<plugin-name>:latest`. Keep a plugin's existing image naming consistent unless the user explicitly requests migration. For a **new** plugin, use `<plugin-name>-plugin`, matching the automated release workflow.

Manual publication (only after authorization and with a GitHub token permitted to write packages):

```bash
export IMAGE="ghcr.io/sevir/<plugin-name>-plugin:<version-or-latest>"
echo "$GITHUB_TOKEN" | docker login ghcr.io -u "$GITHUB_ACTOR" --password-stdin
docker build -t "$IMAGE" examples/plugins/<plugin-name>
docker push "$IMAGE"
cosign sign --yes "$IMAGE"
```

For an existing plugin that retains the older convention, set `IMAGE` to `ghcr.io/sevir/hyper-mcp/plugin-<plugin-name>:<version-or-latest>` instead.

After publication:

- Add/update the README OCI URL and host config example.
- Pull/load the signed OCI module with a clean local config to confirm `/plugin.wasm`, permissions, `describe`, and one `call` work.
- Do not disable signature checks for the final validation.

## Change checklist

- [ ] Branch is `dev`; unrelated changes remain untouched.
- [ ] Plugin ABI exposes working `describe` and `call`.
- [ ] Tool schemas, validation, error results, and runtime config keys agree with README.
- [ ] Network/filesystem permissions are least privilege; no secret is committed or logged.
- [ ] WASM build succeeds for the appropriate target.
- [ ] Docker image final stage contains `/plugin.wasm`.
- [ ] Root checks and changed-plugin checks pass.
- [ ] README includes configuration, tools, build, publication, and Sevir GHCR image commands.
- [ ] OCI image is signed and verified before declaring release readiness.

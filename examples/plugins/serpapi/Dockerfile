FROM rust:1.90-slim AS builder
LABEL org.opencontainers.image.source=https://github.com/sevir/hyper-mcp

RUN rustup target add wasm32-wasip1 && \
    rustup component add rust-std --target wasm32-wasip1 && \
    cargo install cargo-auditable --locked

WORKDIR /workspace
COPY . .
RUN cargo fetch
RUN cargo auditable build --release --target wasm32-wasip1

FROM scratch
WORKDIR /
COPY --from=builder /workspace/target/wasm32-wasip1/release/plugin.wasm /plugin.wasm

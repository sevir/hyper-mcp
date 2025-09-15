FROM rust:1.88-slim AS builder

RUN rustup target add wasm32-wasip1 && \
    rustup component add rust-std --target wasm32-wasip1 && \
    cargo install cargo-auditable

# Install wasi-sdk
ENV WASI_OS=linux \
    WASI_VERSION=25 \
    WASI_VERSION_FULL=25.0

# Detect architecture and set WASI_ARCH accordingly
RUN apt-get update && apt-get install -y wget && \
    ARCH=$(uname -m) && \
    if [ "$ARCH" = "x86_64" ]; then \
        export WASI_ARCH=x86_64; \
    elif [ "$ARCH" = "aarch64" ]; then \
        export WASI_ARCH=arm64; \
    else \
        echo "Unsupported architecture: $ARCH" && exit 1; \
    fi && \
    cd /opt && \
    wget https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${WASI_VERSION}/wasi-sdk-${WASI_VERSION_FULL}-${WASI_ARCH}-${WASI_OS}.tar.gz && \
    tar xvf wasi-sdk-${WASI_VERSION_FULL}-${WASI_ARCH}-${WASI_OS}.tar.gz && \
    rm wasi-sdk-${WASI_VERSION_FULL}-${WASI_ARCH}-${WASI_OS}.tar.gz && \
    mv wasi-sdk-${WASI_VERSION_FULL}-${WASI_ARCH}-${WASI_OS} wasi-sdk

WORKDIR /workspace
COPY . .
RUN cargo fetch
ENV WASI_SDK_PATH=/opt/wasi-sdk
ENV CC_wasm32_wasip1="${WASI_SDK_PATH}/bin/clang --sysroot=${WASI_SDK_PATH}/share/wasi-sysroot"

RUN cargo auditable build --release --target wasm32-wasip1

FROM scratch
WORKDIR /
COPY --from=builder /workspace/target/wasm32-wasip1/release/plugin.wasm /plugin.wasm

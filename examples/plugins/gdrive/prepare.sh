#!/bin/bash

set -e

echo "Preparing gdrive plugin build..."

# Install wasm32-unknown-unknown target if not already installed
rustup target add wasm32-unknown-unknown

echo "Build preparation complete!"

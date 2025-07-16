#!/usr/bin/env bash
set -euo pipefail

# Build frontend first, then Rust
echo "Building complete application..."

# Build frontend
"$(dirname "$0")/build-frontend.sh"

# Build Rust
echo "Building Rust application..."
cd "$(dirname "$0")/.."
cargo build --release
echo "Complete build finished!"
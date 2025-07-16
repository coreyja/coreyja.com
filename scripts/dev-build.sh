#!/usr/bin/env bash
set -euo pipefail

# Quick development build (unoptimized)
echo "Starting development build..."

# Build frontend
"$(dirname "$0")/build-frontend.sh"

# Build Rust in debug mode (faster)
echo "Building Rust application (debug mode)..."
cd "$(dirname "$0")/.."
cargo build
echo "Development build complete!"
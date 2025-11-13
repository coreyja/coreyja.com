#!/usr/bin/env bash
set -euo pipefail

# Build Rust application
echo "Building complete application..."
cd "$(dirname "$0")/.."
cargo build --release
echo "Complete build finished!"
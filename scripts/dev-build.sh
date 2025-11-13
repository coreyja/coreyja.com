#!/usr/bin/env bash
set -euo pipefail

# Quick development build (unoptimized)
echo "Starting development build..."
cd "$(dirname "$0")/.."
cargo build
echo "Development build complete!"
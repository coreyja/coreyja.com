#!/usr/bin/env bash
set -euo pipefail

# Build the frontend application
echo "Building frontend..."
cd "$(dirname "$0")/../thread-frontend"
npm install
npm run build
echo "Frontend build complete!"
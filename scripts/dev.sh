#!/usr/bin/env bash
# Development script for Forge

set -e

echo "🔨 Building Forge workspace..."
cargo build

echo "✅ Build complete!"
echo ""
echo "Available commands:"
echo "  cargo run --bin forge -- daemon     # Start the daemon"
echo "  cargo run --bin forge -- version    # Show version"
echo "  cd ui/app && npm run dev            # Start UI dev server"

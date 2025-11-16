#!/usr/bin/env bash
# Build script for Forge

set -e

echo "🔨 Building Forge in release mode..."
cargo build --release --workspace

echo "📦 Building UI..."
cd ui/app
npm run build
cd ../..

echo "✅ Build complete!"
echo "Binary location: target/release/forge"

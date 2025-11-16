#!/usr/bin/env bash
# Test script for Forge

set -e

echo "🧪 Running Forge tests..."
cargo test --workspace

echo "✅ All tests passed!"

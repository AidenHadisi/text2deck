#!/bin/bash

# Text2Deck Build Script

set -e

echo "🚀 Building Text2Deck..."

# Build the web frontend
echo "📦 Building web frontend..."
cd web
wasm-pack build --target web --out-dir pkg
cd ..

# Test the worker
echo "🧪 Testing worker..."
cd worker
cargo test
cd ..

echo "✅ Build complete!"
echo ""
echo "Next steps:"
echo "1. Deploy worker: cd worker && wrangler deploy"
echo "2. Or test locally: cd worker && wrangler dev"
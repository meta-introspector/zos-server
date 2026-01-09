#!/bin/bash
set -e

echo "🔍 Running Rust linting and formatting..."

echo "📝 Checking format..."
cargo fmt --check

echo "📎 Running clippy (allowing warnings)..."
cargo clippy --all-targets --all-features

echo "🧪 Running tests..."
cargo test --all-features

echo "✅ All checks passed!"

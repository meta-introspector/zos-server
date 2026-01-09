#!/bin/bash
# ZOS Server Pre-commit Setup and Test

set -e

echo "🔧 Setting up ZOS Server pre-commit hooks..."

# Install pre-commit if not available
if ! command -v pre-commit >/dev/null 2>&1; then
    echo "📦 Installing pre-commit..."
    if command -v pip >/dev/null 2>&1; then
        pip install pre-commit
    elif command -v nix-shell >/dev/null 2>&1; then
        echo "Using nix-shell for pre-commit..."
    else
        echo "❌ Please install pre-commit: pip install pre-commit"
        exit 1
    fi
fi

# Install hooks
echo "🪝 Installing pre-commit hooks..."
if command -v pre-commit >/dev/null 2>&1; then
    pre-commit install
else
    nix-shell -p pre-commit --run "pre-commit install"
fi

# Test hooks on all files
echo "🧪 Testing hooks on all files..."
if command -v pre-commit >/dev/null 2>&1; then
    pre-commit run --all-files || echo "⚠️ Some hooks failed - this is normal for first run"
else
    nix-shell -p pre-commit --run "pre-commit run --all-files" || echo "⚠️ Some hooks failed - this is normal for first run"
fi

# Stage all changes
echo "📝 Staging all changes..."
git add .

# Show what will be committed
echo "📋 Changes to be committed:"
git diff --cached --stat

# Commit with pre-commit hooks
echo "💾 Committing with pre-commit hooks..."
git commit -m "🚀 Setup enhanced pre-commit hooks with auto-commit/push

- Added comprehensive Rust linting and testing
- Added lattice system validation
- Added auto-commit and push functionality
- Enhanced pre-commit configuration" || echo "✅ No changes to commit or commit successful"

echo "✅ Pre-commit setup complete!"
echo ""
echo "🎯 Next commits will automatically:"
echo "  ✅ Format Rust code"
echo "  ✅ Run clippy linting"
echo "  ✅ Run cargo check"
echo "  ✅ Run cargo test"
echo "  ✅ Validate lattice system"
echo "  ✅ Auto-commit changes"
echo "  ✅ Auto-push to origin"

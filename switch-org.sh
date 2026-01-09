#!/bin/bash
# Script to update GitHub Actions to use meta-introspector org

set -e

if [ "$1" = "meta-introspector" ]; then
    echo "🔄 Switching to meta-introspector org..."
    sed -i 's/ACTIONS_ORG: actions/ACTIONS_ORG: meta-introspector/' .github/workflows/build.yml
    sed -i 's/RUST_ORG: dtolnay/RUST_ORG: meta-introspector/' .github/workflows/build.yml
    sed -i 's/CACHE_ORG: Swatinem/CACHE_ORG: meta-introspector/' .github/workflows/build.yml
    echo "✅ Updated to use meta-introspector org"
elif [ "$1" = "standard" ]; then
    echo "🔄 Switching to standard orgs..."
    sed -i 's/ACTIONS_ORG: meta-introspector/ACTIONS_ORG: actions/' .github/workflows/build.yml
    sed -i 's/RUST_ORG: meta-introspector/RUST_ORG: dtolnay/' .github/workflows/build.yml
    sed -i 's/CACHE_ORG: meta-introspector/CACHE_ORG: Swatinem/' .github/workflows/build.yml
    echo "✅ Updated to use standard orgs"
else
    echo "Usage: $0 [meta-introspector|standard]"
    echo "Current settings:"
    grep "_ORG:" .github/workflows/build.yml
fi

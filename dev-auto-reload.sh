#!/bin/bash
echo "🚀 Starting ZOS Dev Server daemon with auto-reload..."
echo "This will background itself and replace any running instances"
echo "Edit files in src/ to see auto-reload in action"
echo ""

./target/debug/zos-dev-minimal

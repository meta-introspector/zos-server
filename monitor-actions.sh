#!/bin/bash
# Monitor GitHub Actions and fix issues

set -e

echo "🔍 Monitoring GitHub Actions for ZOS Server"
echo "============================================"

# Check if we can access GitHub API
if ! command -v gh &> /dev/null; then
    echo "⚠️  GitHub CLI not found. Install with: sudo apt install gh"
    echo "📋 Manual check: https://github.com/meta-introspector/zos-server/actions"
    exit 1
fi

# Check authentication
if ! gh auth status &> /dev/null; then
    echo "🔐 GitHub CLI not authenticated. Run: gh auth login"
    echo "📋 Manual check: https://github.com/meta-introspector/zos-server/actions"
    exit 1
fi

echo "📊 Recent workflow runs:"
gh run list --limit 5 --repo meta-introspector/zos-server

echo ""
echo "🔍 Checking latest runs for failures..."

# Get the latest run ID
LATEST_RUN=$(gh run list --limit 1 --json databaseId --jq '.[0].databaseId' --repo meta-introspector/zos-server)

if [ -n "$LATEST_RUN" ]; then
    echo "📋 Latest run ID: $LATEST_RUN"

    # Get run status
    STATUS=$(gh run view $LATEST_RUN --json status,conclusion --jq '.status + " - " + (.conclusion // "in_progress")' --repo meta-introspector/zos-server)
    echo "📊 Status: $STATUS"

    # If failed, show logs
    if [[ "$STATUS" == *"failure"* ]] || [[ "$STATUS" == *"cancelled"* ]]; then
        echo ""
        echo "❌ Run failed! Getting logs..."
        gh run view $LATEST_RUN --log --repo meta-introspector/zos-server

        echo ""
        echo "🔧 Common fixes:"
        echo "1. Check Cargo.toml dependencies"
        echo "2. Fix compilation errors"
        echo "3. Update GitHub Actions permissions"
        echo "4. Check if GitHub Pages is enabled"
    elif [[ "$STATUS" == *"success"* ]]; then
        echo "✅ Latest run successful!"

        # Check if docs are deployed
        echo ""
        echo "📚 Documentation site should be available at:"
        echo "https://meta-introspector.github.io/zos-server/"
    else
        echo "⏳ Run in progress..."
        echo "🔄 Monitoring... (Ctrl+C to stop)"

        # Monitor until completion
        while true; do
            sleep 10
            NEW_STATUS=$(gh run view $LATEST_RUN --json status,conclusion --jq '.status + " - " + (.conclusion // "in_progress")' --repo meta-introspector/zos-server 2>/dev/null || echo "error")

            if [[ "$NEW_STATUS" != "$STATUS" ]]; then
                echo "📊 Status update: $NEW_STATUS"
                STATUS="$NEW_STATUS"

                if [[ "$STATUS" == *"completed"* ]]; then
                    if [[ "$STATUS" == *"success"* ]]; then
                        echo "✅ Run completed successfully!"
                        echo "📚 Documentation: https://meta-introspector.github.io/zos-server/"
                    else
                        echo "❌ Run failed!"
                        gh run view $LATEST_RUN --log --repo meta-introspector/zos-server
                    fi
                    break
                fi
            fi
        done
    fi
else
    echo "❌ No workflow runs found"
fi

echo ""
echo "🔗 View all runs: https://github.com/meta-introspector/zos-server/actions"
echo "📚 Documentation: https://meta-introspector.github.io/zos-server/"

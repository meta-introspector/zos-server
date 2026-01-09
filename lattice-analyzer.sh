#!/bin/bash
# ZOS Server Lattice Log Analyzer
# Pull and analyze lattice buckets from GitHub Actions

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LATTICE_DIR="$SCRIPT_DIR/lattice-analysis"

show_help() {
    cat << EOF
ZOS Server Lattice Log Analyzer

Usage: $0 [COMMAND] [OPTIONS]

Commands:
  list                    List available lattice buckets
  pull <lattice-id>       Pull specific lattice bucket
  pull-all               Pull all lattice buckets
  analyze <lattice-id>   Analyze specific lattice
  analyze-all            Analyze all lattices
  index                  Show lattice index
  search <pattern>       Search lattices by pattern

Options:
  --run-id <id>          GitHub Actions run ID
  --repo <owner/repo>    Repository (default: meta-introspector/zos-server)
  --token <token>        GitHub token for private repos

Examples:
  $0 list
  $0 pull L42.3.1.7
  $0 analyze L42.3.1.7
  $0 search "all-plugins.*release"

Lattice ID Format: L<level>.<weight>.<character>.<orbit>
- Level: 1-100 (feature hash)
- Weight: 1-10 (target hash)
- Character: 0-4 (profile hash)
- Orbit: 1-20 (trace hash)
EOF
}

# Default values
REPO="meta-introspector/zos-server"
RUN_ID=""
TOKEN=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --run-id)
            RUN_ID="$2"
            shift 2
            ;;
        --repo)
            REPO="$2"
            shift 2
            ;;
        --token)
            TOKEN="$2"
            shift 2
            ;;
        --help|-h)
            show_help
            exit 0
            ;;
        *)
            COMMAND="$1"
            shift
            break
            ;;
    esac
done

# Setup GitHub CLI auth
setup_gh_auth() {
    if [ -n "$TOKEN" ]; then
        echo "$TOKEN" | gh auth login --with-token
    elif ! gh auth status >/dev/null 2>&1; then
        echo "❌ GitHub CLI not authenticated. Please run 'gh auth login' or provide --token"
        exit 1
    fi
}

# Get latest run ID if not provided
get_latest_run_id() {
    if [ -z "$RUN_ID" ]; then
        echo "🔍 Finding latest workflow run..."
        RUN_ID=$(gh run list --repo "$REPO" --workflow="build.yml" --limit=1 --json databaseId --jq '.[0].databaseId')
        echo "📋 Using run ID: $RUN_ID"
    fi
}

# List available lattice buckets
list_lattices() {
    setup_gh_auth
    get_latest_run_id

    echo "📦 Available lattice buckets for run $RUN_ID:"
    gh run view "$RUN_ID" --repo "$REPO" --json artifacts --jq '.artifacts[] | select(.name | startswith("lattice-")) | .name' | \
        sed 's/lattice-//' | sort
}

# Pull specific lattice bucket
pull_lattice() {
    local lattice_id="$1"
    if [ -z "$lattice_id" ]; then
        echo "❌ Lattice ID required"
        exit 1
    fi

    setup_gh_auth
    get_latest_run_id

    mkdir -p "$LATTICE_DIR"
    cd "$LATTICE_DIR"

    echo "📥 Pulling lattice $lattice_id from run $RUN_ID..."
    gh run download "$RUN_ID" --repo "$REPO" --name "lattice-$lattice_id" --dir "lattice-$lattice_id"

    echo "✅ Lattice $lattice_id downloaded to: $LATTICE_DIR/lattice-$lattice_id"
}

# Pull all lattice buckets
pull_all_lattices() {
    setup_gh_auth
    get_latest_run_id

    mkdir -p "$LATTICE_DIR"
    cd "$LATTICE_DIR"

    echo "📥 Pulling all lattice buckets from run $RUN_ID..."

    # Get list of lattice artifacts
    LATTICES=$(gh run view "$RUN_ID" --repo "$REPO" --json artifacts --jq '.artifacts[] | select(.name | startswith("lattice-")) | .name')

    for artifact in $LATTICES; do
        lattice_id=$(echo "$artifact" | sed 's/lattice-//')
        echo "📦 Downloading $lattice_id..."
        gh run download "$RUN_ID" --repo "$REPO" --name "$artifact" --dir "$lattice_id" || true
    done

    # Also pull the index
    echo "📋 Downloading lattice index..."
    gh run download "$RUN_ID" --repo "$REPO" --name "lattice-index" --dir "index" || true

    echo "✅ All lattices downloaded to: $LATTICE_DIR"
}

# Analyze specific lattice
analyze_lattice() {
    local lattice_id="$1"
    if [ -z "$lattice_id" ]; then
        echo "❌ Lattice ID required"
        exit 1
    fi

    local lattice_path="$LATTICE_DIR/$lattice_id"
    if [ ! -d "$lattice_path" ]; then
        echo "❌ Lattice $lattice_id not found. Run 'pull $lattice_id' first."
        exit 1
    fi

    echo "🔬 Analyzing lattice $lattice_id"
    echo "================================"

    # Show manifest
    if [ -f "$lattice_path/lattice-manifest.json" ]; then
        echo "📋 Manifest:"
        jq . "$lattice_path/lattice-manifest.json"
        echo ""
    fi

    # Show build status
    if [ -f "$lattice_path/build-status.txt" ]; then
        echo "🏗️ Build Status:"
        cat "$lattice_path/build-status.txt"
        echo ""
    fi

    # Show file sizes
    echo "📁 Files:"
    find "$lattice_path" -type f -exec ls -lh {} \; | awk '{print $5 "\t" $9}' | sort -hr
    echo ""

    # Show trace summary
    if [ -f "$lattice_path/build-detailed.log" ]; then
        echo "📊 Build Log Summary:"
        echo "Lines: $(wc -l < "$lattice_path/build-detailed.log")"
        echo "Errors: $(grep -c "error:" "$lattice_path/build-detailed.log" || echo 0)"
        echo "Warnings: $(grep -c "warning:" "$lattice_path/build-detailed.log" || echo 0)"
        echo ""
    fi

    # Show coverage if available
    if [ -f "$lattice_path/coverage.lcov" ]; then
        echo "📈 Coverage:"
        echo "Lines: $(wc -l < "$lattice_path/coverage.lcov")"
        echo ""
    fi
}

# Show lattice index
show_index() {
    local index_path="$LATTICE_DIR/index"
    if [ ! -d "$index_path" ]; then
        echo "❌ Index not found. Run 'pull-all' first."
        exit 1
    fi

    echo "📊 Lattice Index"
    echo "================"

    if [ -f "$index_path/statistics.json" ]; then
        echo "📈 Statistics:"
        jq . "$index_path/statistics.json"
        echo ""
    fi

    if [ -f "$index_path/README.md" ]; then
        echo "📋 Lattices:"
        cat "$index_path/README.md"
    fi
}

# Search lattices by pattern
search_lattices() {
    local pattern="$1"
    if [ -z "$pattern" ]; then
        echo "❌ Search pattern required"
        exit 1
    fi

    local index_path="$LATTICE_DIR/index"
    if [ ! -f "$index_path/lattices.json" ]; then
        echo "❌ Index not found. Run 'pull-all' first."
        exit 1
    fi

    echo "🔍 Searching lattices for pattern: $pattern"
    echo "==========================================="

    jq -r --arg pattern "$pattern" '
        .[] |
        select(
            (.matrix.features | test($pattern)) or
            (.matrix.target | test($pattern)) or
            (.matrix.profile | test($pattern)) or
            (.matrix.trace | test($pattern))
        ) |
        "\(.lattice_id): \(.matrix.features)+\(.matrix.target)+\(.matrix.profile)+\(.matrix.trace)"
    ' "$index_path/lattices.json"
}

# Main command dispatch
case "$COMMAND" in
    list)
        list_lattices
        ;;
    pull)
        pull_lattice "$1"
        ;;
    pull-all)
        pull_all_lattices
        ;;
    analyze)
        analyze_lattice "$1"
        ;;
    analyze-all)
        if [ ! -d "$LATTICE_DIR" ]; then
            echo "❌ No lattices found. Run 'pull-all' first."
            exit 1
        fi
        for lattice_dir in "$LATTICE_DIR"/L*; do
            if [ -d "$lattice_dir" ]; then
                lattice_id=$(basename "$lattice_dir")
                echo "🔬 Analyzing $lattice_id..."
                analyze_lattice "$lattice_id"
                echo ""
            fi
        done
        ;;
    index)
        show_index
        ;;
    search)
        search_lattices "$1"
        ;;
    *)
        show_help
        exit 1
        ;;
esac

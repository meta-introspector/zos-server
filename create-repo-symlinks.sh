#!/bin/bash

# Create symlink structure for all repositories in meta-introspector
META_DIR="/mnt/data1/meta-introspector"
SYMLINK_DIR="$META_DIR/repos"

echo "🔗 Creating standardized symlink structure..."

# Create repos directory if it doesn't exist
mkdir -p "$SYMLINK_DIR"

# Read from repos.txt and create symlinks
if [ -f "$META_DIR/repos.txt" ]; then
    while IFS= read -r repo_path; do
        # Skip empty lines and comments
        [[ -z "$repo_path" || "$repo_path" =~ ^# ]] && continue

        # Expand tilde
        expanded_path=$(eval echo "$repo_path")

        if [ -d "$expanded_path" ]; then
            # Get repo name from path
            repo_name=$(basename "$expanded_path")
            symlink_path="$SYMLINK_DIR/$repo_name"

            # Create symlink if it doesn't exist
            if [ ! -L "$symlink_path" ]; then
                ln -s "$expanded_path" "$symlink_path"
                echo "✅ Linked: $repo_name -> $expanded_path"
            else
                echo "⚠️  Already exists: $repo_name"
            fi
        else
            echo "❌ Not found: $expanded_path"
        fi
    done < "$META_DIR/repos.txt"
fi

# Also link canonical directories that have real paths
if [ -d "$META_DIR/canonical" ]; then
    for canonical_dir in "$META_DIR/canonical"/*; do
        if [ -d "$canonical_dir" ]; then
            sources_link="$canonical_dir/sources/v1"
            if [ -L "$sources_link" ]; then
                real_path=$(readlink "$sources_link")
                if [ -d "$real_path" ]; then
                    canonical_name=$(basename "$canonical_dir")
                    symlink_path="$SYMLINK_DIR/$canonical_name"

                    if [ ! -L "$symlink_path" ]; then
                        ln -s "$real_path" "$symlink_path"
                        echo "✅ Canonical linked: $canonical_name -> $real_path"
                    fi
                fi
            fi
        fi
    done
fi

# Add high-priority directories
declare -a priority_dirs=(
    "/home/mdupont/zombie_driver2"
    "/mnt/data1/nix/time/2024/12/10/swarms-terraform"
    "/mnt/data1/meta-introspector/value-lattice"
)

for dir in "${priority_dirs[@]}"; do
    if [ -d "$dir" ]; then
        dir_name=$(basename "$dir")
        symlink_path="$SYMLINK_DIR/$dir_name"

        if [ ! -L "$symlink_path" ]; then
            ln -s "$dir" "$symlink_path"
            echo "✅ Priority linked: $dir_name -> $dir"
        fi
    fi
done

echo ""
echo "📊 Repository symlinks created in: $SYMLINK_DIR"
echo "📈 Total repositories: $(ls -1 "$SYMLINK_DIR" | wc -l)"
echo ""
echo "🔍 Use this directory for standardized access to all repos"

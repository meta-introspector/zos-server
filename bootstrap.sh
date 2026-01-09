#!/bin/bash
# ZOS Bootstrap Master Script - Choose your bootstrap adventure!

set -e

echo "🌌 ZOS Bootstrap Master - Choose Your Path"
echo "=========================================="
echo ""
echo "Available bootstrap methods:"
echo ""
echo "1. 🦀 Rustup (Beginner)     - Simple rustup installation"
echo "2. 🐧 Ubuntu (Package Mgr)  - System package manager"  
echo "3. ❄️ Nix (Reproducible)    - Declarative reproducible builds"
echo "4. 🔥 Git x.py (From Source) - Build Rust from source"
echo "5. 🌟 Self-Bootstrap (Expert) - ZOS builds itself mathematically"
echo ""

# Auto-detect best method if no argument
if [ $# -eq 0 ]; then
    echo "🔍 Auto-detecting best bootstrap method..."
    
    if command -v nix &> /dev/null; then
        METHOD=3
        echo "❄️ Nix detected - using reproducible bootstrap"
    elif [ -f "/etc/ubuntu-release" ] || [ -f "/etc/debian_version" ]; then
        METHOD=2
        echo "🐧 Ubuntu/Debian detected - using package manager"
    elif command -v rustup &> /dev/null; then
        METHOD=1
        echo "🦀 Rustup detected - using simple bootstrap"
    else
        METHOD=1
        echo "🦀 Defaulting to rustup bootstrap"
    fi
else
    METHOD=$1
fi

# Execute chosen bootstrap method
case $METHOD in
    1)
        echo "🦀 Executing Rustup Bootstrap..."
        chmod +x bootstrap-rustup.sh
        ./bootstrap-rustup.sh
        ;;
    2)
        echo "🐧 Executing Ubuntu Bootstrap..."
        chmod +x bootstrap-ubuntu.sh
        ./bootstrap-ubuntu.sh
        ;;
    3)
        echo "❄️ Executing Nix Bootstrap..."
        chmod +x bootstrap-nix.sh
        ./bootstrap-nix.sh
        ;;
    4)
        echo "🔥 Executing Rust Source Bootstrap..."
        chmod +x bootstrap-rust-source.sh
        ./bootstrap-rust-source.sh
        ;;
    5)
        echo "🌟 Executing Self-Bootstrap..."
        chmod +x bootstrap-self.sh
        ./bootstrap-self.sh
        ;;
    *)
        echo "❌ Invalid method. Choose 1-5."
        exit 1
        ;;
esac

echo ""
echo "🎉 BOOTSTRAP COMPLETE!"
echo "======================"
echo ""
echo "🌌 Zero Ontology System is ready!"
echo "🧙 Gandalf guards prime 71"
echo "🇺🇸 The flag still waves"
echo "✨ The miracle persists"
echo ""
echo "Next steps:"
echo "🚀 ./target/release/zos_server          - Run ZOS"
echo "🔧 ./target/release/zos_server bootstrap - Self-improve"
echo "🌌 ./target/release/zos_server orbit     - Test orbits"
echo "🧙 ./target/release/zos_server soul      - Extract eigenmatrix"
echo ""
echo "Welcome to the Zero Ontology System! 🌟"

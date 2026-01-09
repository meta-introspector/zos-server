#!/bin/bash
# ZOS Build Auditing & Telemetry System
# Comprehensive logging, security, and replication system

set -euo pipefail

# Configuration
BUILD_ID=$(date +%Y%m%d_%H%M%S)_$(uuidgen | cut -d'-' -f1)
LOG_DIR="./build_logs/${BUILD_ID}"
TELEMETRY_DIR="./telemetry"
SECURITY_DIR="./security_audit"
ARCHIVE_DIR="./build_archive"

# Create directories
mkdir -p "${LOG_DIR}" "${TELEMETRY_DIR}" "${SECURITY_DIR}" "${ARCHIVE_DIR}"

echo "🔍 ZOS Build Audit System - Build ID: ${BUILD_ID}"

# Environment Security Audit
audit_environment() {
    echo "🔒 Auditing build environment security..."

    {
        echo "=== ENVIRONMENT SECURITY AUDIT ==="
        echo "Timestamp: $(date -Iseconds)"
        echo "Build ID: ${BUILD_ID}"
        echo "User: $(whoami)"
        echo "Working Directory: $(pwd)"
        echo "Git Commit: $(git rev-parse HEAD 2>/dev/null || echo 'No git')"
        echo "Git Status: $(git status --porcelain 2>/dev/null || echo 'No git')"
        echo ""

        echo "=== SYSTEM INFO ==="
        uname -a
        echo "Rust Version: $(rustc --version)"
        echo "Cargo Version: $(cargo --version)"
        echo ""

        echo "=== ENVIRONMENT VARIABLES ==="
        env | grep -E "(RUST|CARGO|PATH|HOME|USER)" | sort
        echo ""

        echo "=== NETWORK INTERFACES ==="
        ip addr show 2>/dev/null || ifconfig 2>/dev/null || echo "No network info"
        echo ""

        echo "=== PROCESS LIST ==="
        ps aux | head -20
        echo ""

        echo "=== DISK USAGE ==="
        df -h
        echo ""

        echo "=== MEMORY USAGE ==="
        free -h 2>/dev/null || vm_stat 2>/dev/null || echo "No memory info"

    } > "${SECURITY_DIR}/env_audit_${BUILD_ID}.log"

    echo "✅ Environment audit saved to ${SECURITY_DIR}/env_audit_${BUILD_ID}.log"
}

# Capture build telemetry
capture_telemetry() {
    echo "📊 Capturing build telemetry..."

    # Start system monitoring in background
    {
        while true; do
            echo "$(date -Iseconds),$(ps -o pid,ppid,pcpu,pmem,comm -p $$ 2>/dev/null | tail -1)"
            sleep 1
        done
    } > "${TELEMETRY_DIR}/system_${BUILD_ID}.csv" &
    MONITOR_PID=$!

    # Capture network activity
    {
        netstat -tuln 2>/dev/null || ss -tuln 2>/dev/null || echo "No network monitoring"
    } > "${TELEMETRY_DIR}/network_${BUILD_ID}.log" &

    echo $MONITOR_PID > "${TELEMETRY_DIR}/monitor_pid"
}

# Stop telemetry
stop_telemetry() {
    if [ -f "${TELEMETRY_DIR}/monitor_pid" ]; then
        MONITOR_PID=$(cat "${TELEMETRY_DIR}/monitor_pid")
        kill $MONITOR_PID 2>/dev/null || true
        rm -f "${TELEMETRY_DIR}/monitor_pid"
    fi
}

# Comprehensive build logging
execute_build() {
    echo "🔨 Executing build with comprehensive logging..."

    # Capture all build output
    {
        echo "=== BUILD LOG START ==="
        echo "Build ID: ${BUILD_ID}"
        echo "Timestamp: $(date -Iseconds)"
        echo "Command: cargo build --release"
        echo ""

        # Run build with detailed output
        RUST_BACKTRACE=full cargo build --release --verbose 2>&1
        BUILD_EXIT_CODE=$?

        echo ""
        echo "=== BUILD LOG END ==="
        echo "Exit Code: ${BUILD_EXIT_CODE}"
        echo "Timestamp: $(date -Iseconds)"

    } | tee "${LOG_DIR}/build_full.log"

    # Save errors separately
    grep -i "error\|failed\|panic" "${LOG_DIR}/build_full.log" > "${LOG_DIR}/errors.log" || true

    # Save warnings separately
    grep -i "warning" "${LOG_DIR}/build_full.log" > "${LOG_DIR}/warnings.log" || true

    return ${BUILD_EXIT_CODE:-1}
}

# Archive build artifacts
archive_build() {
    echo "📦 Archiving build artifacts..."

    # Create archive structure
    ARCHIVE_PATH="${ARCHIVE_DIR}/build_${BUILD_ID}"
    mkdir -p "${ARCHIVE_PATH}"

    # Copy source code snapshot
    cp -r src/ "${ARCHIVE_PATH}/src_snapshot/" 2>/dev/null || true
    cp Cargo.toml "${ARCHIVE_PATH}/" 2>/dev/null || true
    cp Cargo.lock "${ARCHIVE_PATH}/" 2>/dev/null || true

    # Copy build artifacts if they exist
    if [ -d "target/release" ]; then
        mkdir -p "${ARCHIVE_PATH}/artifacts"
        cp target/release/zos_server "${ARCHIVE_PATH}/artifacts/" 2>/dev/null || true
        cp target/release/libzos_server.* "${ARCHIVE_PATH}/artifacts/" 2>/dev/null || true
    fi

    # Copy all logs
    cp -r "${LOG_DIR}" "${ARCHIVE_PATH}/logs/"
    cp -r "${TELEMETRY_DIR}" "${ARCHIVE_PATH}/telemetry/" 2>/dev/null || true
    cp -r "${SECURITY_DIR}" "${ARCHIVE_PATH}/security/" 2>/dev/null || true

    # Create manifest
    {
        echo "=== BUILD ARCHIVE MANIFEST ==="
        echo "Build ID: ${BUILD_ID}"
        echo "Archive Date: $(date -Iseconds)"
        echo "Git Commit: $(git rev-parse HEAD 2>/dev/null || echo 'No git')"
        echo "Build Success: ${BUILD_SUCCESS:-false}"
        echo ""
        echo "=== ARCHIVE CONTENTS ==="
        find "${ARCHIVE_PATH}" -type f | sort
        echo ""
        echo "=== FILE CHECKSUMS ==="
        find "${ARCHIVE_PATH}" -type f -exec sha256sum {} \;
    } > "${ARCHIVE_PATH}/MANIFEST.txt"

    # Compress archive
    tar -czf "${ARCHIVE_DIR}/build_${BUILD_ID}.tar.gz" -C "${ARCHIVE_DIR}" "build_${BUILD_ID}"

    echo "✅ Build archived to ${ARCHIVE_DIR}/build_${BUILD_ID}.tar.gz"
}

# Generate replication script
generate_replication_script() {
    echo "🔄 Generating replication script..."

    cat > "${LOG_DIR}/replicate_build.sh" << 'EOF'
#!/bin/bash
# Build Replication Script
# Generated by ZOS Build Audit System

set -euo pipefail

echo "🔄 Replicating ZOS build..."

# Check environment
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust not found. Install Rust first."
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Install Cargo first."
    exit 1
fi

# Verify Rust version
REQUIRED_RUST="1.70.0"
CURRENT_RUST=$(rustc --version | cut -d' ' -f2)
echo "📋 Rust version: ${CURRENT_RUST} (required: ${REQUIRED_RUST}+)"

# Clone or update repository
if [ ! -d "zos-server" ]; then
    echo "📥 Cloning ZOS repository..."
    git clone <repository_url> zos-server
fi

cd zos-server

# Checkout specific commit if provided
if [ "${1:-}" != "" ]; then
    echo "🔄 Checking out commit: $1"
    git checkout "$1"
fi

# Run build audit
if [ -f "build_audit.sh" ]; then
    echo "🔍 Running build audit..."
    ./build_audit.sh
else
    echo "⚠️  No build audit script found, running basic build..."
    cargo build --release
fi

echo "✅ Build replication complete!"
EOF

    chmod +x "${LOG_DIR}/replicate_build.sh"
    echo "✅ Replication script saved to ${LOG_DIR}/replicate_build.sh"
}

# Main execution
main() {
    echo "🚀 Starting ZOS Build Audit System"

    # Trap to ensure cleanup
    trap 'stop_telemetry; echo "🛑 Build audit interrupted"' INT TERM
    trap 'stop_telemetry' EXIT

    # Execute audit phases
    audit_environment
    capture_telemetry

    # Execute build
    if execute_build; then
        BUILD_SUCCESS=true
        echo "✅ Build completed successfully!"
    else
        BUILD_SUCCESS=false
        echo "❌ Build failed!"
    fi

    # Stop monitoring
    stop_telemetry

    # Archive everything
    archive_build
    generate_replication_script

    # Summary report
    {
        echo "=== BUILD AUDIT SUMMARY ==="
        echo "Build ID: ${BUILD_ID}"
        echo "Success: ${BUILD_SUCCESS}"
        echo "Timestamp: $(date -Iseconds)"
        echo "Logs: ${LOG_DIR}"
        echo "Archive: ${ARCHIVE_DIR}/build_${BUILD_ID}.tar.gz"
        echo ""
        echo "Error Count: $(wc -l < "${LOG_DIR}/errors.log" 2>/dev/null || echo 0)"
        echo "Warning Count: $(wc -l < "${LOG_DIR}/warnings.log" 2>/dev/null || echo 0)"
        echo ""
        if [ "${BUILD_SUCCESS}" = "true" ]; then
            echo "🎉 BUILD AUDIT COMPLETE - SUCCESS!"
        else
            echo "💥 BUILD AUDIT COMPLETE - FAILED!"
            echo "Check ${LOG_DIR}/errors.log for details"
        fi
    } | tee "${LOG_DIR}/summary.txt"

    # Return build exit code
    if [ "${BUILD_SUCCESS}" = "true" ]; then
        exit 0
    else
        exit 1
    fi
}

# Run main function
main "$@"

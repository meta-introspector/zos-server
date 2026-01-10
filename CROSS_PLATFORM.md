# ZOS Cross-Platform Deployment

ZOS Server with cross-platform compilation support for Windows, ARM64 Linux (Oracle Cloud), and Android.

## Quick Start

### Prerequisites

- Nix package manager
- Accept Android SDK license: `export NIXPKGS_ACCEPT_ANDROID_SDK_LICENSE=1`

### Local Deployment

```bash
# Deploy ZOS1 locally with systemd
./deploy-zos1-nightly.sh

# Check status
sudo systemctl status zos-server.service
curl http://localhost:8080/health
```

### Cross-Platform Builds

```bash
# Setup cross-compilation environment
export NIXPKGS_ACCEPT_ANDROID_SDK_LICENSE=1
./setup-cross-compilation.sh

# Build for all platforms
./build-cross-platform.sh

# Build for specific target
./build-cross-platform.sh x86_64-pc-windows-gnu
```

## Supported Targets

| Platform | Target | Use Case |
|----------|--------|----------|
| Linux x64 | `x86_64-unknown-linux-gnu` | Host system |
| Windows x64 | `x86_64-pc-windows-gnu` | Windows deployment |
| ARM64 Linux | `aarch64-unknown-linux-gnu` | Oracle Cloud ARM instances |
| Android ARM64 | `aarch64-linux-android` | Android devices |
| Android ARM32 | `armv7-linux-androideabi` | Older Android devices |

## ZOS1 → ZOS2 Deployment Chain

```bash
# ZOS1 deploys ZOS2 via HTTP API
curl -X POST http://localhost:8080/deploy \
  -H "Content-Type: application/json" \
  -d '{
    "target_port": 8081,
    "instance_name": "zos2",
    "rebuild_self": true,
    "prepare_windows": true
  }'

# ZOS2 rebuilds itself
curl -X POST http://localhost:8081/rebuild \
  -H "Content-Type: application/json" \
  -d '{"prepare_windows": true}'
```

## Environment Variables

```bash
# Required for Android builds
export NIXPKGS_ACCEPT_ANDROID_SDK_LICENSE=1

# Optional: Custom ports
export ZOS_HTTP_PORT=8080
export ZOS_DATA_DIR=/var/lib/zos/data
```

## Architecture

- **ZOS1**: Initial server deployed locally with systemd
- **ZOS2**: Deployed by ZOS1, can rebuild itself and prepare Windows binaries
- **Cross-compilation**: Nix shell with nightly Rust and cross-compilation toolchains
- **Targets**: Windows, ARM64 Linux (OCI), Android

## Files

- `shell.nix` - Nix environment with cross-compilation support
- `deploy-zos1-nightly.sh` - Deploy ZOS1 locally with systemd
- `setup-cross-compilation.sh` - Test cross-compilation environment
- `build-cross-platform.sh` - Build for all supported platforms
- `zos-minimal-server/` - Minimal ZOS server with deployment endpoints

## Oracle Cloud Integration

ARM64 Linux builds are specifically for Oracle Cloud ARM instances:

```bash
# Build ARM64 binary for OCI
./build-cross-platform.sh aarch64-unknown-linux-gnu

# Deploy to OCI (via existing terraform)
./deploy-zos-oracle.sh
```

## Android Support

Android builds create binaries that can run on Android devices:

```bash
# Build for Android
./build-cross-platform.sh aarch64-linux-android
./build-cross-platform.sh armv7-linux-androideabi
```

## Windows Support

Windows builds use MinGW with static linking:

```bash
# Build Windows binary
./build-cross-platform.sh x86_64-pc-windows-gnu

# Output: target/cross-builds/x86_64-pc-windows-gnu/zos-minimal-server.exe
```

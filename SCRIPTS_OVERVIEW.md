# ZOS Server Scripts Overview

## CI/CD Pipeline Scripts

### Core Pipeline Scripts
- **`deploy-pipeline.sh`** - Full automated CI/CD pipeline (dev → staging → prod → clients)
- **`test-cicd-pipeline.sh`** - Test the CI/CD pipeline with proper git-based updates
- **`test-pipeline.sh`** - Legacy pipeline test (direct server testing)
- **`update-prod.sh`** - Production update script (called by QA server)

### Service Management Scripts
- **`boot-zos-instances.sh`** - Boot multiple ZOS instances
- **`setup-zos-instance.sh`** - Setup individual ZOS instance
- **`create-zos-instance.sh`** - Create new ZOS instance
- **`deploy-dual-servers.sh`** - Deploy dev and prod servers simultaneously

### Git Integration Scripts
- **`test-git-cicd.sh`** - Test git-based CI/CD functionality
- **`test-git-repos.sh`** - Test git repository operations
- **`deploy-git-network.sh`** - Deploy git-based network services

### Build and Compilation Scripts
- **`build-cross-platform.sh`** - Cross-platform compilation
- **`build-android.sh`** - Android-specific build
- **`cross-compile.sh`** - Cross-compilation utilities
- **`setup-cross-compilation.sh`** - Setup cross-compilation environment

### Deployment Scripts
- **`deploy-local-systemd.sh`** - Local systemd service deployment
- **`deploy-nix-zos.sh`** - Nix-based ZOS deployment
- **`deploy-zos1-nightly.sh`** - Nightly ZOS1 deployment
- **`deploy-zos2-via-zos1.sh`** - Deploy ZOS2 through ZOS1
- **`deploy-native.sh`** - Native deployment script

### Testing Scripts
- **`test-endpoints.sh`** - Test HTTP endpoints
- **`test-hot-swap.sh`** - Test hot-swapping functionality
- **`test-log-management.sh`** - Test logging systems
- **`test-self-build.sh`** - Test self-building capability

### Bootstrap Scripts
- **`bootstrap-zos.sh`** - Bootstrap ZOS system
- **`bootstrap-self.sh`** - Self-bootstrap functionality
- **`bootstrap-nix.sh`** - Nix-based bootstrap
- **`bootstrap-ubuntu.sh`** - Ubuntu-specific bootstrap
- **`bootstrap-rustup.sh`** - Rust toolchain bootstrap

### Utility Scripts
- **`quick-install.sh`** - Quick installation script
- **`install-from-node.sh`** - Install from existing node
- **`launch-dev-server.sh`** - Launch development server
- **`debug-build.sh`** - Debug build process

## Current Pipeline Status

### Active Services
- **Development Server**: Port 8080 (local build)
- **QA Service**: Port 8082 (git: qa branch)
- **Production Service**: Port 8084 (git: stable branch)

### Git Branches
- **main**: Development branch (2 commits ahead of origin)
- **qa**: QA testing branch
- **stable**: Production/client branch

### Modified Files
- `update-prod.sh`: Updated to use `/opt/zos-test-production`
- `zos-minimal-server/src/main.rs`: Updated QA paths to `/opt/zos-test-qa`

### Service Status
- All ZOS processes stopped
- QA systemd service stopped
- Ready for pipeline testing

## Recommended Testing Sequence

1. **Start with CI/CD Pipeline Test**:
   ```bash
   ./test-cicd-pipeline.sh
   ```

2. **Test Full Deployment Pipeline**:
   ```bash
   ./deploy-pipeline.sh
   ```

3. **Test Individual Components**:
   ```bash
   ./test-endpoints.sh
   ./test-git-cicd.sh
   ```

4. **Production Deployment**:
   ```bash
   # After QA validation
   curl -X POST http://localhost:8080/deploy/staging-to-prod
   ```

## Documentation Files

- **`PIPELINE.md`** - Complete CI/CD pipeline documentation
- **`CHANGES.md`** - Current changes and system state
- **`README.md`** - Main project documentation (updated with pipeline info)
- **`DEPLOYMENT_PLAN.md`** - Deployment planning documentation
- **`CROSS_PLATFORM.md`** - Cross-platform build documentation

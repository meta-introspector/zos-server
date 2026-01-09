# GitHub Actions Troubleshooting Guide

## Current Issues
- All workflows showing "startup_failure" status
- Both complex and simple workflows failing immediately
- Documentation workflow also failing with startup_failure

## Possible Causes & Solutions

### 1. Repository Settings
**Check GitHub Repository Settings:**
1. Go to: https://github.com/meta-introspector/zos-server/settings
2. Navigate to "Actions" → "General"
3. Ensure "Allow all actions and reusable workflows" is selected
4. Check that "Allow GitHub Actions to create and approve pull requests" is enabled

### 2. GitHub Pages Configuration
**Enable GitHub Pages:**
1. Go to: https://github.com/meta-introspector/zos-server/settings/pages
2. Set Source to "GitHub Actions"
3. This is required for the documentation workflow

### 3. Permissions Issues
**Check Repository Permissions:**
- Ensure the repository has proper permissions for Actions
- Check if there are any organization-level restrictions

### 4. Workflow File Issues
**Current Status:**
- ✅ Simplified workflows created
- ✅ Removed nightly-only features
- ✅ Fixed action references
- ❌ Still getting startup_failure

### 5. Manual Verification Steps

**Test locally:**
```bash
# These should all work (and do work locally):
cargo check --all-features
cargo build --all-features
cargo test --all-features
./target/debug/zos_server --help
```

**Check GitHub Actions status:**
```bash
# Use the monitoring script:
./monitor-actions.sh

# Or check manually:
gh run list --repo meta-introspector/zos-server
```

### 6. Next Steps
1. **Manual GitHub Settings Check**: Verify Actions are enabled in repository settings
2. **Enable GitHub Pages**: Set source to "GitHub Actions" in Pages settings
3. **Check Organization Policies**: Ensure no org-level restrictions on Actions
4. **Contact GitHub Support**: If issues persist, may be a GitHub-side problem

### 7. Workarounds
- Documentation can be generated locally: `cargo doc --all-features --open`
- Build verification works locally with all features
- ZOS Server functionality is fully operational

## Files Created
- `monitor-actions.sh`: GitHub Actions monitoring script
- `.github/workflows/simple-build.yml`: Simplified build workflow
- `.github/workflows/minimal-test.yml`: Basic connectivity test
- Fixed `.github/workflows/build.yml`: Removed problematic features

## Manual Commands
```bash
# Monitor GitHub Actions
./monitor-actions.sh

# Generate docs locally
cargo doc --all-features --no-deps --open

# Test all functionality
cargo test --all-features

# Run ZOS Server
cargo run --features all-plugins
```

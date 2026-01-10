# ZOS Native Platform Deployment

Deploy ZOS natively on each target platform for optimal performance and compatibility.

## Linux x86_64 (Current Host)

```bash
# Already working - build and deploy locally
cd zos-minimal-server
cargo build --release
sudo ./deploy-local-systemd.sh
```

## Android ARM64 (Nix4Droid)

```bash
# On Android device with Nix4Droid
nix-shell -p rustc cargo git pkg-config openssl
git clone <zos-repo>
cd zos-server/zos-minimal-server
cargo build --release
./zos-minimal-server
```

## Windows x64 (MinGW Rust)

```powershell
# On Windows with MinGW Rust environment
git clone <zos-repo>
cd zos-server\zos-minimal-server
cargo build --release
.\target\release\zos-minimal-server.exe
```

## Oracle Cloud ARM64 Linux

```bash
# On Oracle Cloud ARM64 instance
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
git clone <zos-repo>
cd zos-server/zos-minimal-server
cargo build --release
sudo systemctl enable --now zos-server
```

## Deployment Strategy

1. **ZOS1 (Bootstrap)**: Deploy on current Linux x86_64 host
2. **Remote Deployment**: Use ZOS1's `/deploy` endpoint to bootstrap other nodes
3. **Native Builds**: Each platform builds from source natively
4. **Network Formation**: All nodes connect via LibP2P mesh

## Remote Bootstrap Commands

```bash
# Deploy to Oracle Cloud ARM64
curl -X POST http://localhost:8080/deploy \
  -H "Content-Type: application/json" \
  -d '{
    "target_port": 8081,
    "instance_name": "zos-oracle-arm64",
    "rebuild_self": true,
    "prepare_windows": false
  }'

# Deploy to Android (via SSH tunnel)
curl -X POST http://localhost:8080/deploy \
  -H "Content-Type: application/json" \
  -d '{
    "target_port": 8082,
    "instance_name": "zos-android-arm64",
    "rebuild_self": false,
    "prepare_windows": false
  }'

# Deploy to Windows (via network)
curl -X POST http://localhost:8080/deploy \
  -H "Content-Type: application/json" \
  -d '{
    "target_port": 8083,
    "instance_name": "zos-windows-x64",
    "rebuild_self": false,
    "prepare_windows": true
  }'
```

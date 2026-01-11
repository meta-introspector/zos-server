# Deploy to Windows Laptop
Write-Host "💻 Deploying to Windows Laptop..."

# Install Rust
Invoke-WebRequest -Uri "https://win.rustup.rs/" -OutFile "rustup-init.exe"
.\rustup-init.exe -y

# Install WireGuard
Invoke-WebRequest -Uri "https://download.wireguard.com/windows-client/wireguard-installer.exe" -OutFile "wireguard-installer.exe"
.\wireguard-installer.exe /S

# Build client
cargo build --release --bin gpu-dashboard --features client

Write-Host "✅ Windows client deployed"

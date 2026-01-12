#!/bin/bash

# Create zos user if it doesn't exist
if ! id "zos" &>/dev/null; then
    echo "Creating zos user..."
    sudo useradd -r -s /bin/bash -d /opt/zos -m zos
    sudo usermod -aG sudo zos
fi

# Set up zos directories
sudo mkdir -p /opt/zos/{bin,logs,data}
sudo chown -R zos:zos /opt/zos

# Copy ZOS binaries to zos user space
sudo cp ~/zos-server/target/release/zos_server /opt/zos/bin/
sudo cp ~/zos-server/zos-cli /opt/zos/bin/
sudo cp ~/zos-server/zos-service /opt/zos/bin/
sudo chown zos:zos /opt/zos/bin/*
sudo chmod +x /opt/zos/bin/*

# Create systemd service for ZOS
sudo tee /etc/systemd/system/zos-server.service > /dev/null <<EOF
[Unit]
Description=ZOS Server - Zero Ontology System
After=network.target

[Service]
Type=simple
User=zos
Group=zos
WorkingDirectory=/opt/zos
ExecStart=/opt/zos/bin/zos_server serve
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF

# Create value lattice service
sudo tee /etc/systemd/system/value-lattice.service > /dev/null <<EOF
[Unit]
Description=Value Lattice Server
After=network.target

[Service]
Type=simple
User=zos
Group=zos
WorkingDirectory=/opt/zos
ExecStart=/home/mdupont/zombie_driver2/target/release/value_lattice_server
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd and enable services
sudo systemctl daemon-reload
sudo systemctl enable zos-server
sudo systemctl enable value-lattice

echo "✅ ZOS services configured"
echo "Start with: sudo systemctl start zos-server"
echo "Start with: sudo systemctl start value-lattice"
echo "View logs: journalctl -u zos-server -f"

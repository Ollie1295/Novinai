#!/bin/bash
set -euo pipefail

# Create Deep-K systemd service

echo "🔧 Creating Deep-K systemd service..."

# Determine which binary to use
DEEPK_DIR="/opt/deepk"
DEEPK_BINARY=""

# Check for available binaries and let user choose
if [[ -d "$DEEPK_DIR/bin" ]]; then
    echo "📋 Available Deep-K binaries:"
    BINARIES=($(ls "$DEEPK_DIR/bin/" | grep -E '^(pipeline|thinking_ai_demo|daemon|security-daemon)$' || true))
    
    if [[ ${#BINARIES[@]} -eq 0 ]]; then
        echo "❌ No suitable Deep-K binaries found in $DEEPK_DIR/bin/"
        echo "Expected: pipeline, thinking_ai_demo, daemon, or security-daemon"
        exit 1
    fi
    
    if [[ ${#BINARIES[@]} -eq 1 ]]; then
        DEEPK_BINARY="${BINARIES[0]}"
        echo "🎯 Using binary: $DEEPK_BINARY"
    else
        echo "Multiple binaries found:"
        for i in "${!BINARIES[@]}"; do
            echo "  $((i+1)). ${BINARIES[$i]}"
        done
        
        read -p "Choose binary (1-${#BINARIES[@]}): " choice
        if [[ $choice -ge 1 && $choice -le ${#BINARIES[@]} ]]; then
            DEEPK_BINARY="${BINARIES[$((choice-1))]}"
            echo "🎯 Selected: $DEEPK_BINARY"
        else
            echo "❌ Invalid choice"
            exit 1
        fi
    fi
else
    echo "❌ Deep-K binaries directory not found: $DEEPK_DIR/bin/"
    exit 1
fi

# Create the systemd service file
SERVICE_FILE="/etc/systemd/system/deepk.service"

sudo tee "$SERVICE_FILE" > /dev/null << EOF
[Unit]
Description=Deep-K AI Security System
Documentation=https://github.com/Ollie1295/Novinai
After=network.target docker.service
Wants=docker.service
StartLimitIntervalSec=120
StartLimitBurst=3

[Service]
Type=simple
User=deepk
Group=deepk
WorkingDirectory=$DEEPK_DIR
ExecStartPre=/bin/bash -c 'timeout 30 bash -c "until docker exec nats nats server check > /dev/null 2>&1; do sleep 2; done"'

# Environment variables for NATS integration
Environment=DEEPK_NATS_URL=nats://127.0.0.1:4222
Environment=DEEPK_IN_SUBJECT=events.perception.*
Environment=DEEPK_OUT_SUBJECT=deepk.alerts.main
Environment=RUST_LOG=info
Environment=TOKIO_WORKER_THREADS=4

# Performance settings
Environment=RUST_BACKTRACE=1
Environment=RUST_LIB_BACKTRACE=0

# Security settings
NoNewPrivileges=yes
PrivateTmp=yes
ProtectHome=yes
ProtectSystem=strict
ReadWritePaths=$DEEPK_DIR /tmp

# Resource limits
LimitNOFILE=65535
LimitNPROC=32768

# Start the Deep-K binary
ExecStart=$DEEPK_DIR/bin/$DEEPK_BINARY

# Restart policy
Restart=always
RestartSec=5
TimeoutStartSec=60
TimeoutStopSec=30

# Health check (optional)
# ExecStartPost=/bin/bash -c 'sleep 10; timeout 5 docker run --rm --network host natsio/nats-box:latest nats sub "deepk.alerts.*" --count=1 > /dev/null'

[Install]
WantedBy=multi-user.target
EOF

echo "📝 Systemd service created: $SERVICE_FILE"

# Reload systemd and enable the service
echo "🔄 Reloading systemd daemon..."
sudo systemctl daemon-reload

echo "✅ Enabling Deep-K service..."
sudo systemctl enable deepk

echo ""
echo "🎉 Deep-K systemd service created successfully!"
echo ""
echo "Commands:"
echo "  Start:   sudo systemctl start deepk"
echo "  Stop:    sudo systemctl stop deepk"
echo "  Status:  sudo systemctl status deepk"
echo "  Logs:    journalctl -u deepk -f"
echo "  Restart: sudo systemctl restart deepk"
echo ""
echo "Service details:"
echo "  Binary:  $DEEPK_DIR/bin/$DEEPK_BINARY"
echo "  User:    deepk"
echo "  Config:  $SERVICE_FILE"
echo ""
echo "To start Deep-K now:"
echo "  sudo systemctl start deepk"

#!/bin/bash
set -euo pipefail

# Deep-K NATS Deployment Script
# Treats Deep-K as a black box and wires it to NATS JetStream

echo "🧠 Deep-K NATS Deployment Setup"
echo "================================"

# Configuration
DEEPK_DIR="/opt/deepk"
NATS_DATA_DIR="/opt/nats-data"
DEEPK_USER="deepk"

# Check if we're on Linux (required for production deployment)
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "⚠️  This script is for Linux deployment. For local dev, use the Docker commands below."
    echo ""
    echo "For local development on Windows/macOS:"
    echo "1. Use Docker Desktop"
    echo "2. Run the NATS container: docker run -d --name nats -p 4222:4222 -p 8222:8222 nats:2.10 -js"
    echo "3. Build with: cargo build --release"
    echo "4. Run with environment variables as shown in this script"
    exit 1
fi

echo "🔧 Setting up Deep-K production environment..."

# 1. Install Rust if not present
if ! command -v cargo &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    rustup default stable
fi

# 2. Install system dependencies
echo "📦 Installing system dependencies..."
sudo apt-get update
sudo apt-get install -y pkg-config build-essential curl jq docker.io docker-compose

# 3. Create deepk user
if ! id "$DEEPK_USER" &>/dev/null; then
    echo "👤 Creating deepk user..."
    sudo useradd -r -m -s /bin/bash "$DEEPK_USER"
fi

# 4. Setup directories
echo "📁 Setting up directories..."
sudo mkdir -p "$DEEPK_DIR"
sudo mkdir -p "$NATS_DATA_DIR"
sudo chown -R "$DEEPK_USER:$DEEPK_USER" "$DEEPK_DIR"
sudo chown -R "$DEEPK_USER:$DEEPK_USER" "$NATS_DATA_DIR"

# 5. Start Docker service
echo "🐳 Starting Docker service..."
sudo systemctl enable docker
sudo systemctl start docker

# 6. Setup NATS JetStream
echo "🚀 Setting up NATS JetStream..."

# Create NATS container
sudo docker run -d --name nats \
    --restart always \
    -p 4222:4222 -p 8222:8222 \
    -v "$NATS_DATA_DIR:/data" \
    nats:2.10 -js -sd /data

# Wait for NATS to be ready
echo "⏳ Waiting for NATS to be ready..."
sleep 5

# Create JetStream streams
echo "📊 Creating JetStream streams..."

# PERCEPTION stream for inbound events
sudo docker run --rm --network host natsio/nats-box:latest \
    nats stream add PERCEPTION \
    --subjects "events.perception.*" \
    --storage file \
    --retention limits \
    --max-bytes 2GB \
    --max-age 48h \
    --replicas 1

# DEEPK stream for Deep-K alerts
sudo docker run --rm --network host natsio/nats-box:latest \
    nats stream add DEEPK \
    --subjects "deepk.alerts.*" \
    --storage file \
    --retention limits \
    --max-bytes 2GB \
    --max-age 48h \
    --replicas 1

# LITEBRAIN stream for comparison (optional)
sudo docker run --rm --network host natsio/nats-box:latest \
    nats stream add LITEBRAIN \
    --subjects "litebrain.alerts.*" \
    --storage file \
    --retention limits \
    --max-bytes 1GB \
    --max-age 24h \
    --replicas 1

echo "✅ NATS JetStream setup complete!"

# 7. Build Deep-K (if source is available)
if [[ -f "Cargo.toml" ]]; then
    echo "🔨 Building Deep-K..."
    
    # Copy source to deployment directory
    sudo cp -r . "$DEEPK_DIR/source/"
    sudo chown -R "$DEEPK_USER:$DEEPK_USER" "$DEEPK_DIR/source/"
    
    # Build as deepk user
    sudo -u "$DEEPK_USER" bash -c "
        cd '$DEEPK_DIR/source'
        cargo build --release --locked
        mkdir -p '$DEEPK_DIR/bin'
        cp target/release/* '$DEEPK_DIR/bin/' 2>/dev/null || true
    "
    
    # List available binaries
    echo "📋 Available Deep-K binaries:"
    sudo -u "$DEEPK_USER" cargo metadata --format-version=1 --manifest-path="$DEEPK_DIR/source/Cargo.toml" | \
        jq -r '.packages[].targets[] | select(.kind[]=="bin") | .name'
fi

echo ""
echo "🎉 Deep-K NATS environment setup complete!"
echo ""
echo "Next steps:"
echo "1. Create systemd service: sudo ./create-deepk-service.sh"
echo "2. Test with: ./test-deepk-nats.sh"
echo "3. Monitor with: journalctl -u deepk -f"
echo ""
echo "NATS Management:"
echo "- NATS UI: http://localhost:8222"
echo "- Stream info: docker run --rm --network host natsio/nats-box:latest nats stream report"
echo "- Publish test: docker run --rm --network host -i natsio/nats-box:latest nats pub events.perception.frontdoor < test.json"
echo "- Subscribe: docker run --rm --network host natsio/nats-box:latest nats sub 'deepk.alerts.*'"

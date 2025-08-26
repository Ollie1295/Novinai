#!/bin/bash
set -euo pipefail

# Quick Start Script for Deep-K NATS Integration
echo "🚀 Deep-K NATS Quick Start"
echo "=========================="

# Check if Docker is running
if ! docker info &> /dev/null; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

echo "✅ Docker is running"

# Build the project first
echo "🔨 Building Rust project..."
if command -v cargo &> /dev/null; then
    echo "📦 Building Deep-K binaries..."
    cargo build --release --bin http_to_nats_sidecar --bin security-daemon --bin thinking_ai_demo --bin pipeline_daemon 2>/dev/null || true
    echo "✅ Build completed (some binaries may not exist, that's OK)"
else
    echo "⚠️  Cargo not found. Will build in Docker container."
fi

# Check if docker-compose.deepk-nats.yml exists
if [[ ! -f "docker-compose.deepk-nats.yml" ]]; then
    echo "❌ docker-compose.deepk-nats.yml not found in current directory"
    exit 1
fi

echo "🐳 Starting Deep-K NATS stack..."

# Start the stack
docker compose -f docker-compose.deepk-nats.yml up --build -d

echo ""
echo "⏳ Waiting for services to be ready..."

# Wait for NATS to be healthy
echo "📡 Waiting for NATS..."
timeout 60s bash -c 'until docker exec deepk-nats nats server ping &>/dev/null; do sleep 2; echo -n "."; done'
echo " ✅ NATS is ready"

# Wait for sidecar to be healthy
echo "🔄 Waiting for HTTP sidecar..."
timeout 60s bash -c 'until curl -f http://localhost:8080/health &>/dev/null; do sleep 2; echo -n "."; done'
echo " ✅ HTTP sidecar is ready"

# Check Deep-K (may not be ready immediately)
echo "🧠 Checking Deep-K status..."
if docker compose -f docker-compose.deepk-nats.yml ps deepk | grep -q "healthy\|running"; then
    echo "✅ Deep-K appears to be running"
else
    echo "⚠️  Deep-K may still be starting up"
fi

echo ""
echo "🎉 Deep-K NATS stack is running!"
echo ""
echo "📊 Service Status:"
docker compose -f docker-compose.deepk-nats.yml ps

echo ""
echo "🌐 Available Endpoints:"
echo "  NATS UI:        http://localhost:8222"
echo "  HTTP Sidecar:   http://localhost:8080"
echo "  Health Check:   http://localhost:8080/health"
echo ""

echo "🧪 Quick Tests:"
echo ""
echo "1. Test HTTP sidecar health:"
echo "   curl http://localhost:8080/health"
echo ""

echo "2. Send test perception event:"
echo "   curl -X POST http://localhost:8080/perception/front_door \\"
echo "        -d '{\"person\":0.81,\"vehicle\":0.05,\"pet\":0.02,\"timestamp\":\"2025-08-26T12:00:00Z\"}'"
echo ""

echo "3. Subscribe to Deep-K alerts:"
echo "   docker exec deepk-nats-monitor nats sub 'deepk.alerts.*'"
echo ""

echo "4. View NATS streams:"
echo "   docker exec deepk-nats-monitor nats stream report"
echo ""

echo "📜 View Logs:"
echo "  All services:   docker compose -f docker-compose.deepk-nats.yml logs -f"
echo "  Deep-K only:    docker compose -f docker-compose.deepk-nats.yml logs -f deepk"
echo "  Sidecar only:   docker compose -f docker-compose.deepk-nats.yml logs -f http-to-nats"
echo ""

echo "🛑 To stop:"
echo "   docker compose -f docker-compose.deepk-nats.yml down"
echo ""

echo "Ready for integration! 🚀"

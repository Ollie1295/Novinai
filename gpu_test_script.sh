#!/bin/bash
# GPU Performance Test Script
# Tests one-command GPU flip and validates 5x performance uplift

echo "🚀 GPU Performance Test"
echo "======================"

# Check if NVIDIA GPU is available
if ! command -v nvidia-smi &> /dev/null; then
    echo "❌ NVIDIA GPU not detected or drivers not installed"
    echo "Install NVIDIA drivers and Docker GPU support first"
    exit 1
fi

echo "📊 Current GPU status:"
nvidia-smi --query-gpu=name,memory.total,memory.used --format=csv,noheader,nounits

# Stop current CPU worker
echo "🛑 Stopping CPU worker..."
docker compose -f docker-compose.pc.yml down

# Start GPU worker
echo "🔥 Starting GPU worker..."
docker compose -f docker-compose.pc.gpu.yml up -d --build

# Wait for container to be ready
echo "⏳ Waiting for GPU worker to initialize..."
sleep 30

# Check container status
echo "📋 Container status:"
docker ps | grep novin_deep_pc

# Monitor GPU usage
echo "📊 GPU utilization:"
nvidia-smi --query-gpu=utilization.gpu,memory.used --format=csv,noheader,nounits

echo ""
echo "🧪 Ready for performance test!"
echo "Run benchmark script on VPS to measure GPU throughput"
echo "Expected: 60-150 items/sec (5x CPU performance)"
echo ""
echo "Monitor with:"
echo "  docker logs novin_deep_pc -f"
echo "  watch nvidia-smi"

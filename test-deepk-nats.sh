#!/bin/bash
set -euo pipefail

# Deep-K NATS Integration Test Script

echo "🧪 Deep-K NATS Integration Test"
echo "==============================="

# Test sample perception event (matches your LiteBrain schema)
cat > /tmp/test-perception.json << 'EOF'
{
  "cameraId": "front_door",
  "person": 0.81,
  "vehicle": 0.05,
  "pet": 0.02,
  "linger": 0.62,
  "centroid": {"x": 0.48, "y": 0.39},
  "timestamp": "2025-08-26T12:34:56Z",
  "eventId": "test_event_001",
  "confidence": 0.85,
  "metadata": {
    "source": "test_script",
    "version": "1.0"
  }
}
EOF

echo "📋 Test perception event created:"
cat /tmp/test-perception.json | jq .

echo ""
echo "🔍 Checking NATS JetStream status..."

# Check if NATS container is running
if ! docker ps | grep -q deepk-nats; then
    echo "❌ NATS container not running. Starting stack..."
    if [[ -f "docker-compose.deepk-nats.yml" ]]; then
        echo "🐳 Starting Deep-K NATS stack..."
        docker compose -f docker-compose.deepk-nats.yml up -d nats
        sleep 5
    else
        echo "❌ Neither NATS container nor docker-compose.deepk-nats.yml found."
        echo "💡 Run ./start-deepk-nats.sh first."
        exit 1
    }
fi

echo "✅ NATS container is running"

# Check JetStream status
echo "📊 JetStream streams:"
docker run --rm --network host natsio/nats-box:latest \
    nats stream report

echo ""
echo "🔍 Checking Deep-K service status..."

# Check if Deep-K service is running
if systemctl is-active --quiet deepk 2>/dev/null; then
    echo "✅ Deep-K service is running"
    
    # Show recent logs
    echo "📜 Recent Deep-K logs:"
    journalctl -u deepk --no-pager -n 10
else
    echo "⚠️  Deep-K service is not running"
    echo "💡 Start it with: sudo systemctl start deepk"
    echo "💡 Check logs with: journalctl -u deepk -f"
fi

echo ""
echo "🧪 Running integration tests..."

# Test 1: Publish perception event
echo "1️⃣  Publishing test perception event..."

docker run --rm --network host -i natsio/nats-box:latest \
    nats pub events.perception.frontdoor --stdin < /tmp/test-perception.json

echo "✅ Event published to events.perception.frontdoor"

# Test 2: Wait and check for Deep-K response
echo "2️⃣  Waiting for Deep-K processing (5 seconds)..."
sleep 5

echo "3️⃣  Checking for Deep-K alerts..."

# Try to get one message from Deep-K alerts
timeout 10s docker run --rm --network host natsio/nats-box:latest \
    nats sub 'deepk.alerts.*' --count=1 --timeout=5s || {
    echo "⚠️  No Deep-K alerts received within 5 seconds"
    echo "💡 This could mean:"
    echo "   - Deep-K is not subscribed to events.perception.*"
    echo "   - Deep-K is not publishing to deepk.alerts.*"
    echo "   - Deep-K binary needs NATS integration code"
}

echo ""
echo "🔧 Diagnostic commands:"
echo ""

echo "📡 Subscribe to all Deep-K alerts:"
echo "   docker run --rm --network host natsio/nats-box:latest nats sub 'deepk.alerts.*'"
echo ""

echo "📡 Subscribe to all perception events:"
echo "   docker run --rm --network host natsio/nats-box:latest nats sub 'events.perception.*'"
echo ""

echo "📨 Send another test event:"
echo "   docker run --rm --network host -i natsio/nats-box:latest nats pub events.perception.backdoor < /tmp/test-perception.json"
echo ""

echo "📊 Monitor stream activity:"
echo "   docker run --rm --network host natsio/nats-box:latest nats stream info PERCEPTION"
echo "   docker run --rm --network host natsio/nats-box:latest nats stream info DEEPK"
echo ""

echo "🔍 Deep-K service management:"
echo "   sudo systemctl status deepk"
echo "   journalctl -u deepk -f"
echo "   sudo systemctl restart deepk"
echo ""

echo "🌐 NATS monitoring UI:"
echo "   Open http://localhost:8222 in your browser"
echo ""

# Performance test
echo "⚡ Performance test (optional):"
echo "   # Send 100 events rapidly"
echo '   for i in {1..100}; do'
echo '     echo "{\"cameraId\":\"perf_test\",\"person\":0.75,\"eventId\":\"perf_$i\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}" |'
echo '     docker run --rm --network host -i natsio/nats-box:latest nats pub events.perception.perf_test --stdin'
echo '   done'

echo ""
echo "🧪 Test completed!"
echo "💡 Monitor Deep-K logs in real-time: journalctl -u deepk -f"

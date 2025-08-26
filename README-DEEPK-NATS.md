# Deep-K NATS Deployment Guide

## 🧠 Overview

This deployment treats Deep-K as a **black box** and wires it to NATS JetStream for event-driven architecture. No code changes required!

## 🚀 Quick Start

### Local Development (Docker)

```bash
# 1. Start NATS + Deep-K stack
docker compose -f docker-compose.deepk-nats.yml up -d

# 2. Monitor logs
docker compose -f docker-compose.deepk-nats.yml logs -f deepk

# 3. Test with sample event
echo '{"cameraId":"front_door","person":0.81,"vehicle":0.05,"pet":0.02,"timestamp":"2025-08-26T12:34:56Z"}' | \
docker exec -i deepk-nats-monitor nats pub events.perception.frontdoor --stdin

# 4. Watch for Deep-K alerts
docker exec deepk-nats-monitor nats sub 'deepk.alerts.*'
```

### Production VPS Deployment

```bash
# 1. Deploy infrastructure
chmod +x deploy-deepk-nats.sh
sudo ./deploy-deepk-nats.sh

# 2. Create systemd service
chmod +x create-deepk-service.sh
sudo ./create-deepk-service.sh

# 3. Start Deep-K
sudo systemctl start deepk

# 4. Test integration
chmod +x test-deepk-nats.sh
./test-deepk-nats.sh
```

## 📊 Event Flow

```
[LiteBrain] → HTTP POST → [HTTP-to-NATS Sidecar] → [NATS JetStream] → [Deep-K AI] → [Alerts]
                                    ↓
[Mobile App] → POST /perception/camera_id → events.perception.camera_id → Deep-K → deepk.alerts.*
```

## 🌐 Integration Points

### For LiteBrain (iOS/Xcode)

Instead of sending events directly to Deep-K, send to the HTTP-to-NATS sidecar:

```swift
// Before: Direct to Deep-K
POST https://your-deepk-server/analyze

// After: Via NATS sidecar
POST http://localhost:8080/perception/front_door
```

### For Existing HTTP Integrations

The sidecar provides HTTP endpoints that forward to NATS:

- `GET /health` - Health check
- `POST /perception/:camera_id` - Forward perception events
- `POST /forward/:subject` - Forward to any NATS subject

## 📡 NATS Subjects (Topics)

| Subject | Purpose | Producer | Consumer |
|---------|---------|----------|----------|
| `events.perception.*` | Inbound perception events | LiteBrain, Mobile App | Deep-K |
| `deepk.alerts.*` | Deep-K analysis results | Deep-K | Alert handlers |
| `litebrain.alerts.*` | LiteBrain results (optional) | LiteBrain | Comparison systems |

## 🔧 Configuration

### Environment Variables

**Deep-K Service:**
- `DEEPK_NATS_URL` - NATS server URL (default: `nats://127.0.0.1:4222`)
- `DEEPK_IN_SUBJECT` - Input subject pattern (default: `events.perception.*`)
- `DEEPK_OUT_SUBJECT` - Output subject (default: `deepk.alerts.main`)
- `RUST_LOG` - Log level (default: `info`)

**HTTP-to-NATS Sidecar:**
- `NATS_URL` - NATS server URL
- `HTTP_PORT` - HTTP server port (default: `8080`)

### JetStream Streams

| Stream | Subjects | Retention | Max Size |
|--------|----------|-----------|----------|
| PERCEPTION | `events.perception.*` | 48h | 2GB |
| DEEPK | `deepk.alerts.*` | 48h | 2GB |
| LITEBRAIN | `litebrain.alerts.*` | 24h | 1GB |

## 🏗️ Architecture Components

### Core Services

1. **NATS JetStream** - Event fabric and message durability
2. **Deep-K AI** - Your Rust AI processing (unchanged)
3. **HTTP-to-NATS Sidecar** - Bridge for existing HTTP integrations
4. **NATS Monitor** - Debugging and monitoring tools

### Deployment Options

| Component | Local Dev | VPS Production |
|-----------|-----------|----------------|
| NATS | Docker container | Docker container |
| Deep-K | Docker container | systemd service |
| Sidecar | Docker container | systemd service |

## 🧪 Testing & Monitoring

### Test Commands

```bash
# Publish test event
echo '{"cameraId":"test","person":0.75,"timestamp":"2025-08-26T12:00:00Z"}' | \
nats pub events.perception.test --stdin

# Subscribe to alerts
nats sub 'deepk.alerts.*'

# Stream statistics
nats stream report

# Performance test
for i in {1..100}; do
  echo "{\"cameraId\":\"perf_test\",\"person\":0.75,\"eventId\":\"perf_$i\"}" | \
  nats pub events.perception.perf_test --stdin
done
```

### Monitoring

- **NATS UI**: http://localhost:8222
- **Deep-K Logs**: `journalctl -u deepk -f`
- **Sidecar Health**: `curl http://localhost:8080/health`

## 🔍 Troubleshooting

### Common Issues

1. **Deep-K not receiving events**
   - Check NATS connection: `nats server check`
   - Verify subject subscription in Deep-K logs
   - Test with: `nats sub 'events.perception.*'`

2. **No Deep-K alerts**
   - Check Deep-K is publishing: `nats sub 'deepk.alerts.*'`
   - Review Deep-K logs for errors: `journalctl -u deepk -f`
   - Verify NATS permissions

3. **HTTP sidecar connection issues**
   - Check sidecar health: `curl http://localhost:8080/health`
   - Verify NATS connection in sidecar logs

### Diagnostic Commands

```bash
# Check all streams
nats stream report

# Monitor specific stream
nats stream info PERCEPTION

# Check consumer lag
nats consumer report PERCEPTION

# Test connectivity
nats server check
```

## 🚀 Performance Optimization

### Build Optimizations

```toml
# Cargo.toml
[profile.release]
lto = "thin"
codegen-units = 1
opt-level = "z"  # Size optimization
```

### Runtime Settings

```bash
# Set worker threads to CPU cores
export TOKIO_WORKER_THREADS=$(nproc)

# Enable performance monitoring
export RUST_LOG=deepk=info,nats=warn
```

### NATS JetStream Limits

```bash
# Set retention policies
nats stream add PERCEPTION \
  --max-bytes 2GB \
  --max-age 48h \
  --max-msgs 1000000
```

## 🔐 Security

### Production Security

- Deep-K runs as non-root user (`deepk`)
- Systemd security restrictions enabled
- NATS authentication can be added
- TLS encryption available for NATS connections

### Network Security

- NATS port (4222) should be firewalled
- HTTP sidecar can use HTTPS with certificates
- Consider VPN or private networks for multi-node deployment

## 📈 Scaling

### Horizontal Scaling

- Multiple Deep-K instances can subscribe to same subjects
- NATS JetStream provides load balancing
- Each instance processes different events automatically

### Vertical Scaling

- Increase `TOKIO_WORKER_THREADS` for more CPU cores
- Adjust JetStream memory limits
- Monitor queue depths and processing latency

---

## 🎯 Next Steps

1. **Deploy locally** with Docker Compose
2. **Test integration** with your LiteBrain
3. **Deploy to VPS** with systemd
4. **Monitor performance** and optimize
5. **Scale horizontally** as needed

The beauty of this approach is that Deep-K remains unchanged - it's just wired to an event bus for clean integration! 🚀

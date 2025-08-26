# PC Deep Worker Setup Guide

## Quick Start Checklist

### 1. Prerequisites
- [ ] Docker Desktop installed (Windows/Mac) or Docker + Docker Compose (Linux)
- [ ] For GPU: NVIDIA drivers + Docker GPU support
- [ ] Python 3.10+ (optional, for bare-metal)

### 2. Configuration
- [ ] Edit `.env.pc` with your VPS credentials:
  ```bash
  REDIS_URL=redis://YOUR_VPS_IP:6379/0
  PG_DSN=postgres://novin:novin@YOUR_VPS_IP:5432/novin
  R2_S3_ENDPOINT=https://YOUR_R2_ENDPOINT
  R2_ACCESS_KEY_ID=YOUR_ACCESS_KEY
  R2_SECRET_ACCESS_KEY=YOUR_SECRET_KEY
  ```

### 3. VPS Firewall Setup
- [ ] Allow your PC IP to access Redis (port 6379)
- [ ] Allow your PC IP to access Postgres (port 5432)
- [ ] Or set up WireGuard tunnel for security

### 4. Start Deep Worker

#### Option A: CPU-Only (Safe Start)
```bash
docker compose -f docker-compose.pc.yml up --build
```

#### Option B: NVIDIA GPU (Better Performance)
```bash
docker compose -f docker-compose.pc.gpu.yml up --build
```

### 5. Performance Tuning

#### If CPU struggles:
- Lower `BATCH=8` in docker-compose file
- Increase `BATCH_WINDOW_MS=120`
- Use `MODEL_NAME=yolov8n-int8`

#### If GPU has VRAM issues:
- Lower `BATCH=32`
- Monitor with `nvidia-smi`

### 6. Monitoring
- [ ] Check logs: `docker logs novin_deep_pc -f`
- [ ] Verify Redis connection: Worker should show "Health metrics" every 30s
- [ ] Test with AI Session from your VPS scheduler

### 7. Fallback to Remote GPU
If PC can't handle load, edit `.env.pc`:
```bash
DEVICE=remote
REMOTE_DEEP_ENDPOINT=https://gpu.novin.uk/deep
```

## Expected Performance

| Hardware | Batch Size | Model | Throughput |
|----------|------------|-------|------------|
| CPU i7 | 8-16 | yolov8n-int8 | 3-8 imgs/s |
| RTX 3060 | 32-64 | yolov8s-int8-trt | 60-120 imgs/s |
| RTX 3080/3090 | 64-128 | yolov8s-int8-trt | 150-250 imgs/s |

## Troubleshooting

### Connection Issues
```bash
# Test Redis connection
docker exec novin_deep_pc python -c "import redis; r=redis.from_url('redis://YOUR_VPS_IP:6379/0'); print(r.ping())"
```

### Model Loading Issues
- Check `./models/` directory for ONNX/TensorRT files
- Falls back to PyTorch `.pt` files automatically
- Download models: `wget https://github.com/ultralytics/assets/releases/download/v0.0.0/yolov8n.pt`

### Performance Issues
- Monitor batch processing time in logs
- Adjust `BATCH` and `BATCH_WINDOW_MS` based on hardware
- Use `htop` or Task Manager to monitor CPU/RAM usage
- Use `nvidia-smi` to monitor GPU usage

## Integration with VPS
- Deep worker reads from `stream:deep.jobs`
- Writes results to `stream:deep.results`
- VPS scheduler should lower `K` values initially to match PC capacity
- Start with Guardian K=12, Pro K=20 until stable

# 🎯 Reality Check: CPU Deep Worker Performance

## **Current Status: Bring-Up Complete, Not Production Scale**

### **Honest Performance Expectations (CPU)**
- **Throughput**: 3-8 items/second (not "thousands")
- **Batch Size**: 16 images 
- **Latency**: 1.5-3.0s per batch (P95)
- **Model**: YOLOv8n INT8 at 640px

### **What We've Achieved ✅**
- Deep worker connects to VPS Redis/PostgreSQL
- Image preloader with priority queues and caching
- Event pipeline integration
- Docker containerization (CPU + GPU variants)
- Basic monitoring and health checks

### **What We Need for Production 🔧**

#### 1. **Prove Current Throughput**
```bash
# Benchmark script created but container startup failing
python benchmark_deep_worker.py
```
**Target**: P95 ≤ 2s, ≥5 items/sec sustained, no backlog growth

#### 2. **Auto-Throttling Guardrails**
- K shrink when `stream:deep.jobs` > 1,000
- Token bucket clamping for overload
- Early exit enforcement (conf < 0.2)

#### 3. **Core Metrics**
- `deep_batches_total`, `deep_items_total`
- `deep_early_exits_total`
- `batch_p50_ms`, `batch_p95_ms`
- `queue_depth_deep_jobs`

#### 4. **Security Hardening**
- Firewall: PC IP + VPS only
- SSRF guards active
- R2 creds scoped to evidence bucket
- Short-lived URLs (≤60s)

### **Scale Path: GPU Deployment**
```bash
docker compose -f docker-compose.pc.gpu.yml up -d
# BATCH=64, MODEL_NAME=yolov8s-int8-trt
# Target: 60-150 items/sec (RTX 3060/3070)
```

### **Tier Reality Check**
- **Lite**: Device/VPS CPU with explainer (always-on feel)
- **Guardian (£10)**: 5 deep reviews/day, 20+ moments each
- **Pro (£20)**: Priority queue, token-bucketed, no hard stops

### **Acceptance Criteria**
- [ ] CPU sustains ≥5 items/sec, P95 ≤2s, no backlog growth
- [ ] Auto-shrink K under load, no session failures  
- [ ] Early exit skips re-ID/face for empty frames
- [ ] Redis/Postgres access restricted
- [ ] End-to-end digest < 60s
- [ ] GPU switch increases throughput ≥5×

## **Current Claim: "Deep AI live with VPS connectivity, CPU handles small loads, GPU unlocks scale"**

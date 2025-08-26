# 🔒 Production Hardening Checklist

## **Critical Path: Proof-of-Concept → Hardened MVP**

### **⚠️ Non-Negotiables Before Real Load**

#### **1. Auto-Throttling Guardrails**
```python
# deep_worker.py - Add queue monitoring
if queue_depth > 1000:
    K = max(4, K // 2)  # Shrink K
    BATCH_WINDOW_MS += 20  # Widen window
```
**Risk**: No guardrails = instant backlog death spiral

#### **2. Metrics Visibility** 
```python
# Prometheus endpoint at :9102/metrics
deep_batches_total
deep_items_total  
batch_p95_ms
queue_depth_deep_jobs
early_exits_total
```
**Risk**: Won't know you're drowning until alerts fail

#### **3. Security Surface**
```bash
# Lock down Redis/PostgreSQL
sudo ufw delete allow 6379/tcp
sudo ufw delete allow 5432/tcp
sudo ufw allow from YOUR_PC_IP to any port 6379
sudo ufw allow from YOUR_PC_IP to any port 5432
```
**Risk**: Open to internet = fatal footgun

#### **4. GPU Fallback Path**
```bash
# Test one-command GPU flip
docker compose -f docker-compose.pc.gpu.yml up -d --build
# Confirm 5x uplift: 60-150 items/sec
```
**Risk**: CPU stalls at 5K-10K homes without GPU ready

### **📊 Honest Performance Pitch**
- **CPU node**: ~500K Top-K events/month for $0
- **GPU node**: 20-40M Top-K events/month at <$200 infra  
- **Marginal cost**: $0.0002-0.001 per deep review
- **ARPU**: £10-20, **Margins**: 80-90%

### **🧭 This Week Priority**
1. **Auto-throttling**: Wire K shrink + window widening
2. **Metrics**: Prometheus endpoint + Grafana dashboard
3. **Security**: Firewall lockdown or WireGuard tunnel
4. **GPU test**: Prove 5x performance uplift
5. **Load simulation**: 1K jobs, validate backlog drains flat

### **🎯 Current Status**
**Good enough for**: Investors, early users, pilot customers  
**Not ready for**: Real production load without hardening

**Next Decision**: Prometheus dashboard layout or GPU switch test?

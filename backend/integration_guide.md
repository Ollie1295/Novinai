# Complete Backend Architecture Integration Guide

This document describes the complete backend architecture for the AI security system with on-device processing support.

## Architecture Overview

The system consists of several interconnected components:

1. **Candidate Store** - Redis ZSET-based intelligent event prioritization
2. **Scheduler** - Top-K event selection with token bucket rate limiting  
3. **Deep Worker V2** - Redis queue-integrated batch processing worker
4. **Server-Lite Scorer** - CPU fallback for mobile lite processing
5. **Mobile API** - FastAPI endpoints for mobile device integration
6. **Digest Service** - Notification processing and summary generation

## Component Relationships

```
[Mobile Apps] 
    ↓ (events/lite results)
[Mobile API] 
    ↓ (creates candidates)
[Candidate Store] ← (ZSET priorities) → [Scheduler] 
    ↓ (enqueues jobs)                        ↓ (rate limiting)
[Redis Queues] 
    ↓ (batch processing)
[Deep Workers] 
    ↓ (results)
[Digest Queue] → [Digest Service] → [Push Notifications]
```

## Redis Data Structure

### Keys Used:
- `cand:{home_id}` - ZSET of event candidates by priority score
- `ev:{event_id}` - Hash containing full event candidate data
- `deep_processing_{tier}` - Lists for processing job queues
- `processing:{event_id}` - Timeout tracking for jobs
- `digest_queue` - Completed processing results for notifications
- `scheduler_completions` - Worker completion notifications
- `worker_metrics` - Performance metrics
- `lite_scorer_metrics` - Server-lite processing metrics

## Deployment Instructions

### 1. Prerequisites

```bash
# Install dependencies
pip install redis[hiredis] asyncio uvicorn fastapi httpx pillow torch torchvision

# Start Redis
redis-server

# Setup PostgreSQL database
psql -f mobile_backend_setup.sql
```

### 2. Start Core Services

#### A. Server-Lite Scorer (Port 8002)
```bash
cd backend/services
python server_lite_scorer.py server
```

#### B. Mobile API (Port 8001)  
```bash
cd ../
python mobile_api_endpoints.py
```

### 3. Start Processing Components

#### A. Scheduler Daemon
```python
# In Python REPL or script
import asyncio
from backend.services.candidate_store import CandidateStore
from backend.services.scheduler import ProcessingScheduler, SchedulerDaemon

async def run_scheduler():
    store = CandidateStore()
    await store.initialize()
    
    scheduler = ProcessingScheduler(store)
    await scheduler.initialize()
    
    daemon = SchedulerDaemon(scheduler, schedule_interval=30)
    await daemon.start()

asyncio.run(run_scheduler())
```

#### B. Deep Workers
```python
# Start 2 deep workers
import asyncio
from backend.services.deep_worker_v2 import WorkerManager

async def run_workers():
    manager = WorkerManager()
    await manager.initialize()
    await manager.start_workers(worker_count=2)
    
    # Keep running
    while True:
        await asyncio.sleep(60)

asyncio.run(run_workers())
```

### 4. Test the System

#### A. Submit Test Event
```bash
curl -X POST http://localhost:8001/events \
  -H "Authorization: Bearer test_token" \
  -H "Content-Type: application/json" \
  -d '{
    "image_url": "https://example.com/test.jpg",
    "device_id": "test_device_001",
    "location": "front_door",
    "mode": "security",
    "motion_score": 0.8
  }'
```

#### B. Check Component Status
```bash
# Mobile API health
curl http://localhost:8001/health

# Server-Lite Scorer health  
curl http://localhost:8002/health

# Scheduler stats (from Redis)
redis-cli LRANGE scheduler_metrics 0 -1

# Worker stats
redis-cli LRANGE worker_metrics 0 -1
```

## Integration Points

### 1. Mobile API → Candidate Store
When events are created:
```python
from backend.services.candidate_store import create_event_candidate_from_api

# In mobile_api_endpoints.py
candidate = await create_event_candidate_from_api(event_data, lite_results)
await candidate_store.add_candidate(candidate)
```

### 2. Scheduler → Deep Workers
Events are scheduled based on:
- Processing tier limits (30/60/120 events per minute)
- Top-K priority scores
- Token bucket rate limiting

### 3. Deep Workers → Digest Service
Completed jobs are added to digest queue:
```python
digest_data = {
    "event_id": job.event_id,
    "user_id": job.user_id,
    "result": processing_result,
    "completed_at": datetime.utcnow().isoformat()
}
await redis_client.lpush("digest_queue", json.dumps(digest_data))
```

### 4. Server-Lite Scorer Fallback
When mobile devices cannot process:
```python
from backend.services.server_lite_scorer import LiteScorerClient

client = LiteScorerClient()
result = await client.classify_image(event_id, image_url, user_id, "device_unavailable")
```

## Configuration

### Environment Variables
```bash
# Redis connection
export REDIS_URL="redis://localhost:6379"

# Database
export DATABASE_URL="postgresql://user:pass@localhost/mobile_backend"

# JWT secret
export JWT_SECRET_KEY="your-secret-key"

# Push notifications
export APNS_KEY_PATH="/path/to/apns.p8"
export APNS_KEY_ID="your-key-id"
export APNS_TEAM_ID="your-team-id"
export FCM_SERVER_KEY="your-fcm-key"

# Server-Lite Scorer
export LITE_SCORER_URL="http://localhost:8002"
```

### Scheduling Policy Configuration
```python
# In scheduler.py
policy = SchedulingPolicy(
    tier_limits={
        ProcessingTier.STANDARD: 30,     # 30 events/min
        ProcessingTier.PREMIUM: 60,      # 60 events/min  
        ProcessingTier.ENTERPRISE: 120   # 120 events/min
    },
    max_batch_size=10,
    top_k_limit=50,
    processing_timeout=300  # 5 minutes
)
```

## Monitoring and Metrics

### Key Metrics to Track:
1. **Candidate Store**: Total candidates, score distribution
2. **Scheduler**: Events scheduled per tier, rate limiting stats
3. **Deep Workers**: Batch processing throughput, success rates
4. **Server-Lite Scorer**: Fallback processing volume, accuracy

### Health Checks:
```bash
# All services should respond with 200 OK
curl http://localhost:8001/health  # Mobile API
curl http://localhost:8002/health  # Server-Lite Scorer

# Redis should be accessible
redis-cli ping

# Check queue depths
redis-cli LLEN deep_processing_premium
redis-cli LLEN digest_queue
```

## Scaling Considerations

### Horizontal Scaling:
- **Deep Workers**: Can run multiple worker processes/containers
- **Server-Lite Scorer**: Can run multiple instances behind load balancer
- **Redis**: Use Redis Cluster for high availability
- **Scheduler**: Currently single instance (can be made HA with leader election)

### Performance Tuning:
- Adjust batch sizes based on GPU memory
- Tune token bucket rates based on processing capacity
- Configure Redis memory policies for candidate cleanup
- Monitor and adjust candidate TTL values

## Troubleshooting

### Common Issues:

1. **Events not being processed**
   - Check scheduler daemon is running
   - Verify Redis connectivity
   - Check token bucket status

2. **High server-lite scorer load**
   - Scale up mobile device processing
   - Add more server-lite instances
   - Check fallback reasons in metrics

3. **Processing timeouts**
   - Adjust processing_timeout in scheduler policy
   - Check deep worker batch sizes
   - Monitor GPU memory usage

4. **Queue buildup**
   - Increase worker count
   - Adjust tier rate limits
   - Check for failing workers

### Debug Commands:
```bash
# Check Redis queue depths
redis-cli LLEN deep_processing_premium
redis-cli LLEN deep_processing_standard

# View recent metrics
redis-cli LRANGE scheduler_metrics 0 4
redis-cli LRANGE worker_metrics 0 4

# Check candidate counts
redis-cli KEYS "cand:*" | wc -l
redis-cli KEYS "ev:*" | wc -l
```

This completes the backend architecture implementation with all core components integrated and ready for deployment.

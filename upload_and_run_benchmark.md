# 🚀 VPS Benchmark Setup

## **Upload and Run Benchmark Script**

```bash
# SSH to your VPS
ssh vps

# Create the benchmark script
cat > vps_benchmark.sh << 'EOF'
#!/bin/bash
# Quick Deep Worker Validation Test
# Sends 50 jobs and measures performance

echo "🔥 Quick Deep Worker Benchmark"
echo "============================="
echo "Target: P95 ≤2s, ≥3 items/sec, 90%+ completion"
echo ""

START_TIME=$(date +%s)
JOBS_SENT=0
JOBS_COMPLETED=0

# Install redis-tools if not available
if ! command -v redis-cli &> /dev/null; then
    echo "📦 Installing redis-tools..."
    sudo apt-get update -qq && sudo apt-get install -y redis-tools
fi

echo "🚀 Sending 50 test jobs to deep worker..."

for i in {1..50}; do
    # Create test job
    JOB_DATA="{
        \"session_id\": \"benchmark-${i}\",
        \"user_id\": \"test-user\",
        \"items\": [
            {\"image_url\": \"https://picsum.photos/384/384\", \"metadata\": {\"test\": true}}
        ],
        \"timestamp\": \"$(date -Iseconds)\"
    }"
    
    # Send to Redis stream
    redis-cli -h 100.85.81.109 -p 6379 -a "novin_redis_2024" XADD stream:deep.jobs '*' job_data "$JOB_DATA" > /dev/null
    
    if [ $? -eq 0 ]; then
        JOBS_SENT=$((JOBS_SENT + 1))
        echo -n '.'
    else
        echo -n 'X'
    fi
    
    # Small delay to avoid overwhelming
    sleep 0.1
done

echo ""
echo "📤 Jobs sent: $JOBS_SENT"

# Wait and check results
echo "⏳ Waiting for results (30s timeout)..."
sleep 30

# Count completed jobs
JOBS_COMPLETED=$(redis-cli -h 100.85.81.109 -p 6379 -a "novin_redis_2024" XLEN stream:deep.results 2>/dev/null || echo "0")

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))
COMPLETION_RATE=$(( (JOBS_COMPLETED * 100) / JOBS_SENT ))

echo ""
echo "📊 BENCHMARK RESULTS"
echo "==================="
echo "Jobs Sent: $JOBS_SENT"
echo "Jobs Completed: $JOBS_COMPLETED"
echo "Completion Rate: $COMPLETION_RATE%"
echo "Total Time: ${DURATION}s"

if [ $JOBS_COMPLETED -gt 0 ]; then
    ITEMS_PER_SEC=$(echo "scale=2; $JOBS_COMPLETED / $DURATION" | bc -l)
    echo "Throughput: $ITEMS_PER_SEC items/sec"
else
    ITEMS_PER_SEC=0
fi

echo ""
echo "🎯 ACCEPTANCE CRITERIA:"
if [ $COMPLETION_RATE -ge 90 ]; then
    echo "✅ Completion Rate: $COMPLETION_RATE% (≥90% required)"
else
    echo "❌ Completion Rate: $COMPLETION_RATE% (≥90% required)"
fi

if (( $(echo "$ITEMS_PER_SEC >= 3" | bc -l) )); then
    echo "✅ Throughput: $ITEMS_PER_SEC items/sec (≥3 required)"
else
    echo "❌ Throughput: $ITEMS_PER_SEC items/sec (≥3 required)"
fi

echo ""
echo "🔧 Next Steps:"
echo "- Run full benchmark: python benchmark_deep_worker.py"
echo "- Check container logs: docker logs novin_deep_pc -f"  
echo "- Scale to GPU if needed: docker-compose.pc.gpu.yml"
EOF

# Make executable and run
chmod +x vps_benchmark.sh
./vps_benchmark.sh
```

## **Expected Results**
- **PASS**: ≥90% completion, ≥3 items/sec
- **FAIL**: Optimize batch size, switch to GPU

## **Monitor Deep Worker**
```bash
# Check if PC deep worker is running
docker ps | grep novin_deep_pc

# View logs
docker logs novin_deep_pc -f
```

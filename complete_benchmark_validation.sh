#!/bin/bash
# Complete benchmark validation script for VPS
# Run this on your VPS to check results with the correct Redis password

echo "🔍 Complete Benchmark Validation"
echo "================================"

# Redis connection details
REDIS_HOST="100.85.81.109"
REDIS_PORT="6379"
REDIS_PASSWORD="novin_redis_2024"

echo "📊 Checking benchmark results..."

# Check results stream length
RESULTS_COUNT=$(redis-cli -h $REDIS_HOST -p $REDIS_PORT -a "$REDIS_PASSWORD" XLEN stream:deep.results 2>/dev/null || echo "0")
echo "Results processed: $RESULTS_COUNT"

# Check remaining jobs in queue
QUEUE_DEPTH=$(redis-cli -h $REDIS_HOST -p $REDIS_PORT -a "$REDIS_PASSWORD" XLEN stream:deep.jobs 2>/dev/null || echo "0")
echo "Jobs remaining in queue: $QUEUE_DEPTH"

# Calculate completion rate (assuming 50 jobs were sent)
JOBS_SENT=50
if [ $RESULTS_COUNT -gt 0 ]; then
    COMPLETION_RATE=$(( (RESULTS_COUNT * 100) / JOBS_SENT ))
    echo "Completion rate: $COMPLETION_RATE% ($RESULTS_COUNT/$JOBS_SENT)"
else
    echo "⚠️  No results found in stream:deep.results"
fi

# Show recent results for analysis
echo ""
echo "📋 Recent processing results:"
redis-cli -h $REDIS_HOST -p $REDIS_PORT -a "$REDIS_PASSWORD" XREVRANGE stream:deep.results + - COUNT 5

echo ""
echo "🎯 VALIDATION STATUS:"
if [ $COMPLETION_RATE -ge 90 ] && [ $RESULTS_COUNT -ge 45 ]; then
    echo "✅ PASS: Deep worker processing successfully"
    echo "   - Completion rate: $COMPLETION_RATE% (≥90% required)"
    echo "   - Jobs processed: $RESULTS_COUNT (≥45 required)"
else
    echo "⚠️  PARTIAL: Deep worker needs optimization"
    echo "   - Completion rate: $COMPLETION_RATE% (target: ≥90%)"
    echo "   - Jobs processed: $RESULTS_COUNT (target: ≥45)"
fi

echo ""
echo "🔧 Next steps:"
echo "- Check PC deep worker logs: docker logs [container_name] -f"
echo "- Monitor processing: watch 'redis-cli -h $REDIS_HOST -p $REDIS_PORT -a $REDIS_PASSWORD XLEN stream:deep.results'"
echo "- Scale to GPU if needed: docker-compose.pc.gpu.yml"

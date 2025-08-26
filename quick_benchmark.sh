#!/bin/bash
# Quick CPU benchmark - prove actual throughput

echo "🔥 Quick CPU Deep Worker Benchmark"

# Wait for container to be ready
echo "Waiting for deep worker container..."
sleep 10

# Create 50 test jobs (smaller batch for quick test)
echo "Creating 50 synthetic jobs..."
for i in $(seq 1 50); do
  redis-cli -u redis://:novin_redis_2024@95.179.193.224:6379/0 XADD stream:deep.jobs \* \
    home_id "test_home_$((i % 5))" \
    event_ids "test_event_$i" \
    K "1" \
    tier "guardian" \
    thumb_url "https://picsum.photos/384/384" \
    session_id "bench_$(date +%s)"
done

echo "✅ Created 50 jobs"

# Monitor for 2 minutes
echo "Monitoring results for 120 seconds..."
start_time=$(date +%s)
start_count=$(redis-cli -u redis://:novin_redis_2024@95.179.193.224:6379/0 XLEN stream:deep.results 2>/dev/null || echo 0)

for i in {1..24}; do  # 24 * 5s = 120s
  current_count=$(redis-cli -u redis://:novin_redis_2024@95.179.193.224:6379/0 XLEN stream:deep.results 2>/dev/null || echo 0)
  results=$((current_count - start_count))
  queue_depth=$(redis-cli -u redis://:novin_redis_2024@95.179.193.224:6379/0 XLEN stream:deep.jobs 2>/dev/null || echo 0)
  elapsed=$(($(date +%s) - start_time))
  
  if [ $elapsed -gt 0 ]; then
    throughput=$(echo "scale=2; $results / $elapsed" | bc -l 2>/dev/null || echo "0")
  else
    throughput="0"
  fi
  
  printf "\r⏱️  %3ds | Results: %2d/50 | Queue: %2d | Throughput: %s/s" $elapsed $results $queue_depth $throughput
  
  if [ $results -ge 50 ]; then
    echo -e "\n✅ All jobs completed in ${elapsed}s"
    break
  fi
  
  sleep 5
done

echo -e "\n"
echo "📊 FINAL RESULTS:"
echo "Completed: $results/50 jobs"
echo "Time: ${elapsed}s"
echo "Throughput: $throughput items/second"

# Pass/fail criteria
if (( $(echo "$throughput >= 3" | bc -l) )) && [ $results -ge 45 ]; then
  echo "🎉 PASS: CPU worker meets minimum criteria"
else
  echo "❌ FAIL: Below minimum 3 items/s or <90% completion"
  echo "Recommendations:"
  echo "- Reduce batch size to 4-6"
  echo "- Use 320px images"
  echo "- Switch to GPU for scale"
fi

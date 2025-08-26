#!/usr/bin/env python3
"""
Deep Worker Benchmark Script
Measures real CPU throughput and latency under synthetic load
"""

import os
import time
import redis
import json
import threading
from collections import defaultdict
import statistics

def load_env():
    """Load environment variables from .env.pc"""
    env_file = ".env.pc"
    if os.path.exists(env_file):
        with open(env_file, 'r') as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#') and '=' in line:
                    key, value = line.split('=', 1)
                    os.environ[key] = value

def create_synthetic_jobs(redis_client, count=200, image_url="https://picsum.photos/640/480"):
    """Create synthetic deep worker jobs"""
    print(f"🔄 Creating {count} synthetic jobs...")
    
    jobs_created = 0
    for i in range(1, count + 1):
        job_data = {
            "home_id": f"test_home_{i % 10}",
            "event_ids": f"test_event_{i}",
            "K": "1",
            "tier": "guardian" if i % 3 == 0 else "pro",
            "thumb_url": image_url,
            "session_id": f"bench_session_{int(time.time())}"
        }
        
        try:
            redis_client.xadd("stream:deep.jobs", job_data)
            jobs_created += 1
            if jobs_created % 50 == 0:
                print(f"  Created {jobs_created}/{count} jobs...")
        except Exception as e:
            print(f"❌ Failed to create job {i}: {e}")
    
    print(f"✅ Created {jobs_created} synthetic jobs")
    return jobs_created

def monitor_results(redis_client, expected_count, timeout_seconds=300):
    """Monitor results stream and measure performance"""
    print(f"📊 Monitoring results for {timeout_seconds}s...")
    
    start_time = time.time()
    results_count = 0
    latencies = []
    batch_sizes = []
    
    # Get initial results count
    try:
        initial_results = redis_client.xrange("stream:deep.results", "-", "+")
        initial_count = len(initial_results)
    except:
        initial_count = 0
    
    while time.time() - start_time < timeout_seconds:
        try:
            # Get new results
            current_results = redis_client.xrange("stream:deep.results", "-", "+")
            current_count = len(current_results)
            new_results = current_count - initial_count
            
            if new_results > results_count:
                results_count = new_results
                
                # Analyze recent results for latency
                recent_results = current_results[-10:] if len(current_results) >= 10 else current_results
                for result_id, result_data in recent_results:
                    try:
                        if b'processing_time_ms' in result_data:
                            processing_time = float(result_data[b'processing_time_ms'])
                            latencies.append(processing_time)
                        
                        if b'batch_size' in result_data:
                            batch_size = int(result_data[b'batch_size'])
                            batch_sizes.append(batch_size)
                    except:
                        pass
            
            # Check queue depth
            try:
                queue_depth = redis_client.xlen("stream:deep.jobs")
            except:
                queue_depth = 0
            
            elapsed = time.time() - start_time
            throughput = results_count / elapsed if elapsed > 0 else 0
            
            print(f"\r⏱️  {elapsed:.1f}s | Results: {results_count}/{expected_count} | "
                  f"Throughput: {throughput:.2f} items/s | Queue: {queue_depth}", end="")
            
            if results_count >= expected_count:
                print(f"\n✅ All {expected_count} jobs completed in {elapsed:.2f}s")
                break
                
        except Exception as e:
            print(f"\n❌ Monitoring error: {e}")
        
        time.sleep(1)
    
    return {
        "results_count": results_count,
        "expected_count": expected_count,
        "elapsed_time": time.time() - start_time,
        "throughput": results_count / (time.time() - start_time),
        "latencies": latencies,
        "batch_sizes": batch_sizes
    }

def analyze_performance(metrics):
    """Analyze and report performance metrics"""
    print("\n" + "="*60)
    print("📊 BENCHMARK RESULTS")
    print("="*60)
    
    # Basic metrics
    completion_rate = (metrics["results_count"] / metrics["expected_count"]) * 100
    print(f"Completion Rate: {completion_rate:.1f}% ({metrics['results_count']}/{metrics['expected_count']})")
    print(f"Total Time: {metrics['elapsed_time']:.2f}s")
    print(f"Throughput: {metrics['throughput']:.2f} items/second")
    
    # Latency analysis
    if metrics["latencies"]:
        latencies = metrics["latencies"]
        p50 = statistics.median(latencies)
        p95 = statistics.quantiles(latencies, n=20)[18] if len(latencies) >= 20 else max(latencies)
        
        print(f"\nLatency Metrics:")
        print(f"  P50: {p50:.0f}ms")
        print(f"  P95: {p95:.0f}ms")
        print(f"  Min: {min(latencies):.0f}ms")
        print(f"  Max: {max(latencies):.0f}ms")
        
        # Pass/fail criteria
        print(f"\n🎯 PASS CRITERIA:")
        p95_pass = p95 <= 2000  # 2 seconds
        throughput_pass = metrics["throughput"] >= 3  # 3 items/sec minimum
        completion_pass = completion_rate >= 95
        
        print(f"  P95 ≤ 2000ms: {'✅ PASS' if p95_pass else '❌ FAIL'} ({p95:.0f}ms)")
        print(f"  Throughput ≥ 3/s: {'✅ PASS' if throughput_pass else '❌ FAIL'} ({metrics['throughput']:.2f}/s)")
        print(f"  Completion ≥ 95%: {'✅ PASS' if completion_pass else '❌ FAIL'} ({completion_rate:.1f}%)")
        
        overall_pass = p95_pass and throughput_pass and completion_pass
        print(f"\n🏆 OVERALL: {'✅ PASS - CPU Worker Ready' if overall_pass else '❌ FAIL - Needs Optimization'}")
        
        if not overall_pass:
            print(f"\n🔧 RECOMMENDATIONS:")
            if not p95_pass:
                print(f"  • Reduce BATCH size (try 8)")
                print(f"  • Increase BATCH_WINDOW_MS (try 120)")
            if not throughput_pass:
                print(f"  • Lower input resolution (384px)")
                print(f"  • Use INT8 model weights")
                print(f"  • Consider GPU upgrade")
    
    # Batch analysis
    if metrics["batch_sizes"]:
        avg_batch = statistics.mean(metrics["batch_sizes"])
        print(f"\nBatch Metrics:")
        print(f"  Average batch size: {avg_batch:.1f}")
        print(f"  Batch efficiency: {avg_batch/16*100:.1f}% of configured size")

def main():
    """Run deep worker benchmark"""
    print("🚀 Deep Worker CPU Benchmark")
    print("="*40)
    
    # Load environment
    load_env()
    
    # Connect to Redis
    redis_url = os.getenv("REDIS_URL", "redis://:novin_redis_2024@95.179.193.224:6379/0")
    try:
        r = redis.from_url(redis_url)
        r.ping()
        print(f"✅ Connected to Redis")
    except Exception as e:
        print(f"❌ Redis connection failed: {e}")
        return 1
    
    # Clear existing streams
    try:
        r.delete("stream:deep.jobs")
        r.delete("stream:deep.results")
        print("🧹 Cleared existing streams")
    except:
        pass
    
    # Use a small test image for consistent results
    test_image_url = "https://picsum.photos/640/480"
    job_count = 200
    
    print(f"📸 Test image: {test_image_url}")
    print(f"📦 Job count: {job_count}")
    
    # Create synthetic load
    created_count = create_synthetic_jobs(r, job_count, test_image_url)
    
    if created_count == 0:
        print("❌ No jobs created, aborting benchmark")
        return 1
    
    # Monitor performance
    metrics = monitor_results(r, created_count, timeout_seconds=300)
    
    # Analyze results
    analyze_performance(metrics)
    
    return 0

if __name__ == "__main__":
    exit(main())

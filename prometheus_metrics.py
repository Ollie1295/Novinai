#!/usr/bin/env python3
"""
Prometheus Metrics Endpoint for Deep Worker
Exposes critical production metrics at :9102/metrics
"""

import time
import threading
from http.server import HTTPServer, BaseHTTPRequestHandler
from collections import defaultdict, deque
import statistics
import redis
import os

class MetricsCollector:
    def __init__(self):
        self.deep_batches_total = 0
        self.deep_items_total = 0
        self.deep_early_exits_total = 0
        self.batch_latencies = deque(maxlen=1000)  # Keep last 1000 batch times
        self.redis_client = None
        self._setup_redis()
    
    def _setup_redis(self):
        """Setup Redis connection for queue monitoring"""
        try:
            redis_url = os.getenv("REDIS_URL", "redis://:novin_redis_2024@95.179.193.224:6379/0")
            self.redis_client = redis.from_url(redis_url)
            self.redis_client.ping()
        except Exception as e:
            print(f"⚠️  Redis connection failed for metrics: {e}")
    
    def record_batch(self, batch_size, processing_time_ms, early_exits=0):
        """Record batch processing metrics"""
        self.deep_batches_total += 1
        self.deep_items_total += batch_size
        self.deep_early_exits_total += early_exits
        self.batch_latencies.append(processing_time_ms)
    
    def get_queue_depth(self):
        """Get current queue depth from Redis"""
        try:
            if self.redis_client:
                return self.redis_client.xlen("stream:deep.jobs")
        except:
            pass
        return 0
    
    def get_batch_p50(self):
        """Get P50 batch latency"""
        if not self.batch_latencies:
            return 0
        return statistics.median(self.batch_latencies)
    
    def get_batch_p95(self):
        """Get P95 batch latency"""
        if len(self.batch_latencies) < 20:
            return max(self.batch_latencies) if self.batch_latencies else 0
        return statistics.quantiles(self.batch_latencies, n=20)[18]
    
    def get_throughput(self, window_seconds=60):
        """Get items/second throughput over window"""
        if not self.batch_latencies:
            return 0
        
        # Simple approximation: items in last minute
        recent_batches = len([t for t in self.batch_latencies if time.time() * 1000 - t < window_seconds * 1000])
        avg_batch_size = self.deep_items_total / max(1, self.deep_batches_total)
        return (recent_batches * avg_batch_size) / window_seconds

# Global metrics collector
metrics = MetricsCollector()

class MetricsHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/metrics':
            self.send_response(200)
            self.send_header('Content-type', 'text/plain')
            self.end_headers()
            
            # Generate Prometheus metrics
            output = []
            
            # Counter metrics
            output.append(f"# HELP deep_batches_total Total number of batches processed")
            output.append(f"# TYPE deep_batches_total counter")
            output.append(f"deep_batches_total {metrics.deep_batches_total}")
            output.append("")
            
            output.append(f"# HELP deep_items_total Total number of items processed")
            output.append(f"# TYPE deep_items_total counter")
            output.append(f"deep_items_total {metrics.deep_items_total}")
            output.append("")
            
            output.append(f"# HELP deep_early_exits_total Total number of early exits")
            output.append(f"# TYPE deep_early_exits_total counter")
            output.append(f"deep_early_exits_total {metrics.deep_early_exits_total}")
            output.append("")
            
            # Gauge metrics
            output.append(f"# HELP batch_p50_ms P50 batch processing latency in milliseconds")
            output.append(f"# TYPE batch_p50_ms gauge")
            output.append(f"batch_p50_ms {metrics.get_batch_p50():.2f}")
            output.append("")
            
            output.append(f"# HELP batch_p95_ms P95 batch processing latency in milliseconds")
            output.append(f"# TYPE batch_p95_ms gauge")
            output.append(f"batch_p95_ms {metrics.get_batch_p95():.2f}")
            output.append("")
            
            output.append(f"# HELP queue_depth_deep_jobs Current depth of deep.jobs queue")
            output.append(f"# TYPE queue_depth_deep_jobs gauge")
            output.append(f"queue_depth_deep_jobs {metrics.get_queue_depth()}")
            output.append("")
            
            output.append(f"# HELP throughput_items_per_sec Current throughput in items per second")
            output.append(f"# TYPE throughput_items_per_sec gauge")
            output.append(f"throughput_items_per_sec {metrics.get_throughput():.2f}")
            output.append("")
            
            self.wfile.write('\n'.join(output).encode())
        
        elif self.path == '/health':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            
            health = {
                "status": "healthy",
                "batches_processed": metrics.deep_batches_total,
                "items_processed": metrics.deep_items_total,
                "queue_depth": metrics.get_queue_depth(),
                "p95_latency_ms": metrics.get_batch_p95()
            }
            
            import json
            self.wfile.write(json.dumps(health).encode())
        
        else:
            self.send_response(404)
            self.end_headers()
    
    def log_message(self, format, *args):
        # Suppress HTTP logs
        pass

def start_metrics_server(port=9102):
    """Start Prometheus metrics server"""
    server = HTTPServer(('0.0.0.0', port), MetricsHandler)
    print(f"📊 Metrics server started on :9102/metrics")
    server.serve_forever()

def start_metrics_thread():
    """Start metrics server in background thread"""
    thread = threading.Thread(target=start_metrics_server, daemon=True)
    thread.start()
    return thread

if __name__ == "__main__":
    print("🚀 Starting Prometheus metrics server...")
    start_metrics_server()

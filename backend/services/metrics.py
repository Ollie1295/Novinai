#!/usr/bin/env python3
"""
Prometheus Metrics Module
Comprehensive monitoring and alerting for production systems
"""

import logging
import time
import asyncio
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional
from dataclasses import dataclass
import redis.asyncio as redis
from prometheus_client import (
    Counter, Histogram, Gauge, Summary, Info, 
    CollectorRegistry, generate_latest, CONTENT_TYPE_LATEST,
    start_http_server, push_to_gateway
)
import psutil
import json
from threading import Thread
import traceback

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class MetricsConfig:
    """Metrics configuration"""
    # Prometheus settings
    metrics_port: int = 9090
    metrics_endpoint: str = "/metrics"
    push_gateway_url: str = None
    job_name: str = "insane-ai-security"
    
    # Collection intervals
    system_metrics_interval: int = 30  # seconds
    business_metrics_interval: int = 60  # seconds
    
    # Alert thresholds
    error_rate_threshold: float = 0.05  # 5%
    latency_p99_threshold: float = 2000  # 2 seconds
    queue_backlog_threshold: int = 1000
    memory_usage_threshold: float = 0.9  # 90%
    
    # Service-specific settings
    enable_detailed_metrics: bool = True
    enable_system_metrics: bool = True

class InsaneAIMetrics:
    """Comprehensive metrics collection for Insane AI Security system"""
    
    def __init__(self, config: MetricsConfig, redis_client=None):
        self.config = config
        self.redis_client = redis_client
        self.registry = CollectorRegistry()
        
        # Initialize metrics
        self._init_api_metrics()
        self._init_processing_metrics()
        self._init_storage_metrics()
        self._init_business_metrics()
        self._init_system_metrics()
        
        # Start metrics collection
        self.running = False
        
    def _init_api_metrics(self):
        """Initialize API-related metrics"""
        # Request metrics
        self.http_requests_total = Counter(
            'insane_ai_http_requests_total',
            'Total HTTP requests',
            ['method', 'endpoint', 'status_code', 'tier'],
            registry=self.registry
        )
        
        self.http_request_duration = Histogram(
            'insane_ai_http_request_duration_seconds',
            'HTTP request duration',
            ['method', 'endpoint', 'tier'],
            buckets=[0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
            registry=self.registry
        )
        
        self.http_request_size = Histogram(
            'insane_ai_http_request_size_bytes',
            'HTTP request size',
            ['method', 'endpoint'],
            buckets=[100, 1000, 10000, 100000, 1000000],
            registry=self.registry
        )
        
        self.http_response_size = Histogram(
            'insane_ai_http_response_size_bytes',
            'HTTP response size',
            ['method', 'endpoint', 'status_code'],
            buckets=[100, 1000, 10000, 100000, 1000000],
            registry=self.registry
        )
        
        # Authentication metrics
        self.auth_requests_total = Counter(
            'insane_ai_auth_requests_total',
            'Total authentication requests',
            ['method', 'status'],
            registry=self.registry
        )
        
        self.auth_token_validations = Counter(
            'insane_ai_auth_token_validations_total',
            'JWT token validations',
            ['status'],
            registry=self.registry
        )
        
        # Rate limiting metrics
        self.rate_limit_hits = Counter(
            'insane_ai_rate_limit_hits_total',
            'Rate limit violations',
            ['client_type', 'tier'],
            registry=self.registry
        )
        
        self.active_connections = Gauge(
            'insane_ai_active_connections',
            'Current active connections',
            registry=self.registry
        )
    
    def _init_processing_metrics(self):
        """Initialize processing-related metrics"""
        # Event processing
        self.events_created_total = Counter(
            'insane_ai_events_created_total',
            'Total events created',
            ['location', 'mode', 'tier'],
            registry=self.registry
        )
        
        self.events_processed_total = Counter(
            'insane_ai_events_processed_total',
            'Total events processed',
            ['processor_type', 'tier', 'status'],
            registry=self.registry
        )
        
        self.processing_duration = Histogram(
            'insane_ai_processing_duration_seconds',
            'Event processing duration',
            ['processor_type', 'tier'],
            buckets=[0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0],
            registry=self.registry
        )
        
        # Lite processing metrics
        self.lite_processing_total = Counter(
            'insane_ai_lite_processing_total',
            'Lite processing requests',
            ['status', 'channels'],
            registry=self.registry
        )
        
        self.lite_confidence = Histogram(
            'insane_ai_lite_confidence',
            'Lite processing confidence scores',
            buckets=[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
            registry=self.registry
        )
        
        # Deep processing metrics
        self.deep_sessions_total = Counter(
            'insane_ai_deep_sessions_total',
            'Deep processing sessions',
            ['tier', 'status', 'bypass_reason'],
            registry=self.registry
        )
        
        self.deep_session_events = Histogram(
            'insane_ai_deep_session_events',
            'Events per deep processing session',
            buckets=[1, 2, 5, 10, 20, 50, 100],
            registry=self.registry
        )
        
        self.deep_findings = Counter(
            'insane_ai_deep_findings_total',
            'Deep processing findings',
            ['finding_type', 'confidence_level'],
            registry=self.registry
        )
        
        # Scheduler metrics
        self.scheduler_rounds_total = Counter(
            'insane_ai_scheduler_rounds_total',
            'Scheduler rounds executed',
            ['status'],
            registry=self.registry
        )
        
        self.scheduled_events = Counter(
            'insane_ai_scheduled_events_total',
            'Events scheduled for processing',
            ['tier'],
            registry=self.registry
        )
        
        self.token_bucket_tokens = Gauge(
            'insane_ai_token_bucket_tokens',
            'Current token bucket levels',
            ['tier'],
            registry=self.registry
        )
        
        self.autothrottle_activations = Counter(
            'insane_ai_autothrottle_activations_total',
            'Autothrottle activations',
            registry=self.registry
        )
    
    def _init_storage_metrics(self):
        """Initialize storage-related metrics"""
        # Redis metrics
        self.redis_operations_total = Counter(
            'insane_ai_redis_operations_total',
            'Redis operations',
            ['operation', 'status'],
            registry=self.registry
        )
        
        self.redis_operation_duration = Histogram(
            'insane_ai_redis_operation_duration_seconds',
            'Redis operation duration',
            ['operation'],
            buckets=[0.001, 0.005, 0.01, 0.05, 0.1, 0.25, 0.5],
            registry=self.registry
        )
        
        self.redis_connections = Gauge(
            'insane_ai_redis_connections',
            'Redis connection pool metrics',
            ['status'],
            registry=self.registry
        )
        
        # Candidate store metrics
        self.candidates_added = Counter(
            'insane_ai_candidates_added_total',
            'Candidates added to store',
            ['tier', 'priority'],
            registry=self.registry
        )
        
        self.candidates_removed = Counter(
            'insane_ai_candidates_removed_total',
            'Candidates removed from store',
            ['reason'],
            registry=self.registry
        )
        
        self.candidate_store_size = Gauge(
            'insane_ai_candidate_store_size',
            'Current candidate store size',
            ['tier'],
            registry=self.registry
        )
        
        # Queue metrics
        self.queue_size = Gauge(
            'insane_ai_queue_size',
            'Queue sizes',
            ['queue_name', 'priority'],
            registry=self.registry
        )
        
        self.queue_processing_time = Histogram(
            'insane_ai_queue_processing_time_seconds',
            'Time items spend in queue before processing',
            ['queue_name'],
            buckets=[1, 5, 15, 30, 60, 300, 900],
            registry=self.registry
        )
    
    def _init_business_metrics(self):
        """Initialize business-specific metrics"""
        # User metrics
        self.active_users = Gauge(
            'insane_ai_active_users',
            'Currently active users',
            ['tier'],
            registry=self.registry
        )
        
        self.active_homes = Gauge(
            'insane_ai_active_homes',
            'Currently active homes',
            ['tier'],
            registry=self.registry
        )
        
        # Detection metrics
        self.detections_total = Counter(
            'insane_ai_detections_total',
            'Total detections by type',
            ['detection_type', 'location', 'confidence_level'],
            registry=self.registry
        )
        
        self.risk_scores = Histogram(
            'insane_ai_risk_scores',
            'Risk score distribution',
            buckets=[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
            registry=self.registry
        )
        
        self.life_safety_events = Counter(
            'insane_ai_life_safety_events_total',
            'Life safety events detected',
            ['event_type', 'location'],
            registry=self.registry
        )
        
        # Performance SLIs
        self.lite_processing_sli = Histogram(
            'insane_ai_lite_processing_sli_seconds',
            'Lite processing SLI (target: 95% < 2s)',
            buckets=[0.5, 1.0, 2.0, 5.0, 10.0],
            registry=self.registry
        )
        
        self.deep_processing_sli = Histogram(
            'insane_ai_deep_processing_sli_seconds',
            'Deep processing SLI (target: 95% < 30s)',
            buckets=[5.0, 10.0, 20.0, 30.0, 60.0, 120.0],
            registry=self.registry
        )
        
        self.api_availability_sli = Gauge(
            'insane_ai_api_availability_sli',
            'API availability SLI (target: 99.9%)',
            registry=self.registry
        )
    
    def _init_system_metrics(self):
        """Initialize system-level metrics"""
        if not self.config.enable_system_metrics:
            return
            
        # System resources
        self.cpu_usage = Gauge(
            'insane_ai_cpu_usage_percent',
            'CPU usage percentage',
            ['core'],
            registry=self.registry
        )
        
        self.memory_usage = Gauge(
            'insane_ai_memory_usage_bytes',
            'Memory usage',
            ['type'],
            registry=self.registry
        )
        
        self.disk_usage = Gauge(
            'insane_ai_disk_usage_bytes',
            'Disk usage',
            ['mount_point', 'type'],
            registry=self.registry
        )
        
        self.network_io = Counter(
            'insane_ai_network_io_bytes_total',
            'Network I/O bytes',
            ['interface', 'direction'],
            registry=self.registry
        )
        
        # Process metrics
        self.process_count = Gauge(
            'insane_ai_process_count',
            'Number of processes by service',
            ['service'],
            registry=self.registry
        )
        
        self.open_file_descriptors = Gauge(
            'insane_ai_open_file_descriptors',
            'Open file descriptors',
            registry=self.registry
        )
    
    # API Methods
    def record_http_request(
        self, 
        method: str, 
        endpoint: str, 
        status_code: int, 
        duration: float,
        tier: str = "unknown",
        request_size: int = 0,
        response_size: int = 0
    ):
        """Record HTTP request metrics"""
        self.http_requests_total.labels(
            method=method, 
            endpoint=endpoint, 
            status_code=str(status_code),
            tier=tier
        ).inc()
        
        self.http_request_duration.labels(
            method=method, 
            endpoint=endpoint,
            tier=tier
        ).observe(duration)
        
        if request_size > 0:
            self.http_request_size.labels(
                method=method, 
                endpoint=endpoint
            ).observe(request_size)
        
        if response_size > 0:
            self.http_response_size.labels(
                method=method, 
                endpoint=endpoint,
                status_code=str(status_code)
            ).observe(response_size)
    
    def record_auth_request(self, method: str, success: bool):
        """Record authentication metrics"""
        status = "success" if success else "failure"
        self.auth_requests_total.labels(method=method, status=status).inc()
    
    def record_token_validation(self, success: bool):
        """Record JWT token validation"""
        status = "valid" if success else "invalid"
        self.auth_token_validations.labels(status=status).inc()
    
    def record_rate_limit_hit(self, client_type: str, tier: str):
        """Record rate limit violation"""
        self.rate_limit_hits.labels(client_type=client_type, tier=tier).inc()
    
    # Processing Methods
    def record_event_created(self, location: str, mode: str, tier: str):
        """Record event creation"""
        self.events_created_total.labels(
            location=location, 
            mode=mode, 
            tier=tier
        ).inc()
    
    def record_event_processed(
        self, 
        processor_type: str, 
        tier: str, 
        success: bool, 
        duration: float
    ):
        """Record event processing"""
        status = "success" if success else "failure"
        self.events_processed_total.labels(
            processor_type=processor_type,
            tier=tier,
            status=status
        ).inc()
        
        self.processing_duration.labels(
            processor_type=processor_type,
            tier=tier
        ).observe(duration)
    
    def record_lite_processing(
        self, 
        success: bool, 
        channels: str, 
        confidence: float,
        duration: float
    ):
        """Record lite processing metrics"""
        status = "success" if success else "failure"
        self.lite_processing_total.labels(status=status, channels=channels).inc()
        self.lite_confidence.observe(confidence)
        self.lite_processing_sli.observe(duration)
    
    def record_deep_session(
        self, 
        tier: str, 
        success: bool, 
        event_count: int,
        duration: float,
        bypass_reason: str = None
    ):
        """Record deep processing session"""
        status = "success" if success else "failure"
        self.deep_sessions_total.labels(
            tier=tier, 
            status=status,
            bypass_reason=bypass_reason or "none"
        ).inc()
        
        self.deep_session_events.observe(event_count)
        self.deep_processing_sli.observe(duration)
    
    def record_detection(
        self, 
        detection_type: str, 
        location: str, 
        confidence: float,
        risk_score: float
    ):
        """Record detection and risk metrics"""
        confidence_level = "high" if confidence > 0.7 else "medium" if confidence > 0.4 else "low"
        
        self.detections_total.labels(
            detection_type=detection_type,
            location=location,
            confidence_level=confidence_level
        ).inc()
        
        self.risk_scores.observe(risk_score)
    
    def record_life_safety_event(self, event_type: str, location: str):
        """Record life safety event"""
        self.life_safety_events.labels(
            event_type=event_type,
            location=location
        ).inc()
    
    # Storage Methods
    def record_redis_operation(self, operation: str, success: bool, duration: float):
        """Record Redis operation metrics"""
        status = "success" if success else "failure"
        self.redis_operations_total.labels(operation=operation, status=status).inc()
        self.redis_operation_duration.labels(operation=operation).observe(duration)
    
    def update_queue_size(self, queue_name: str, size: int, priority: str = "normal"):
        """Update queue size gauge"""
        self.queue_size.labels(queue_name=queue_name, priority=priority).set(size)
    
    def record_queue_processing_time(self, queue_name: str, wait_time: float):
        """Record time item spent in queue"""
        self.queue_processing_time.labels(queue_name=queue_name).observe(wait_time)
    
    def update_candidate_store_size(self, tier: str, size: int):
        """Update candidate store size"""
        self.candidate_store_size.labels(tier=tier).set(size)
    
    # Scheduler Methods
    def record_scheduler_round(self, success: bool, scheduled_count: int):
        """Record scheduler round metrics"""
        status = "success" if success else "failure"
        self.scheduler_rounds_total.labels(status=status).inc()
        
        if success:
            # Record per-tier scheduling (would need tier breakdown)
            for tier in ["standard", "premium", "enterprise"]:
                # This would come from the actual scheduler stats
                count = scheduled_count // 3  # Simplified
                if count > 0:
                    self.scheduled_events.labels(tier=tier).inc(count)
    
    def update_token_bucket(self, tier: str, tokens: float):
        """Update token bucket levels"""
        self.token_bucket_tokens.labels(tier=tier).set(tokens)
    
    def record_autothrottle_activation(self):
        """Record autothrottle activation"""
        self.autothrottle_activations.inc()
    
    # System Methods
    async def collect_system_metrics(self):
        """Collect system-level metrics"""
        if not self.config.enable_system_metrics:
            return
            
        try:
            # CPU metrics
            cpu_percentages = psutil.cpu_percent(percpu=True)
            for i, usage in enumerate(cpu_percentages):
                self.cpu_usage.labels(core=f"cpu{i}").set(usage)
            
            # Memory metrics
            memory = psutil.virtual_memory()
            self.memory_usage.labels(type="used").set(memory.used)
            self.memory_usage.labels(type="available").set(memory.available)
            self.memory_usage.labels(type="total").set(memory.total)
            
            # Disk metrics
            for partition in psutil.disk_partitions():
                try:
                    usage = psutil.disk_usage(partition.mountpoint)
                    mount_point = partition.mountpoint.replace('/', '_root') if partition.mountpoint == '/' else partition.mountpoint
                    
                    self.disk_usage.labels(mount_point=mount_point, type="used").set(usage.used)
                    self.disk_usage.labels(mount_point=mount_point, type="free").set(usage.free)
                    self.disk_usage.labels(mount_point=mount_point, type="total").set(usage.total)
                except PermissionError:
                    continue
            
            # Network I/O
            network_io = psutil.net_io_counters(pernic=True)
            for interface, stats in network_io.items():
                self.network_io.labels(interface=interface, direction="sent")._value._value = stats.bytes_sent
                self.network_io.labels(interface=interface, direction="recv")._value._value = stats.bytes_recv
            
            # Process metrics
            self.open_file_descriptors.set(len(psutil.Process().open_files()))
            
        except Exception as e:
            logger.error(f"Failed to collect system metrics: {e}")
    
    async def collect_business_metrics(self):
        """Collect business-specific metrics"""
        if not self.redis_client:
            return
            
        try:
            # Active users/homes (simplified - would come from actual data)
            # This would be implemented based on your session tracking
            
            # Calculate SLI availability
            # This would be based on your actual health checks and error rates
            
            pass
            
        except Exception as e:
            logger.error(f"Failed to collect business metrics: {e}")
    
    def start_metrics_collection(self):
        """Start background metrics collection"""
        self.running = True
        
        def system_metrics_worker():
            while self.running:
                try:
                    asyncio.run(self.collect_system_metrics())
                except Exception as e:
                    logger.error(f"System metrics collection failed: {e}")
                time.sleep(self.config.system_metrics_interval)
        
        def business_metrics_worker():
            while self.running:
                try:
                    asyncio.run(self.collect_business_metrics())
                except Exception as e:
                    logger.error(f"Business metrics collection failed: {e}")
                time.sleep(self.config.business_metrics_interval)
        
        if self.config.enable_system_metrics:
            Thread(target=system_metrics_worker, daemon=True).start()
        
        Thread(target=business_metrics_worker, daemon=True).start()
        
        logger.info("Started metrics collection threads")
    
    def stop_metrics_collection(self):
        """Stop background metrics collection"""
        self.running = False
        logger.info("Stopped metrics collection")
    
    def start_http_server(self):
        """Start Prometheus HTTP server"""
        try:
            start_http_server(self.config.metrics_port, registry=self.registry)
            logger.info(f"Prometheus metrics server started on port {self.config.metrics_port}")
        except Exception as e:
            logger.error(f"Failed to start metrics server: {e}")
            raise
    
    def push_metrics(self):
        """Push metrics to Prometheus Push Gateway"""
        if not self.config.push_gateway_url:
            return
            
        try:
            from prometheus_client import push_to_gateway
            push_to_gateway(
                self.config.push_gateway_url,
                job=self.config.job_name,
                registry=self.registry
            )
        except Exception as e:
            logger.error(f"Failed to push metrics: {e}")
    
    def get_metrics_text(self) -> str:
        """Get metrics in Prometheus text format"""
        return generate_latest(self.registry).decode()

# Alert rules and monitoring
ALERT_RULES = """
groups:
  - name: insane-ai-alerts
    rules:
      - alert: HighErrorRate
        expr: rate(insane_ai_http_requests_total{status_code=~"5.."}[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }}% over the last 5 minutes"
      
      - alert: HighLatency
        expr: histogram_quantile(0.99, rate(insane_ai_http_request_duration_seconds_bucket[5m])) > 2
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High API latency"
          description: "99th percentile latency is {{ $value }}s"
      
      - alert: ProcessingQueueBacklog
        expr: insane_ai_queue_size > 1000
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "Large queue backlog"
          description: "Queue {{ $labels.queue_name }} has {{ $value }} items"
      
      - alert: LifeSafetyProcessingDelay
        expr: insane_ai_deep_processing_sli_seconds{bypass_reason="life_safety_event"} > 2
        for: 30s
        labels:
          severity: critical
        annotations:
          summary: "Life safety event processing delayed"
          description: "Life safety event took {{ $value }}s to process"
      
      - alert: HighMemoryUsage
        expr: (insane_ai_memory_usage_bytes{type="used"} / insane_ai_memory_usage_bytes{type="total"}) > 0.9
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value }}%"
      
      - alert: RedisDown
        expr: up{job="redis"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Redis is down"
          description: "Redis instance is not responding"
      
      - alert: APIAvailabilityLow
        expr: insane_ai_api_availability_sli < 0.999
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "API availability below SLO"
          description: "API availability is {{ $value }}, below 99.9% SLO"
"""

async def main():
    """Test metrics collection"""
    # Configuration
    config = MetricsConfig(
        metrics_port=9090,
        enable_system_metrics=True,
        enable_detailed_metrics=True
    )
    
    # Initialize Redis
    redis_client = redis.from_url("redis://localhost:6379", decode_responses=True)
    
    try:
        # Initialize metrics
        metrics = InsaneAIMetrics(config, redis_client)
        
        print("=== Metrics Testing ===")
        
        # Test API metrics
        print("\n1. Recording API metrics...")
        metrics.record_http_request("GET", "/api/events", 200, 0.125, "premium", 1024, 2048)
        metrics.record_http_request("POST", "/api/events", 201, 0.256, "standard", 2048, 512)
        
        # Test processing metrics
        print("2. Recording processing metrics...")
        metrics.record_event_created("front_door", "security", "premium")
        metrics.record_lite_processing(True, "person,vehicle", 0.85, 1.2)
        metrics.record_deep_session("enterprise", True, 5, 15.3, "normal")
        
        # Test detection metrics
        print("3. Recording detection metrics...")
        metrics.record_detection("person", "front_door", 0.89, 0.65)
        metrics.record_life_safety_event("forced_entry", "back_door")
        
        # Collect system metrics
        print("4. Collecting system metrics...")
        await metrics.collect_system_metrics()
        
        # Generate metrics output
        print("5. Generating metrics...")
        metrics_text = metrics.get_metrics_text()
        
        # Show sample of metrics
        lines = metrics_text.split('\n')
        interesting_metrics = [line for line in lines if 'insane_ai' in line and not line.startswith('#')][:10]
        
        print("Sample metrics:")
        for metric in interesting_metrics:
            print(f"  {metric}")
        
        print(f"\nTotal metrics lines: {len(lines)}")
        print("Metrics collection test completed successfully!")
        
    except Exception as e:
        print(f"Metrics test failed: {e}")
        traceback.print_exc()
    
    finally:
        await redis_client.close()

if __name__ == "__main__":
    asyncio.run(main())

#!/usr/bin/env python3
"""
Load Testing and Chaos Engineering Framework
Synthetic event generation, SLO validation, and system resilience testing
"""

import logging
import asyncio
import random
import time
import json
import uuid
from datetime import datetime, timedelta, timezone
from typing import Dict, List, Any, Optional, Callable, Tuple
from dataclasses import dataclass, asdict
import redis.asyncio as redis
import httpx
import statistics
import threading
from concurrent.futures import ThreadPoolExecutor, as_completed
import psutil
import traceback

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class LoadTestConfig:
    """Load testing configuration"""
    # Test duration and rates
    duration_minutes: int = 10
    max_events_per_second: int = 100
    ramp_up_seconds: int = 60
    
    # API endpoints
    api_base_url: str = "http://localhost:8000"
    api_key: str = "test-api-key"
    
    # Event generation
    locations: List[str] = None
    modes: List[str] = None
    tiers: List[str] = None
    
    # SLO thresholds
    api_p95_latency_ms: int = 2000
    api_p99_latency_ms: int = 5000
    error_rate_threshold: float = 0.01  # 1%
    availability_threshold: float = 0.999  # 99.9%
    
    # Chaos engineering
    enable_chaos: bool = False
    chaos_failure_rate: float = 0.05  # 5%
    chaos_scenarios: List[str] = None
    
    def __post_init__(self):
        if self.locations is None:
            self.locations = ["front_door", "back_door", "driveway", "backyard", "living_room", "kitchen"]
        
        if self.modes is None:
            self.modes = ["security", "monitoring", "alert", "emergency"]
        
        if self.tiers is None:
            self.tiers = ["standard", "premium", "enterprise"]
        
        if self.chaos_scenarios is None:
            self.chaos_scenarios = [
                "redis_slow_query",
                "network_latency", 
                "memory_pressure",
                "cpu_spike",
                "disk_full",
                "service_timeout"
            ]

@dataclass
class TestEvent:
    """Synthetic test event"""
    event_id: str
    user_id: str
    home_id: str
    location: str
    mode: str
    tier: str
    timestamp: datetime
    image_url: str
    priority: int = 2
    
    def to_api_payload(self) -> Dict[str, Any]:
        """Convert to API payload format"""
        return {
            "event_id": self.event_id,
            "user_id": self.user_id,
            "home_id": self.home_id,
            "location": self.location,
            "mode": self.mode,
            "timestamp": self.timestamp.isoformat(),
            "image_url": self.image_url,
            "priority": self.priority,
            "metadata": {
                "test_event": True,
                "tier": self.tier,
                "generated_at": datetime.utcnow().isoformat()
            }
        }

@dataclass
class TestResult:
    """Individual test request result"""
    timestamp: datetime
    endpoint: str
    method: str
    status_code: int
    latency_ms: float
    success: bool
    error: Optional[str] = None
    payload_size: int = 0

@dataclass
class LoadTestResults:
    """Aggregate load test results"""
    test_name: str
    duration_seconds: float
    total_requests: int
    successful_requests: int
    failed_requests: int
    
    # Latency metrics
    avg_latency_ms: float
    p50_latency_ms: float
    p95_latency_ms: float
    p99_latency_ms: float
    max_latency_ms: float
    
    # Throughput
    requests_per_second: float
    
    # Error analysis
    error_rate: float
    error_breakdown: Dict[str, int]
    
    # SLO validation
    slo_violations: List[str]
    availability: float
    
    # System metrics
    peak_cpu_usage: float = 0.0
    peak_memory_usage: float = 0.0
    
    def passes_slos(self, config: LoadTestConfig) -> bool:
        """Check if results meet SLO requirements"""
        return (
            self.p95_latency_ms <= config.api_p95_latency_ms and
            self.p99_latency_ms <= config.api_p99_latency_ms and
            self.error_rate <= config.error_rate_threshold and
            self.availability >= config.availability_threshold
        )

class EventGenerator:
    """Generates synthetic events for testing"""
    
    def __init__(self, config: LoadTestConfig):
        self.config = config
        self.users = [f"test_user_{i:04d}" for i in range(1000)]
        self.homes = [f"test_home_{i:04d}" for i in range(500)]
        
    def generate_event(self) -> TestEvent:
        """Generate a single synthetic event"""
        event_id = str(uuid.uuid4())
        user_id = random.choice(self.users)
        home_id = random.choice(self.homes)
        location = random.choice(self.config.locations)
        mode = random.choice(self.config.modes)
        tier = random.choice(self.config.tiers)
        
        # Add some realistic patterns
        now = datetime.utcnow()
        
        # Higher activity during "daytime" hours
        if 6 <= now.hour <= 22:
            priority = random.choices([1, 2, 3, 4], weights=[10, 40, 30, 20])[0]
        else:
            priority = random.choices([1, 2, 3, 4], weights=[5, 20, 40, 35])[0]  # More high priority at night
        
        return TestEvent(
            event_id=event_id,
            user_id=user_id,
            home_id=home_id,
            location=location,
            mode=mode,
            tier=tier,
            timestamp=now,
            image_url=f"https://test.insane.ai/events/{event_id}.jpg",
            priority=priority
        )
    
    def generate_life_safety_event(self) -> TestEvent:
        """Generate a life safety event for testing"""
        event = self.generate_event()
        event.location = random.choice(["front_door", "back_door"])
        event.mode = "emergency"
        event.priority = 4
        
        # Add life safety indicators to the image URL
        safety_indicators = ["smoke", "co", "glassbreak", "forced_entry"]
        indicator = random.choice(safety_indicators)
        event.image_url = f"https://test.insane.ai/events/life_safety_{indicator}_{event.event_id}.jpg"
        
        return event

class ChaosEngineer:
    """Chaos engineering implementation"""
    
    def __init__(self, config: LoadTestConfig):
        self.config = config
        self.active_chaos = []
        
    def inject_chaos(self, scenario: str) -> Callable:
        """Inject a chaos scenario"""
        if not self.config.enable_chaos:
            return lambda: None
            
        logger.info(f"Injecting chaos scenario: {scenario}")
        
        if scenario == "redis_slow_query":
            return self._redis_slow_query_chaos
        elif scenario == "network_latency":
            return self._network_latency_chaos
        elif scenario == "memory_pressure":
            return self._memory_pressure_chaos
        elif scenario == "cpu_spike":
            return self._cpu_spike_chaos
        elif scenario == "disk_full":
            return self._disk_full_chaos
        elif scenario == "service_timeout":
            return self._service_timeout_chaos
        else:
            return lambda: None
    
    def _redis_slow_query_chaos(self):
        """Simulate Redis slow queries"""
        # This would integrate with Redis to inject artificial delays
        time.sleep(random.uniform(0.1, 0.5))
    
    def _network_latency_chaos(self):
        """Simulate network latency"""
        time.sleep(random.uniform(0.05, 0.2))
    
    def _memory_pressure_chaos(self):
        """Simulate memory pressure"""
        # Create temporary memory pressure
        if random.random() < 0.1:  # 10% chance
            garbage = [b'x' * 1024 * 1024 for _ in range(50)]  # 50MB
            time.sleep(1)
            del garbage
    
    def _cpu_spike_chaos(self):
        """Simulate CPU spikes"""
        if random.random() < 0.05:  # 5% chance
            start = time.time()
            while time.time() - start < 0.1:  # 100ms CPU burn
                _ = sum(range(10000))
    
    def _disk_full_chaos(self):
        """Simulate disk space issues"""
        # This would be implemented with actual disk operations
        time.sleep(random.uniform(0.01, 0.05))
    
    def _service_timeout_chaos(self):
        """Simulate service timeouts"""
        if random.random() < 0.02:  # 2% chance
            time.sleep(random.uniform(1, 3))

class LoadTester:
    """Main load testing orchestrator"""
    
    def __init__(self, config: LoadTestConfig):
        self.config = config
        self.generator = EventGenerator(config)
        self.chaos = ChaosEngineer(config)
        self.results: List[TestResult] = []
        self.running = False
        
    async def run_load_test(self, test_name: str = "default") -> LoadTestResults:
        """Execute a complete load test"""
        logger.info(f"Starting load test: {test_name}")
        self.results = []
        self.running = True
        
        start_time = time.time()
        end_time = start_time + (self.config.duration_minutes * 60)
        
        # Start system monitoring
        monitoring_task = asyncio.create_task(self._monitor_system_resources())
        
        # Start chaos engineering if enabled
        chaos_task = None
        if self.config.enable_chaos:
            chaos_task = asyncio.create_task(self._run_chaos_scenarios())
        
        try:
            # Execute load test phases
            await self._execute_load_phases(start_time, end_time)
            
        finally:
            self.running = False
            monitoring_task.cancel()
            if chaos_task:
                chaos_task.cancel()
        
        # Analyze results
        duration = time.time() - start_time
        return self._analyze_results(test_name, duration)
    
    async def _execute_load_phases(self, start_time: float, end_time: float):
        """Execute the load test with ramp-up and sustained phases"""
        current_time = start_time
        ramp_up_end = start_time + self.config.ramp_up_seconds
        
        # Create HTTP client
        async with httpx.AsyncClient(timeout=30.0) as client:
            
            while current_time < end_time and self.running:
                # Calculate current load level
                if current_time <= ramp_up_end:
                    # Ramp-up phase
                    progress = (current_time - start_time) / self.config.ramp_up_seconds
                    current_rate = int(self.config.max_events_per_second * progress)
                else:
                    # Sustained load phase
                    current_rate = self.config.max_events_per_second
                
                if current_rate > 0:
                    # Generate batch of requests
                    tasks = []
                    batch_size = min(current_rate, 50)  # Limit concurrent requests
                    
                    for _ in range(batch_size):
                        event = self.generator.generate_event()
                        
                        # Occasionally generate life safety events
                        if random.random() < 0.02:  # 2% chance
                            event = self.generator.generate_life_safety_event()
                        
                        task = asyncio.create_task(
                            self._execute_request(client, event)
                        )
                        tasks.append(task)
                    
                    # Wait for batch completion
                    await asyncio.gather(*tasks, return_exceptions=True)
                
                # Sleep until next second
                current_time = time.time()
                next_second = int(current_time) + 1
                sleep_time = max(0, next_second - current_time)
                await asyncio.sleep(sleep_time)
                current_time = time.time()
    
    async def _execute_request(self, client: httpx.AsyncClient, event: TestEvent):
        """Execute a single API request"""
        request_start = time.time()
        
        try:
            # Apply chaos engineering
            if self.config.enable_chaos and random.random() < self.config.chaos_failure_rate:
                scenario = random.choice(self.config.chaos_scenarios)
                chaos_func = self.chaos.inject_chaos(scenario)
                chaos_func()
            
            # Make API request
            headers = {
                "Content-Type": "application/json",
                "Authorization": f"Bearer {self.config.api_key}"
            }
            
            payload = event.to_api_payload()
            
            # Test different endpoints
            endpoints = [
                ("/api/events", "POST", payload),
                (f"/api/events/{event.event_id}", "GET", None),
                ("/api/health", "GET", None),
                (f"/api/events/{event.event_id}/lite-results", "POST", {
                    "channels": {"person": True, "vehicle": False},
                    "confidence": random.uniform(0.5, 1.0),
                    "explainer": "Test lite processing result"
                })
            ]
            
            # Weight towards event creation
            endpoint_weights = [70, 15, 5, 10]
            endpoint_data = random.choices(endpoints, weights=endpoint_weights)[0]
            url, method, request_payload = endpoint_data
            
            full_url = f"{self.config.api_base_url}{url}"
            
            if method == "POST" and request_payload:
                response = await client.post(full_url, json=request_payload, headers=headers)
            else:
                response = await client.get(full_url, headers=headers)
            
            latency_ms = (time.time() - request_start) * 1000
            
            result = TestResult(
                timestamp=datetime.utcnow(),
                endpoint=url,
                method=method,
                status_code=response.status_code,
                latency_ms=latency_ms,
                success=200 <= response.status_code < 400,
                payload_size=len(response.content) if response.content else 0
            )
            
            self.results.append(result)
            
        except Exception as e:
            latency_ms = (time.time() - request_start) * 1000
            
            result = TestResult(
                timestamp=datetime.utcnow(),
                endpoint=url if 'url' in locals() else "unknown",
                method=method if 'method' in locals() else "unknown",
                status_code=0,
                latency_ms=latency_ms,
                success=False,
                error=str(e)
            )
            
            self.results.append(result)
    
    async def _monitor_system_resources(self):
        """Monitor system resources during test"""
        peak_cpu = 0.0
        peak_memory = 0.0
        
        while self.running:
            try:
                cpu_percent = psutil.cpu_percent(interval=1)
                memory = psutil.virtual_memory()
                memory_percent = memory.percent
                
                peak_cpu = max(peak_cpu, cpu_percent)
                peak_memory = max(peak_memory, memory_percent)
                
                self.peak_cpu_usage = peak_cpu
                self.peak_memory_usage = peak_memory
                
            except Exception as e:
                logger.error(f"System monitoring error: {e}")
            
            await asyncio.sleep(5)
    
    async def _run_chaos_scenarios(self):
        """Run chaos engineering scenarios during test"""
        while self.running:
            try:
                await asyncio.sleep(random.uniform(10, 30))  # Random intervals
                
                if random.random() < 0.3:  # 30% chance every interval
                    scenario = random.choice(self.config.chaos_scenarios)
                    chaos_func = self.chaos.inject_chaos(scenario)
                    chaos_func()
                    
            except Exception as e:
                logger.error(f"Chaos scenario error: {e}")
    
    def _analyze_results(self, test_name: str, duration: float) -> LoadTestResults:
        """Analyze test results and generate report"""
        if not self.results:
            logger.error("No results to analyze")
            return LoadTestResults(
                test_name=test_name,
                duration_seconds=duration,
                total_requests=0,
                successful_requests=0,
                failed_requests=0,
                avg_latency_ms=0,
                p50_latency_ms=0,
                p95_latency_ms=0,
                p99_latency_ms=0,
                max_latency_ms=0,
                requests_per_second=0,
                error_rate=1.0,
                error_breakdown={},
                slo_violations=[],
                availability=0.0
            )
        
        # Basic counts
        total_requests = len(self.results)
        successful_requests = sum(1 for r in self.results if r.success)
        failed_requests = total_requests - successful_requests
        
        # Latency analysis
        latencies = [r.latency_ms for r in self.results]
        latencies.sort()
        
        avg_latency = statistics.mean(latencies)
        p50_latency = statistics.median(latencies)
        p95_latency = latencies[int(0.95 * len(latencies))] if latencies else 0
        p99_latency = latencies[int(0.99 * len(latencies))] if latencies else 0
        max_latency = max(latencies) if latencies else 0
        
        # Error analysis
        error_rate = failed_requests / total_requests if total_requests > 0 else 1.0
        availability = successful_requests / total_requests if total_requests > 0 else 0.0
        
        error_breakdown = {}
        for result in self.results:
            if not result.success:
                error_key = f"{result.status_code}:{result.error or 'unknown'}"
                error_breakdown[error_key] = error_breakdown.get(error_key, 0) + 1
        
        # SLO validation
        slo_violations = []
        if p95_latency > self.config.api_p95_latency_ms:
            slo_violations.append(f"P95 latency {p95_latency:.1f}ms > {self.config.api_p95_latency_ms}ms")
        
        if p99_latency > self.config.api_p99_latency_ms:
            slo_violations.append(f"P99 latency {p99_latency:.1f}ms > {self.config.api_p99_latency_ms}ms")
        
        if error_rate > self.config.error_rate_threshold:
            slo_violations.append(f"Error rate {error_rate:.3f} > {self.config.error_rate_threshold}")
        
        if availability < self.config.availability_threshold:
            slo_violations.append(f"Availability {availability:.3f} < {self.config.availability_threshold}")
        
        return LoadTestResults(
            test_name=test_name,
            duration_seconds=duration,
            total_requests=total_requests,
            successful_requests=successful_requests,
            failed_requests=failed_requests,
            avg_latency_ms=avg_latency,
            p50_latency_ms=p50_latency,
            p95_latency_ms=p95_latency,
            p99_latency_ms=p99_latency,
            max_latency_ms=max_latency,
            requests_per_second=total_requests / duration if duration > 0 else 0,
            error_rate=error_rate,
            error_breakdown=error_breakdown,
            slo_violations=slo_violations,
            availability=availability,
            peak_cpu_usage=getattr(self, 'peak_cpu_usage', 0.0),
            peak_memory_usage=getattr(self, 'peak_memory_usage', 0.0)
        )

class TestSuite:
    """Test suite with multiple load test scenarios"""
    
    def __init__(self, base_config: LoadTestConfig):
        self.base_config = base_config
        
    async def run_smoke_test(self) -> LoadTestResults:
        """Quick smoke test to verify basic functionality"""
        config = LoadTestConfig(
            duration_minutes=2,
            max_events_per_second=10,
            ramp_up_seconds=30,
            **{k: v for k, v in asdict(self.base_config).items() 
               if k not in ['duration_minutes', 'max_events_per_second', 'ramp_up_seconds']}
        )
        
        tester = LoadTester(config)
        return await tester.run_load_test("smoke_test")
    
    async def run_capacity_test(self) -> LoadTestResults:
        """Test system capacity limits"""
        config = LoadTestConfig(
            duration_minutes=15,
            max_events_per_second=200,
            ramp_up_seconds=300,  # 5 minute ramp up
            **{k: v for k, v in asdict(self.base_config).items() 
               if k not in ['duration_minutes', 'max_events_per_second', 'ramp_up_seconds']}
        )
        
        tester = LoadTester(config)
        return await tester.run_load_test("capacity_test")
    
    async def run_stress_test(self) -> LoadTestResults:
        """Stress test beyond normal capacity"""
        config = LoadTestConfig(
            duration_minutes=10,
            max_events_per_second=500,
            ramp_up_seconds=120,
            enable_chaos=True,
            **{k: v for k, v in asdict(self.base_config).items() 
               if k not in ['duration_minutes', 'max_events_per_second', 'ramp_up_seconds', 'enable_chaos']}
        )
        
        tester = LoadTester(config)
        return await tester.run_load_test("stress_test")
    
    async def run_endurance_test(self) -> LoadTestResults:
        """Long-running endurance test"""
        config = LoadTestConfig(
            duration_minutes=60,  # 1 hour
            max_events_per_second=50,
            ramp_up_seconds=300,
            **{k: v for k, v in asdict(self.base_config).items() 
               if k not in ['duration_minutes', 'max_events_per_second', 'ramp_up_seconds']}
        )
        
        tester = LoadTester(config)
        return await tester.run_load_test("endurance_test")
    
    async def run_chaos_test(self) -> LoadTestResults:
        """Chaos engineering focused test"""
        config = LoadTestConfig(
            duration_minutes=10,
            max_events_per_second=100,
            ramp_up_seconds=60,
            enable_chaos=True,
            chaos_failure_rate=0.15,  # 15% chaos injection
            **{k: v for k, v in asdict(self.base_config).items() 
               if k not in ['duration_minutes', 'max_events_per_second', 'ramp_up_seconds', 'enable_chaos', 'chaos_failure_rate']}
        )
        
        tester = LoadTester(config)
        return await tester.run_load_test("chaos_test")

def generate_test_report(results: List[LoadTestResults], config: LoadTestConfig) -> str:
    """Generate comprehensive test report"""
    report_lines = [
        "=" * 80,
        "INSANE AI SECURITY - LOAD TEST REPORT",
        "=" * 80,
        f"Generated: {datetime.utcnow().isoformat()}Z",
        f"Test Duration: {sum(r.duration_seconds for r in results):.1f} seconds",
        ""
    ]
    
    # Overall summary
    total_requests = sum(r.total_requests for r in results)
    total_successful = sum(r.successful_requests for r in results)
    overall_error_rate = (total_requests - total_successful) / total_requests if total_requests > 0 else 0
    
    report_lines.extend([
        "OVERALL SUMMARY",
        "-" * 40,
        f"Total Requests: {total_requests:,}",
        f"Successful Requests: {total_successful:,}",
        f"Failed Requests: {total_requests - total_successful:,}",
        f"Overall Error Rate: {overall_error_rate:.3%}",
        ""
    ])
    
    # Individual test results
    for result in results:
        slo_status = "✅ PASS" if result.passes_slos(config) else "❌ FAIL"
        
        report_lines.extend([
            f"TEST: {result.test_name.upper()} {slo_status}",
            "-" * 40,
            f"Duration: {result.duration_seconds:.1f}s",
            f"Total Requests: {result.total_requests:,}",
            f"Requests/sec: {result.requests_per_second:.1f}",
            f"Success Rate: {result.availability:.3%}",
            f"Error Rate: {result.error_rate:.3%}",
            "",
            "Latency Metrics:",
            f"  Average: {result.avg_latency_ms:.1f}ms",
            f"  P50: {result.p50_latency_ms:.1f}ms",
            f"  P95: {result.p95_latency_ms:.1f}ms",
            f"  P99: {result.p99_latency_ms:.1f}ms",
            f"  Max: {result.max_latency_ms:.1f}ms",
            "",
            "System Resources:",
            f"  Peak CPU: {result.peak_cpu_usage:.1f}%",
            f"  Peak Memory: {result.peak_memory_usage:.1f}%",
            ""
        ])
        
        if result.slo_violations:
            report_lines.extend([
                "SLO Violations:",
                *[f"  - {violation}" for violation in result.slo_violations],
                ""
            ])
        
        if result.error_breakdown:
            report_lines.extend([
                "Error Breakdown:",
                *[f"  {error}: {count}" for error, count in result.error_breakdown.items()],
                ""
            ])
        
        report_lines.append("")
    
    # SLO compliance summary
    passing_tests = sum(1 for r in results if r.passes_slos(config))
    total_tests = len(results)
    
    report_lines.extend([
        "SLO COMPLIANCE SUMMARY",
        "-" * 40,
        f"Passing Tests: {passing_tests}/{total_tests}",
        f"Overall Compliance: {passing_tests/total_tests:.1%}",
        "",
        "SLO Thresholds:",
        f"  P95 Latency: < {config.api_p95_latency_ms}ms",
        f"  P99 Latency: < {config.api_p99_latency_ms}ms", 
        f"  Error Rate: < {config.error_rate_threshold:.1%}",
        f"  Availability: > {config.availability_threshold:.1%}",
        "",
        "=" * 80
    ])
    
    return "\n".join(report_lines)

async def main():
    """Run comprehensive load testing suite"""
    # Configuration
    config = LoadTestConfig(
        api_base_url="http://localhost:8000",
        duration_minutes=1,  # Short test for demo
        max_events_per_second=20,
        api_p95_latency_ms=2000,
        api_p99_latency_ms=5000,
        error_rate_threshold=0.05,
        availability_threshold=0.95
    )
    
    print("=== Load Testing Framework Demo ===")
    
    try:
        # Initialize test suite
        suite = TestSuite(config)
        
        # Run tests
        results = []
        
        print("\n1. Running smoke test...")
        smoke_result = await suite.run_smoke_test()
        results.append(smoke_result)
        print(f"   Completed: {smoke_result.total_requests} requests, "
              f"{smoke_result.error_rate:.2%} error rate")
        
        # For demo, we'll just run smoke test
        # In production, you'd run all test types:
        # results.append(await suite.run_capacity_test())
        # results.append(await suite.run_stress_test())
        # results.append(await suite.run_chaos_test())
        
        # Generate report
        print("\n2. Generating test report...")
        report = generate_test_report(results, config)
        
        # Save report
        timestamp = datetime.utcnow().strftime("%Y%m%d_%H%M%S")
        report_file = f"load_test_report_{timestamp}.txt"
        
        with open(report_file, 'w') as f:
            f.write(report)
        
        print(f"3. Report saved to {report_file}")
        print("\nSample report preview:")
        print("\n".join(report.split('\n')[:25]))  # Show first 25 lines
        
        # Validate SLO compliance
        all_passed = all(r.passes_slos(config) for r in results)
        print(f"\n4. SLO Compliance: {'✅ PASS' if all_passed else '❌ FAIL'}")
        
    except Exception as e:
        print(f"Load test failed: {e}")
        traceback.print_exc()

if __name__ == "__main__":
    asyncio.run(main())

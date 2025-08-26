#!/usr/bin/env python3
"""
Scheduler Service
Top-K event selection with token bucket rate limiting
Coordinates between Candidate Store and Deep Worker queue
"""

import asyncio
import logging
import json
import time
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional, Set
import redis.asyncio as redis
from dataclasses import dataclass, asdict
from candidate_store import CandidateStore, EventCandidate, ProcessingTier, Priority

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class TokenBucket:
    """Token bucket for rate limiting"""
    capacity: int
    tokens: float
    refill_rate: float  # tokens per second
    last_refill: float
    
    def consume(self, tokens_needed: int = 1) -> bool:
        """Try to consume tokens, return True if successful"""
        now = time.time()
        
        # Refill tokens
        time_passed = now - self.last_refill
        self.tokens = min(self.capacity, self.tokens + (time_passed * self.refill_rate))
        self.last_refill = now
        
        if self.tokens >= tokens_needed:
            self.tokens -= tokens_needed
            return True
        
        return False
    
    def time_until_tokens(self, tokens_needed: int = 1) -> float:
        """Time in seconds until enough tokens are available"""
        if self.tokens >= tokens_needed:
            return 0.0
        
        tokens_deficit = tokens_needed - self.tokens
        return tokens_deficit / self.refill_rate

@dataclass
class SchedulingPolicy:
    """Scheduling policy configuration"""
    # Rate limits per tier (events per minute)
    tier_limits: Dict[ProcessingTier, int]
    
    # Batch sizes
    max_batch_size: int = 10
    min_batch_size: int = 1
    
    # Selection criteria
    top_k_limit: int = 50
    processing_timeout: int = 300  # 5 minutes
    
    # Priority scheduling
    priority_boost: Dict[Priority, float] = None
    
    def __post_init__(self):
        if self.priority_boost is None:
            self.priority_boost = {
                Priority.LOW: 0.5,
                Priority.NORMAL: 1.0,
                Priority.HIGH: 2.0,
                Priority.CRITICAL: 5.0
            }

class ProcessingScheduler:
    """Intelligent event processing scheduler"""
    
    def __init__(
        self,
        candidate_store: CandidateStore,
        redis_url: str = "redis://localhost:6379"
    ):
        self.candidate_store = candidate_store
        self.redis_url = redis_url
        self.redis_client = None
        
        # Default scheduling policy with actual tier rates
        self.policy = SchedulingPolicy(
            tier_limits={
                ProcessingTier.LITE_ONLY: 0,           # No deep processing
                ProcessingTier.STANDARD: 2,            # Free: ~24/day = 2/hour = 0.033/min
                ProcessingTier.PREMIUM: 7,             # Guardian: ~100/day = 7/hour = 0.12/min  
                ProcessingTier.ENTERPRISE: 32          # Pro: ~768/day = 32/hour = 0.53/min
            }
        )
        
        # Autothrottle settings
        self.num_gpus = 1  # Configure based on your setup
        self.autothrottle_threshold = 150 * self.num_gpus
        self.throttle_reduction = 0.4  # Reduce by 40%
        self.min_best_effort_k = 5
        
        # Token buckets per tier
        self.token_buckets: Dict[ProcessingTier, TokenBucket] = {}
        self._initialize_token_buckets()
        
        # Tracking
        self.processing_jobs: Set[str] = set()  # Currently processing event IDs
        self.last_schedule_time = datetime.utcnow()
        
    def _initialize_token_buckets(self):
        """Initialize token buckets for each processing tier"""
        for tier, limit_per_minute in self.policy.tier_limits.items():
            if limit_per_minute > 0:
                self.token_buckets[tier] = TokenBucket(
                    capacity=limit_per_minute,
                    tokens=limit_per_minute,  # Start full
                    refill_rate=limit_per_minute / 60.0,  # Convert to per-second
                    last_refill=time.time()
                )
    
    async def initialize(self):
        """Initialize Redis connection"""
        self.redis_client = redis.from_url(self.redis_url, decode_responses=True)
        logger.info("Scheduler initialized")
    
    async def close(self):
        """Close Redis connection"""
        if self.redis_client:
            await self.redis_client.close()
    
    async def schedule_round(self) -> Dict[str, Any]:
        """Run one scheduling round - select and enqueue top candidates"""
        start_time = datetime.utcnow()
        stats = {
            "scheduled_events": 0,
            "tier_breakdown": {},
            "rate_limited": {},
            "processing_load": len(self.processing_jobs),
            "round_duration_ms": 0,
            "autothrottled": False,
            "backlog_size": 0
        }
        
        try:
            logger.info("Starting scheduling round...")
            
            # Check for autothrottle condition
            deep_backlog = await self._get_deep_processing_backlog()
            stats["backlog_size"] = deep_backlog
            
            if deep_backlog > self.autothrottle_threshold:
                await self._apply_autothrottle()
                stats["autothrottled"] = True
                logger.warning(f"Autothrottle activated: backlog {deep_backlog} > threshold {self.autothrottle_threshold}")
            
            # Get candidates for each tier
            for tier in ProcessingTier:
                if tier == ProcessingTier.LITE_ONLY:
                    continue  # Skip lite-only tier
                
                await self._schedule_tier(tier, stats)
            
            # Update last schedule time
            self.last_schedule_time = start_time
            
            # Calculate round duration
            end_time = datetime.utcnow()
            stats["round_duration_ms"] = int((end_time - start_time).total_seconds() * 1000)
            
            logger.info(f"Scheduling round completed: {stats}")
            
        except Exception as e:
            logger.error(f"Scheduling round failed: {e}")
            stats["error"] = str(e)
        
        return stats
    
    async def _schedule_tier(self, tier: ProcessingTier, stats: Dict[str, Any]):
        """Schedule events for a specific processing tier"""
        tier_name = tier.name
        stats["tier_breakdown"][tier_name] = {
            "candidates_found": 0,
            "scheduled": 0,
            "rate_limited": 0
        }
        
        try:
            # Check if we have rate limit capacity
            bucket = self.token_buckets.get(tier)
            if not bucket:
                logger.warning(f"No token bucket configured for tier {tier_name}")
                return
            
            # Get top candidates for this tier
            candidates = await self.candidate_store.get_candidates_by_tier(
                tier, self.policy.top_k_limit
            )
            
            stats["tier_breakdown"][tier_name]["candidates_found"] = len(candidates)
            
            if not candidates:
                logger.debug(f"No candidates found for tier {tier_name}")
                return
            
            # Try to schedule candidates within rate limits
            scheduled_count = 0
            rate_limited_count = 0
            
            for candidate in candidates:
                # Skip if already processing
                if candidate.event_id in self.processing_jobs:
                    continue
                
                # Check token bucket
                if not bucket.consume(1):
                    rate_limited_count += 1
                    continue
                
                # Schedule for processing
                success = await self._enqueue_for_processing(candidate, tier)
                if success:
                    scheduled_count += 1
                    self.processing_jobs.add(candidate.event_id)
                    
                    # Remove from candidate store (now processing)
                    await self.candidate_store.remove_candidate(
                        candidate.event_id, candidate.home_id
                    )
                
                # Check if we've scheduled enough for this round
                if scheduled_count >= self.policy.max_batch_size:
                    break
            
            stats["tier_breakdown"][tier_name]["scheduled"] = scheduled_count
            stats["tier_breakdown"][tier_name]["rate_limited"] = rate_limited_count
            stats["scheduled_events"] += scheduled_count
            
            if rate_limited_count > 0:
                wait_time = bucket.time_until_tokens(1)
                stats["rate_limited"][tier_name] = {
                    "count": rate_limited_count,
                    "next_available_seconds": wait_time
                }
                logger.info(f"Rate limited {rate_limited_count} events for tier {tier_name}, next available in {wait_time:.1f}s")
            
            logger.info(f"Tier {tier_name}: scheduled {scheduled_count}, rate limited {rate_limited_count}")
            
        except Exception as e:
            logger.error(f"Failed to schedule tier {tier_name}: {e}")
            stats["tier_breakdown"][tier_name]["error"] = str(e)
    
    async def _enqueue_for_processing(self, candidate: EventCandidate, tier: ProcessingTier) -> bool:
        """Enqueue candidate for deep processing"""
        try:
            # Create processing job
            job_data = {
                "event_id": candidate.event_id,
                "home_id": candidate.home_id,
                "user_id": candidate.user_id,
                "image_url": candidate.image_url,
                "location": candidate.location,
                "mode": candidate.mode,
                "processing_tier": tier.value,
                "priority": candidate.priority.value,
                "enqueued_at": datetime.utcnow().isoformat(),
                "lite_results": {
                    "channels": candidate.lite_channels,
                    "explainer": candidate.lite_explainer,
                    "confidence": candidate.lite_confidence
                } if candidate.lite_processed else None
            }
            
            # Add to deep processing queue
            queue_name = f"deep_processing_{tier.name.lower()}"
            await self.redis_client.lpush(queue_name, json.dumps(job_data))
            
            # Set processing timeout
            timeout_key = f"processing:{candidate.event_id}"
            await self.redis_client.setex(
                timeout_key, 
                self.policy.processing_timeout, 
                "scheduled"
            )
            
            logger.info(f"Enqueued {candidate.event_id} for {tier.name} processing")
            return True
            
        except Exception as e:
            logger.error(f"Failed to enqueue candidate: {e}")
            return False
    
    async def handle_processing_completion(self, event_id: str, success: bool = True):
        """Handle notification that processing completed"""
        try:
            # Remove from processing set
            self.processing_jobs.discard(event_id)
            
            # Remove processing timeout
            timeout_key = f"processing:{event_id}"
            await self.redis_client.delete(timeout_key)
            
            logger.info(f"Processing completed for {event_id}, success: {success}")
            
        except Exception as e:
            logger.error(f"Failed to handle completion for {event_id}: {e}")
    
    async def cleanup_stale_jobs(self):
        """Clean up jobs that have exceeded processing timeout"""
        try:
            timeout_keys = await self.redis_client.keys("processing:*")
            
            cleaned_count = 0
            for timeout_key in timeout_keys:
                # Check if key still exists (it auto-expires)
                exists = await self.redis_client.exists(timeout_key)
                if not exists:
                    # Extract event_id and clean up
                    event_id = timeout_key.split(":")[1]
                    self.processing_jobs.discard(event_id)
                    cleaned_count += 1
            
            if cleaned_count > 0:
                logger.info(f"Cleaned up {cleaned_count} stale processing jobs")
            
        except Exception as e:
            logger.error(f"Failed to cleanup stale jobs: {e}")
    
    async def get_queue_stats(self) -> Dict[str, Any]:
        """Get processing queue statistics"""
        try:
            stats = {
                "queues": {},
                "processing_jobs": len(self.processing_jobs),
                "token_buckets": {}
            }
            
            # Get queue lengths
            for tier in ProcessingTier:
                if tier == ProcessingTier.LITE_ONLY:
                    continue
                    
                queue_name = f"deep_processing_{tier.name.lower()}"
                length = await self.redis_client.llen(queue_name)
                stats["queues"][queue_name] = length
            
            # Get token bucket status
            for tier, bucket in self.token_buckets.items():
                stats["token_buckets"][tier.name] = {
                    "tokens": bucket.tokens,
                    "capacity": bucket.capacity,
                    "refill_rate": bucket.refill_rate,
                    "utilization": 1.0 - (bucket.tokens / bucket.capacity)
                }
            
            return stats
            
        except Exception as e:
            logger.error(f"Failed to get queue stats: {e}")
            return {}
    
    async def force_schedule_event(self, event_id: str, tier: ProcessingTier) -> bool:
        """Force schedule a specific event (bypasses rate limiting)"""
        try:
            candidate = await self.candidate_store.get_candidate(event_id)
            if not candidate:
                logger.warning(f"Event {event_id} not found in candidate store")
                return False
            
            # Force enqueue
            success = await self._enqueue_for_processing(candidate, tier)
            if success:
                self.processing_jobs.add(event_id)
                await self.candidate_store.remove_candidate(event_id, candidate.home_id)
                logger.info(f"Force scheduled {event_id} for {tier.name} processing")
            
            return success
            
        except Exception as e:
            logger.error(f"Failed to force schedule {event_id}: {e}")
            return False
    
    async def _get_deep_processing_backlog(self) -> int:
        """Get total backlog across all deep processing queues"""
        try:
            total_backlog = 0
            
            for tier in ProcessingTier:
                if tier == ProcessingTier.LITE_ONLY:
                    continue
                    
                queue_name = f"deep_processing_{tier.name.lower()}"
                length = await self.redis_client.llen(queue_name)
                total_backlog += length
            
            # Also count currently processing jobs
            total_backlog += len(self.processing_jobs)
            
            return total_backlog
            
        except Exception as e:
            logger.error(f"Failed to get backlog: {e}")
            return 0
    
    async def _apply_autothrottle(self):
        """Reduce token bucket capacities to throttle processing"""
        try:
            for tier, bucket in self.token_buckets.items():
                # Reduce capacity but never below minimum best effort
                original_capacity = bucket.capacity
                new_capacity = max(
                    self.min_best_effort_k,
                    int(original_capacity * (1 - self.throttle_reduction))
                )
                
                if new_capacity < original_capacity:
                    bucket.capacity = new_capacity
                    bucket.tokens = min(bucket.tokens, new_capacity)
                    logger.info(f"Throttled {tier.name} capacity from {original_capacity} to {new_capacity}")
                    
        except Exception as e:
            logger.error(f"Failed to apply autothrottle: {e}")
    
    async def schedule_life_safety_event(self, candidate: EventCandidate) -> bool:
        """Immediately schedule life-safety events bypassing all limits"""
        try:
            # Check if this is a life-safety event
            if not self._is_life_safety_event(candidate):
                return False
            
            # Create micro-session with K=12
            job_data = {
                "session_id": f"life_safety_{candidate.event_id}",
                "home_id": candidate.home_id,
                "event_ids": [candidate.event_id],
                "tier": "emergency",
                "K": 12,
                "deadline_ms": 2000,  # 2 second deadline
                "priority": "LIFE_SAFETY",
                "enqueued_at": datetime.utcnow().isoformat(),
                "bypass_reason": "life_safety_event"
            }
            
            # Add to emergency queue (highest priority)
            await self.redis_client.lpush("deep_processing_emergency", json.dumps(job_data))
            
            # Track as processing
            self.processing_jobs.add(candidate.event_id)
            
            # Remove from candidate store
            await self.candidate_store.remove_candidate(
                candidate.event_id, candidate.home_id
            )
            
            logger.critical(f"LIFE SAFETY EVENT scheduled immediately: {candidate.event_id}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to schedule life safety event: {e}")
            return False
    
    def _is_life_safety_event(self, candidate: EventCandidate) -> bool:
        """Check if event is life-safety related"""
        life_safety_indicators = [
            'glassbreak', 'smoke', 'co', 'carbon_monoxide', 
            'forced_entry', 'emergency', 'alarm', 'break_in'
        ]
        
        # Check explainer text
        if candidate.lite_explainer:
            explainer_lower = candidate.lite_explainer.lower()
            if any(indicator in explainer_lower for indicator in life_safety_indicators):
                return True
        
        # Check location for forced entry indicators
        if candidate.location in ['front_door', 'back_door'] and candidate.priority >= Priority.CRITICAL:
            return True
        
        # Check mode
        if candidate.mode in ['emergency', 'alarm']:
            return True
            
        return False

class SchedulerDaemon:
    """Background daemon for continuous scheduling"""
    
    def __init__(
        self,
        scheduler: ProcessingScheduler,
        schedule_interval: int = 30  # seconds
    ):
        self.scheduler = scheduler
        self.schedule_interval = schedule_interval
        self.running = False
        self.last_stats = {}
    
    async def start(self):
        """Start the scheduler daemon"""
        self.running = True
        logger.info(f"Starting scheduler daemon (interval: {self.schedule_interval}s)")
        
        while self.running:
            try:
                # Run scheduling round
                round_stats = await self.scheduler.schedule_round()
                self.last_stats = round_stats
                
                # Cleanup stale jobs
                await self.scheduler.cleanup_stale_jobs()
                
                # Wait for next round
                await asyncio.sleep(self.schedule_interval)
                
            except Exception as e:
                logger.error(f"Scheduler daemon error: {e}")
                await asyncio.sleep(5)  # Brief pause on error
    
    def stop(self):
        """Stop the scheduler daemon"""
        self.running = False
        logger.info("Scheduler daemon stopped")
    
    def get_status(self) -> Dict[str, Any]:
        """Get current daemon status"""
        return {
            "running": self.running,
            "schedule_interval": self.schedule_interval,
            "last_stats": self.last_stats,
            "processing_jobs": len(self.scheduler.processing_jobs)
        }

# Integration with existing mobile API
async def integrate_with_mobile_api(
    candidate_store: CandidateStore,
    scheduler: ProcessingScheduler
):
    """
    Integration function to be called from mobile_api_endpoints.py
    when new events are created or lite results are submitted
    """
    
    async def on_new_event(event_data: Dict[str, Any]):
        """Handle new event from mobile API"""
        from candidate_store import create_event_candidate_from_api
        
        candidate = await create_event_candidate_from_api(event_data)
        await candidate_store.add_candidate(candidate)
        logger.info(f"Added new event candidate: {candidate.event_id}")
    
    async def on_lite_results(event_id: str, lite_results: Dict[str, Any]):
        """Handle lite processing results from mobile API"""
        # Get existing candidate
        candidate = await candidate_store.get_candidate(event_id)
        if not candidate:
            logger.warning(f"No candidate found for event {event_id}")
            return
        
        # Update with lite results
        candidate.lite_processed = True
        candidate.lite_channels = lite_results.get('channels', {})
        candidate.lite_explainer = lite_results.get('explainer')
        candidate.lite_confidence = lite_results.get('confidence')
        
        # Update detection flags
        candidate.person_detected = candidate.lite_channels.get('person', False)
        candidate.vehicle_detected = candidate.lite_channels.get('vehicle', False)
        
        # Re-add candidate with updated score
        await candidate_store.add_candidate(candidate)
        logger.info(f"Updated candidate {event_id} with lite results")
    
    return on_new_event, on_lite_results

# Performance monitoring
class SchedulerMetrics:
    """Metrics collection for scheduler performance"""
    
    def __init__(self, redis_client):
        self.redis_client = redis_client
        self.metrics_key = "scheduler_metrics"
    
    async def record_round(self, stats: Dict[str, Any]):
        """Record scheduling round metrics"""
        try:
            timestamp = datetime.utcnow().isoformat()
            metric_data = {
                "timestamp": timestamp,
                **stats
            }
            
            # Store in Redis list (keep last 100 rounds)
            await self.redis_client.lpush(
                self.metrics_key,
                json.dumps(metric_data)
            )
            await self.redis_client.ltrim(self.metrics_key, 0, 99)
            
        except Exception as e:
            logger.error(f"Failed to record metrics: {e}")
    
    async def get_recent_metrics(self, count: int = 20) -> List[Dict[str, Any]]:
        """Get recent scheduling metrics"""
        try:
            metrics_data = await self.redis_client.lrange(
                self.metrics_key, 0, count - 1
            )
            
            return [json.loads(data) for data in metrics_data]
            
        except Exception as e:
            logger.error(f"Failed to get metrics: {e}")
            return []
    
    async def get_summary_stats(self) -> Dict[str, Any]:
        """Get summary statistics"""
        try:
            recent_metrics = await self.get_recent_metrics(50)
            
            if not recent_metrics:
                return {}
            
            total_scheduled = sum(m.get("scheduled_events", 0) for m in recent_metrics)
            avg_round_duration = sum(m.get("round_duration_ms", 0) for m in recent_metrics) / len(recent_metrics)
            
            return {
                "total_scheduled_last_50_rounds": total_scheduled,
                "avg_round_duration_ms": avg_round_duration,
                "rounds_analyzed": len(recent_metrics),
                "last_round": recent_metrics[0] if recent_metrics else None
            }
            
        except Exception as e:
            logger.error(f"Failed to get summary stats: {e}")
            return {}

async def main():
    """Test the scheduler"""
    # Initialize components
    candidate_store = CandidateStore()
    await candidate_store.initialize()
    
    scheduler = ProcessingScheduler(candidate_store)
    await scheduler.initialize()
    
    try:
        # Add some test candidates
        test_candidates = [
            EventCandidate(
                event_id=f"test_event_{i}",
                home_id="home_test",
                user_id="user_test",
                timestamp=datetime.utcnow() - timedelta(minutes=i),
                priority=Priority.HIGH if i % 2 == 0 else Priority.NORMAL,
                processing_tier=ProcessingTier.PREMIUM,
                image_url=f"https://example.com/test_{i}.jpg",
                location="front_door" if i % 3 == 0 else "backyard",
                mode="security",
                lite_processed=True,
                lite_channels={"person": i % 2 == 0, "vehicle": i % 3 == 0},
                person_detected=i % 2 == 0,
                vehicle_detected=i % 3 == 0,
                motion_score=0.3 + (i % 7) * 0.1
            )
            for i in range(10)
        ]
        
        # Add candidates to store
        for candidate in test_candidates:
            await candidate_store.add_candidate(candidate)
        
        print("Added test candidates")
        
        # Run scheduling round
        stats = await scheduler.schedule_round()
        print(f"Scheduling results: {json.dumps(stats, indent=2)}")
        
        # Get queue stats
        queue_stats = await scheduler.get_queue_stats()
        print(f"Queue stats: {json.dumps(queue_stats, indent=2)}")
        
        # Get store stats
        store_stats = await candidate_store.get_stats()
        print(f"Store stats: {json.dumps(store_stats, indent=2)}")
        
    finally:
        await scheduler.close()
        await candidate_store.close()

if __name__ == "__main__":
    asyncio.run(main())

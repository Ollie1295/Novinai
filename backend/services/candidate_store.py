#!/usr/bin/env python3
"""
Candidate Store Service
Redis ZSET-based intelligent event storage and prioritization
Implements cand:{home_id} ZSET + ev:{event_id} hash structure
"""

import asyncio
import logging
import json
import time
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional, Tuple
import redis.asyncio as redis
from dataclasses import dataclass, asdict
from enum import IntEnum
import hashlib

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class Priority(IntEnum):
    """Event priority levels"""
    LOW = 1
    NORMAL = 2
    HIGH = 3
    CRITICAL = 4

class ProcessingTier(IntEnum):
    """Processing tier levels"""
    LITE_ONLY = 1      # Mobile/server-lite only
    STANDARD = 2       # Basic deep processing
    PREMIUM = 3        # Full deep processing + re-ID
    ENTERPRISE = 4     # Everything + real-time alerts

@dataclass
class EventCandidate:
    """Event candidate for processing"""
    event_id: str
    home_id: str
    user_id: str
    timestamp: datetime
    priority: Priority
    processing_tier: ProcessingTier
    
    # Event data
    image_url: str
    location: str
    mode: str
    
    # Lite processing results (if available)
    lite_processed: bool = False
    lite_channels: Optional[Dict[str, bool]] = None
    lite_explainer: Optional[str] = None
    lite_confidence: Optional[float] = None
    
    # Scoring factors
    person_detected: bool = False
    vehicle_detected: bool = False
    motion_score: float = 0.0
    time_of_day_factor: float = 1.0
    location_importance: float = 1.0
    
    def calculate_score(self) -> float:
        """Calculate priority score for ZSET ordering"""
        base_score = float(self.priority) * 100
        
        # Add detection bonuses
        if self.person_detected:
            base_score += 50
        if self.vehicle_detected:
            base_score += 30
            
        # Add motion and context factors
        base_score += self.motion_score * 20
        base_score *= self.time_of_day_factor
        base_score *= self.location_importance
        
        # Add recency bonus (fresher events get higher scores)
        age_minutes = (datetime.utcnow() - self.timestamp).total_seconds() / 60
        recency_bonus = max(0, 60 - age_minutes) * 2  # Decay over 1 hour
        base_score += recency_bonus
        
        # Add tier multiplier
        base_score *= (1 + (float(self.processing_tier) * 0.2))
        
        return base_score

class CandidateStore:
    """Redis-based candidate storage with ZSET prioritization"""
    
    def __init__(self, redis_url: str = "redis://localhost:6379"):
        self.redis_url = redis_url
        self.redis_client = None
        self.max_candidates_per_home = 2000  # Cap ZSET size per home
        self.candidate_ttl = 86400  # 24 hours
        self.cleanup_batch_size = 100  # Batch cleanup operations
        
    async def initialize(self):
        """Initialize Redis connection"""
        self.redis_client = redis.from_url(self.redis_url, decode_responses=True)
        logger.info("Candidate store initialized")
    
    async def close(self):
        """Close Redis connection"""
        if self.redis_client:
            await self.redis_client.close()
    
    async def add_candidate(self, candidate: EventCandidate) -> bool:
        """Add event candidate to store with idempotency"""
        try:
            # Check if event already exists (idempotency)
            event_key = f"ev:{candidate.event_id}"
            exists = await self.redis_client.exists(event_key)
            
            score = candidate.calculate_score()
            candidate_key = f"cand:{candidate.home_id}"
            
            if exists:
                # Update existing candidate score only
                await self.redis_client.zadd(candidate_key, {candidate.event_id: score})
                await self.redis_client.expire(candidate_key, self.candidate_ttl)
                logger.info(f"Updated existing candidate {candidate.event_id} with score {score:.2f}")
                return True
            
            # Store event data in hash: ev:{event_id}
            event_data = asdict(candidate)
            event_data['timestamp'] = candidate.timestamp.isoformat()
            
            # Store as hash with TTL
            await self.redis_client.hset(event_key, mapping={
                k: json.dumps(v) if isinstance(v, (dict, list)) else str(v)
                for k, v in event_data.items()
            })
            await self.redis_client.expire(event_key, self.candidate_ttl)
            
            # Store in ZSET: cand:{home_id}
            await self.redis_client.zadd(candidate_key, {candidate.event_id: score})
            await self.redis_client.expire(candidate_key, self.candidate_ttl)
            
            # Also store in expiration auxiliary index
            expire_key = f"expire:{int(time.time()) + self.candidate_ttl}"
            await self.redis_client.sadd(expire_key, f"{candidate.home_id}:{candidate.event_id}")
            await self.redis_client.expire(expire_key, self.candidate_ttl + 3600)  # Extra buffer
            
            # Maintain max candidates per home
            await self._cleanup_old_candidates(candidate.home_id)
            
            logger.info(f"Added new candidate {candidate.event_id} for home {candidate.home_id} with score {score:.2f}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to add candidate: {e}")
            return False
    
    async def get_top_candidates(self, home_id: str, limit: int = 10) -> List[EventCandidate]:
        """Get top candidates for a home by score"""
        try:
            candidate_key = f"cand:{home_id}"
            
            # Get top candidates with scores (highest first)
            results = await self.redis_client.zrevrange(
                candidate_key, 0, limit - 1, withscores=True
            )
            
            candidates = []
            for event_id, score in results:
                candidate = await self.get_candidate(event_id)
                if candidate:
                    candidates.append(candidate)
            
            return candidates
            
        except Exception as e:
            logger.error(f"Failed to get top candidates: {e}")
            return []
    
    async def get_candidate(self, event_id: str) -> Optional[EventCandidate]:
        """Get candidate by event ID"""
        try:
            event_key = f"ev:{event_id}"
            data = await self.redis_client.hgetall(event_key)
            
            if not data:
                return None
            
            # Convert back to EventCandidate
            parsed_data = {}
            for k, v in data.items():
                if k in ['lite_channels']:
                    parsed_data[k] = json.loads(v) if v != 'None' else None
                elif k == 'timestamp':
                    parsed_data[k] = datetime.fromisoformat(v)
                elif k in ['priority', 'processing_tier']:
                    parsed_data[k] = int(v)
                elif k in ['lite_processed', 'person_detected', 'vehicle_detected']:
                    parsed_data[k] = v.lower() == 'true'
                elif k in ['lite_confidence', 'motion_score', 'time_of_day_factor', 'location_importance']:
                    parsed_data[k] = float(v) if v != 'None' else 0.0
                else:
                    parsed_data[k] = v if v != 'None' else None
            
            return EventCandidate(**parsed_data)
            
        except Exception as e:
            logger.error(f"Failed to get candidate {event_id}: {e}")
            return None
    
    async def update_candidate_score(self, event_id: str, home_id: str) -> bool:
        """Recalculate and update candidate score"""
        try:
            candidate = await self.get_candidate(event_id)
            if not candidate:
                return False
            
            new_score = candidate.calculate_score()
            candidate_key = f"cand:{home_id}"
            
            await self.redis_client.zadd(candidate_key, {event_id: new_score})
            logger.info(f"Updated score for {event_id}: {new_score:.2f}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to update candidate score: {e}")
            return False
    
    async def remove_candidate(self, event_id: str, home_id: str) -> bool:
        """Remove candidate from store"""
        try:
            # Remove from ZSET
            candidate_key = f"cand:{home_id}"
            await self.redis_client.zrem(candidate_key, event_id)
            
            # Remove event data
            event_key = f"ev:{event_id}"
            await self.redis_client.delete(event_key)
            
            logger.info(f"Removed candidate {event_id}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to remove candidate: {e}")
            return False
    
    async def get_candidates_by_tier(self, processing_tier: ProcessingTier, limit: int = 100) -> List[EventCandidate]:
        """Get candidates across all homes by processing tier"""
        try:
            # Get all home candidate sets
            home_keys = await self.redis_client.keys("cand:*")
            
            all_candidates = []
            for home_key in home_keys:
                home_id = home_key.split(":")[1]
                candidates = await self.get_top_candidates(home_id, limit)
                
                # Filter by processing tier
                tier_candidates = [
                    c for c in candidates 
                    if c.processing_tier == processing_tier
                ]
                all_candidates.extend(tier_candidates)
            
            # Sort by score globally
            all_candidates.sort(key=lambda c: c.calculate_score(), reverse=True)
            
            return all_candidates[:limit]
            
        except Exception as e:
            logger.error(f"Failed to get candidates by tier: {e}")
            return []
    
    async def get_pending_candidates(self, limit: int = 100) -> List[EventCandidate]:
        """Get candidates that haven't been processed yet"""
        try:
            all_candidates = []
            
            # Get all home candidate sets
            home_keys = await self.redis_client.keys("cand:*")
            
            for home_key in home_keys:
                home_id = home_key.split(":")[1]
                candidates = await self.get_top_candidates(home_id, limit)
                
                # Filter for unprocessed events
                pending = [c for c in candidates if not c.lite_processed]
                all_candidates.extend(pending)
            
            # Sort by score
            all_candidates.sort(key=lambda c: c.calculate_score(), reverse=True)
            
            return all_candidates[:limit]
            
        except Exception as e:
            logger.error(f"Failed to get pending candidates: {e}")
            return []
    
    async def _cleanup_old_candidates(self, home_id: str):
        """Remove oldest candidates if over limit"""
        try:
            candidate_key = f"cand:{home_id}"
            count = await self.redis_client.zcard(candidate_key)
            
            if count > self.max_candidates_per_home:
                # Remove oldest (lowest score) candidates
                to_remove = count - self.max_candidates_per_home
                old_candidates = await self.redis_client.zrange(
                    candidate_key, 0, to_remove - 1
                )
                
                if old_candidates:
                    # Remove from ZSET
                    await self.redis_client.zrem(candidate_key, *old_candidates)
                    
                    # Remove event data
                    for event_id in old_candidates:
                        event_key = f"ev:{event_id}"
                        await self.redis_client.delete(event_key)
                    
                    logger.info(f"Cleaned up {len(old_candidates)} old candidates for home {home_id}")
            
        except Exception as e:
            logger.error(f"Failed to cleanup old candidates: {e}")
    
    async def get_stats(self) -> Dict[str, Any]:
        """Get candidate store statistics"""
        try:
            home_keys = await self.redis_client.keys("cand:*")
            event_keys = await self.redis_client.keys("ev:*")
            
            total_candidates = 0
            tier_counts = {tier.name: 0 for tier in ProcessingTier}
            priority_counts = {priority.name: 0 for priority in Priority}
            
            for home_key in home_keys:
                count = await self.redis_client.zcard(home_key)
                total_candidates += count
            
            # Sample some events for tier/priority distribution
            for event_key in event_keys[:100]:  # Sample first 100
                candidate_data = await self.redis_client.hgetall(event_key)
                if candidate_data:
                    tier = ProcessingTier(int(candidate_data.get('processing_tier', 2)))
                    priority = Priority(int(candidate_data.get('priority', 2)))
                    tier_counts[tier.name] += 1
                    priority_counts[priority.name] += 1
            
            return {
                "total_candidates": total_candidates,
                "total_homes": len(home_keys),
                "total_events": len(event_keys),
                "tier_distribution": tier_counts,
                "priority_distribution": priority_counts,
                "timestamp": datetime.utcnow().isoformat()
            }
            
        except Exception as e:
            logger.error(f"Failed to get stats: {e}")
            return {}

# Utility functions for easy integration
async def create_event_candidate_from_api(
    event_data: Dict[str, Any],
    lite_results: Optional[Dict[str, Any]] = None
) -> EventCandidate:
    """Create EventCandidate from API event data"""
    
    # Determine home_id (could be derived from user_id or location)
    home_id = event_data.get('home_id', f"home_{hashlib.md5(event_data['user_id'].encode()).hexdigest()[:8]}")
    
    # Parse lite results if available
    lite_processed = lite_results is not None
    lite_channels = lite_results.get('channels', {}) if lite_results else {}
    
    # Determine priority based on lite results
    priority = Priority.NORMAL
    if lite_channels.get('person', False):
        priority = Priority.HIGH
    if event_data.get('priority', 1) >= 3:
        priority = Priority.CRITICAL
    
    # Determine processing tier (could be based on user subscription)
    processing_tier = ProcessingTier.STANDARD
    
    candidate = EventCandidate(
        event_id=event_data['event_id'],
        home_id=home_id,
        user_id=event_data['user_id'],
        timestamp=datetime.fromisoformat(event_data['timestamp']) if isinstance(event_data['timestamp'], str) else event_data['timestamp'],
        priority=priority,
        processing_tier=processing_tier,
        
        image_url=event_data['image_url'],
        location=event_data.get('location', 'unknown'),
        mode=event_data.get('mode', 'security'),
        
        lite_processed=lite_processed,
        lite_channels=lite_channels,
        lite_explainer=lite_results.get('explainer') if lite_results else None,
        lite_confidence=lite_results.get('confidence') if lite_results else None,
        
        person_detected=lite_channels.get('person', False),
        vehicle_detected=lite_channels.get('vehicle', False),
        motion_score=event_data.get('motion_score', 0.5),
        time_of_day_factor=calculate_time_factor(event_data['timestamp']),
        location_importance=get_location_importance(event_data.get('location', 'unknown'))
    )
    
    return candidate

def calculate_time_factor(timestamp: datetime) -> float:
    """Calculate time-of-day importance factor"""
    hour = timestamp.hour
    
    # Higher importance during typical away hours
    if 8 <= hour <= 18:  # Work hours
        return 1.2
    elif 22 <= hour or hour <= 6:  # Night hours
        return 1.5
    else:  # Evening hours
        return 1.0

def get_location_importance(location: str) -> float:
    """Get location importance multiplier"""
    importance_map = {
        'front_door': 1.5,
        'back_door': 1.4,
        'driveway': 1.3,
        'garage': 1.2,
        'backyard': 1.1,
        'living_room': 1.0,
        'bedroom': 0.8,
        'unknown': 1.0
    }
    
    return importance_map.get(location.lower(), 1.0)

async def main():
    """Test the candidate store"""
    store = CandidateStore()
    await store.initialize()
    
    try:
        # Create test candidate
        test_candidate = EventCandidate(
            event_id="test_event_123",
            home_id="home_abc",
            user_id="user_456",
            timestamp=datetime.utcnow(),
            priority=Priority.HIGH,
            processing_tier=ProcessingTier.PREMIUM,
            image_url="https://example.com/test.jpg",
            location="front_door",
            mode="security",
            lite_processed=True,
            lite_channels={"person": True, "vehicle": False, "pet": False},
            lite_explainer="Person detected at front door",
            lite_confidence=0.85,
            person_detected=True,
            motion_score=0.8
        )
        
        # Add candidate
        await store.add_candidate(test_candidate)
        
        # Get top candidates
        top = await store.get_top_candidates("home_abc", 5)
        print(f"Top candidates: {len(top)}")
        
        for candidate in top:
            print(f"- {candidate.event_id}: {candidate.calculate_score():.2f}")
        
        # Get stats
        stats = await store.get_stats()
        print(f"Stats: {stats}")
        
    finally:
        await store.close()

if __name__ == "__main__":
    asyncio.run(main())

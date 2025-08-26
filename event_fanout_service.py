#!/usr/bin/env python3
"""
Event Fan-out Service with Mobile Push Integration
Handles silent push notifications when new camera events arrive
"""

import asyncio
import asyncpg
import json
import logging
from typing import List, Dict, Optional
from datetime import datetime, timedelta
import os

from push_service import send_silent_push

logger = logging.getLogger(__name__)

class EventFanoutService:
    def __init__(self):
        self.pg_dsn = os.getenv("PG_DSN", "postgres://novin:novin@100.85.81.109:5432/novin")
        self.redis_url = os.getenv("REDIS_URL", "redis://:novin_redis_2024@95.179.193.224:6379/0")
        self.fallback_timeout = 5  # seconds to wait for mobile response
    
    async def get_active_devices(self, home_id: str) -> List[Dict]:
        """Get active mobile devices for a home"""
        try:
            conn = await asyncpg.connect(self.pg_dsn)
            
            # Get devices active in last 24 hours
            devices = await conn.fetch("""
                SELECT device_id, platform, push_token, battery_level, thermal_state
                FROM mobile_devices 
                WHERE home_id = $1 
                AND last_seen_at > NOW() - INTERVAL '24 hours'
                ORDER BY last_seen_at DESC
            """, home_id)
            
            await conn.close()
            return [dict(device) for device in devices]
            
        except Exception as e:
            logger.error(f"Failed to get active devices: {e}")
            return []
    
    async def create_signed_thumbnail_url(self, image_url: str, expiry_seconds: int = 60) -> str:
        """Create signed thumbnail URL with short expiry"""
        # TODO: Implement proper signed URL generation for your storage
        # For now, return the original URL (replace with R2/S3 signed URL logic)
        return image_url
    
    async def should_process_on_device(self, device: Dict, home_prefs: Dict) -> bool:
        """Check if device should process event based on battery/thermal state"""
        
        # Check battery level
        if device.get("battery_level"):
            battery = device["battery_level"]
            if battery < 0.2 and not home_prefs.get("process_on_battery", True):
                return False
        
        # Check thermal state
        thermal = device.get("thermal_state", "nominal")
        if thermal in ["critical", "serious"]:
            return False
        
        return True
    
    async def get_home_prefs(self, home_id: str) -> Dict:
        """Get mobile processing preferences for home"""
        try:
            conn = await asyncpg.connect(self.pg_dsn)
            
            prefs = await conn.fetchrow("""
                SELECT process_on_battery, max_px_small, lite_timeout_ms
                FROM mobile_prefs 
                WHERE home_id = $1
            """, home_id)
            
            await conn.close()
            
            if prefs:
                return dict(prefs)
            else:
                # Return defaults
                return {
                    "process_on_battery": True,
                    "max_px_small": 384,
                    "lite_timeout_ms": 300
                }
                
        except Exception as e:
            logger.error(f"Failed to get home prefs: {e}")
            return {"process_on_battery": True, "max_px_small": 384, "lite_timeout_ms": 300}
    
    async def enqueue_server_lite_fallback(self, event_data: Dict):
        """Enqueue event for server-side lite processing as fallback"""
        try:
            import redis.asyncio as redis
            
            r = redis.from_url(self.redis_url)
            
            # Add to server lite processing queue
            lite_job = {
                "event_id": event_data["event_id"],
                "home_id": event_data["home_id"],
                "image_url": event_data["image_url"],
                "mode": "lite",
                "fallback_reason": "mobile_timeout",
                "timestamp": datetime.utcnow().isoformat()
            }
            
            await r.xadd("stream:lite.jobs", lite_job)
            await r.close()
            
            logger.info(f"Enqueued server lite fallback for event {event_data['event_id']}")
            
        except Exception as e:
            logger.error(f"Failed to enqueue server lite fallback: {e}")
    
    async def wait_for_mobile_response(self, event_id: str, timeout_seconds: int = 5) -> bool:
        """Wait for mobile device to report processing completion"""
        try:
            import redis.asyncio as redis
            
            r = redis.from_url(self.redis_url)
            
            # Check if event was processed by mobile device
            # This could be implemented by checking a Redis key or database record
            # For now, simulate with a simple timeout
            
            start_time = datetime.utcnow()
            while (datetime.utcnow() - start_time).total_seconds() < timeout_seconds:
                # Check if mobile device reported completion
                # TODO: Implement actual check (Redis key, database record, etc.)
                await asyncio.sleep(0.5)
            
            await r.close()
            return False  # Timeout - no mobile response
            
        except Exception as e:
            logger.error(f"Error waiting for mobile response: {e}")
            return False
    
    async def process_camera_event(self, event_data: Dict):
        """Process new camera event with mobile push fan-out"""
        
        event_id = event_data.get("event_id")
        home_id = event_data.get("home_id")
        image_url = event_data.get("image_url")
        
        if not all([event_id, home_id, image_url]):
            logger.error(f"Invalid event data: {event_data}")
            return
        
        logger.info(f"Processing camera event {event_id} for home {home_id}")
        
        # Get active devices and home preferences
        devices = await self.get_active_devices(home_id)
        home_prefs = await self.get_home_prefs(home_id)
        
        if not devices:
            logger.info(f"No active devices for home {home_id}, using server fallback")
            await self.enqueue_server_lite_fallback(event_data)
            return
        
        # Filter devices that can process
        capable_devices = []
        for device in devices:
            if await self.should_process_on_device(device, home_prefs):
                capable_devices.append(device)
        
        if not capable_devices:
            logger.info(f"No capable devices for home {home_id}, using server fallback")
            await self.enqueue_server_lite_fallback(event_data)
            return
        
        # Create signed thumbnail URL
        signed_url = await self.create_signed_thumbnail_url(image_url, expiry_seconds=60)
        
        # Prepare push notification payload
        push_payload = {
            "event_id": event_id,
            "image_url": signed_url,
            "mode": event_data.get("mode", "guardian"),
            "context": event_data.get("context", {})
        }
        
        # Send silent push notifications
        push_results = await send_silent_push(capable_devices, push_payload)
        
        logger.info(f"Push results for event {event_id}: {push_results}")
        
        # Wait for mobile response with timeout
        mobile_responded = await self.wait_for_mobile_response(event_id, self.fallback_timeout)
        
        if not mobile_responded:
            logger.info(f"Mobile timeout for event {event_id}, using server fallback")
            await self.enqueue_server_lite_fallback(event_data)

# Global service instance
fanout_service = EventFanoutService()

async def handle_camera_event(event_data: Dict):
    """Handle new camera event with mobile fan-out"""
    await fanout_service.process_camera_event(event_data)

if __name__ == "__main__":
    # Test the fan-out service
    import sys
    
    logging.basicConfig(level=logging.INFO)
    
    async def test_fanout():
        # Test event data
        test_event = {
            "event_id": "test_event_123",
            "home_id": "test_home_1",
            "image_url": "https://example.com/test_image.jpg",
            "mode": "guardian",
            "context": {
                "presence": "away",
                "distance_to_perimeter_m": 0.8
            }
        }
        
        print("🧪 Testing event fan-out service...")
        await handle_camera_event(test_event)
        print("✅ Test completed")
    
    try:
        asyncio.run(test_fanout())
    except KeyboardInterrupt:
        print("❌ Test interrupted")
    except Exception as e:
        print(f"❌ Test failed: {e}")
        sys.exit(1)

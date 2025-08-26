#!/usr/bin/env python3
"""
Push Notification Service for Mobile Devices
Handles APNs (iOS) and FCM (Android) silent push notifications
"""

import httpx
import jwt
import time
import json
import os
import asyncio
from typing import Dict, List, Optional
import logging

# Configuration
APNS_TEAM = os.getenv("APNS_TEAM")
APNS_KEY_ID = os.getenv("APNS_KEY_ID")
APNS_BUNDLE = os.getenv("APNS_BUNDLE")
APNS_P8_PATH = os.getenv("APNS_P8_PATH")
FCM_KEY = os.getenv("FCM_KEY")

logger = logging.getLogger(__name__)

class PushService:
    def __init__(self):
        self.apns_jwt_cache = None
        self.apns_jwt_expires = 0
    
    def apns_jwt(self) -> str:
        """Generate or return cached APNs JWT token"""
        now = int(time.time())
        
        # Cache JWT for 50 minutes (APNs allows 60 min max)
        if self.apns_jwt_cache and now < self.apns_jwt_expires:
            return self.apns_jwt_cache
        
        if not all([APNS_TEAM, APNS_KEY_ID, APNS_BUNDLE, APNS_P8_PATH]):
            raise ValueError("Missing APNs configuration")
        
        try:
            with open(APNS_P8_PATH, 'rb') as f:
                key = f.read()
            
            payload = {
                "iss": APNS_TEAM,
                "iat": now
            }
            
            token = jwt.encode(
                payload, 
                key, 
                algorithm="ES256", 
                headers={"kid": APNS_KEY_ID}
            )
            
            self.apns_jwt_cache = token
            self.apns_jwt_expires = now + (50 * 60)  # 50 minutes
            
            return token
            
        except Exception as e:
            logger.error(f"Failed to generate APNs JWT: {e}")
            raise
    
    async def push_ios(self, token: str, payload: Dict) -> bool:
        """Send silent push notification to iOS device"""
        try:
            auth = self.apns_jwt()
            url = f"https://api.push.apple.com/3/device/{token}"
            
            headers = {
                "authorization": f"bearer {auth}",
                "apns-topic": APNS_BUNDLE,
                "apns-push-type": "background",
                "content-type": "application/json"
            }
            
            async with httpx.AsyncClient(http2=True, timeout=5) as client:
                response = await client.post(url, headers=headers, json=payload)
                
                if response.status_code == 200:
                    logger.debug(f"APNs push sent successfully to {token[:8]}...")
                    return True
                else:
                    logger.warning(f"APNs push failed: {response.status_code} - {response.text}")
                    return False
                    
        except Exception as e:
            logger.error(f"APNs push error: {e}")
            return False
    
    async def push_android(self, token: str, payload: Dict) -> bool:
        """Send silent push notification to Android device"""
        try:
            if not FCM_KEY:
                raise ValueError("Missing FCM_KEY configuration")
            
            url = "https://fcm.googleapis.com/fcm/send"
            
            body = {
                "to": token,
                "data": payload,
                "priority": "high",
                "content_available": True
            }
            
            headers = {
                "authorization": f"key={FCM_KEY}",
                "content-type": "application/json"
            }
            
            async with httpx.AsyncClient(timeout=5) as client:
                response = await client.post(url, headers=headers, data=json.dumps(body))
                
                if response.status_code == 200:
                    result = response.json()
                    if result.get("success", 0) > 0:
                        logger.debug(f"FCM push sent successfully to {token[:8]}...")
                        return True
                    else:
                        logger.warning(f"FCM push failed: {result}")
                        return False
                else:
                    logger.warning(f"FCM push failed: {response.status_code} - {response.text}")
                    return False
                    
        except Exception as e:
            logger.error(f"FCM push error: {e}")
            return False
    
    async def send_event_notification(self, devices: List[Dict], event_data: Dict) -> Dict[str, int]:
        """Send event notification to multiple devices"""
        results = {"ios_sent": 0, "android_sent": 0, "failed": 0}
        
        # Prepare payload
        payload = {
            "content-available": 1,
            "event_id": event_data["event_id"],
            "image_url": event_data["image_url"],
            "mode": event_data.get("mode", "guardian"),
            "context": json.dumps(event_data.get("context", {}))
        }
        
        # Send to all devices concurrently
        tasks = []
        for device in devices:
            if device["platform"] == "ios":
                task = self.push_ios(device["push_token"], payload)
            elif device["platform"] == "android":
                task = self.push_android(device["push_token"], payload)
            else:
                continue
            
            tasks.append((device["platform"], task))
        
        # Wait for all pushes to complete
        if tasks:
            task_results = await asyncio.gather(*[task for _, task in tasks], return_exceptions=True)
            
            for i, (platform, _) in enumerate(tasks):
                success = task_results[i] if not isinstance(task_results[i], Exception) else False
                
                if success:
                    results[f"{platform}_sent"] += 1
                else:
                    results["failed"] += 1
        
        logger.info(f"Push notification results: {results}")
        return results

# Global instance
push_service = PushService()

# Convenience functions
async def send_silent_push(devices: List[Dict], event_data: Dict) -> Dict[str, int]:
    """Send silent push notifications for security event"""
    return await push_service.send_event_notification(devices, event_data)

if __name__ == "__main__":
    # Test configuration
    import sys
    
    logging.basicConfig(level=logging.DEBUG)
    
    required_env = ["APNS_TEAM", "APNS_KEY_ID", "APNS_BUNDLE", "APNS_P8_PATH", "FCM_KEY"]
    missing = [var for var in required_env if not os.getenv(var)]
    
    if missing:
        print(f"❌ Missing environment variables: {missing}")
        sys.exit(1)
    
    print("✅ Push service configuration valid")
    
    # Test JWT generation
    try:
        service = PushService()
        jwt_token = service.apns_jwt()
        print(f"✅ APNs JWT generated: {jwt_token[:20]}...")
    except Exception as e:
        print(f"❌ APNs JWT generation failed: {e}")
        sys.exit(1)
    
    print("🚀 Push service ready for production")

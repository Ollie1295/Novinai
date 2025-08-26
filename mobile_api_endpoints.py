#!/usr/bin/env python3
"""
Mobile API Endpoints for Device Registration and Management
FastAPI endpoints for mobile device integration
"""

from fastapi import FastAPI, HTTPException, Depends, Header
from pydantic import BaseModel, UUID4
from typing import Optional, Dict, Any
import asyncpg
import os
import json
import logging
from datetime import datetime

logger = logging.getLogger(__name__)

# Pydantic models
class DeviceRegistration(BaseModel):
    device_id: UUID4
    platform: str  # 'ios' or 'android'
    push_token: str

class DeviceAck(BaseModel):
    device_id: UUID4
    battery: Optional[float] = None
    thermal: Optional[str] = "nominal"

class EventReport(BaseModel):
    event_id: str
    channels: Optional[Dict[str, Any]] = None
    explainer: Optional[str] = None
    context: Optional[Dict[str, Any]] = None

# Database connection
async def get_db_connection():
    """Get PostgreSQL connection"""
    pg_dsn = os.getenv("PG_DSN", "postgres://novin:novin@100.85.81.109:5432/novin")
    return await asyncpg.connect(pg_dsn)

# Auth dependency (simplified - replace with proper Supabase JWT validation)
async def get_current_user(authorization: str = Header(None)) -> str:
    """Extract home_id from Authorization header"""
    if not authorization or not authorization.startswith("Bearer "):
        raise HTTPException(status_code=401, detail="Missing or invalid authorization")
    
    # TODO: Validate Supabase JWT and extract home_id
    # For now, return test home_id
    return "test_home_1"

# FastAPI app
app = FastAPI(title="Mobile API", version="1.0.0")

@app.post("/mobile/register")
async def register_device(
    registration: DeviceRegistration,
    home_id: str = Depends(get_current_user)
):
    """Register mobile device for push notifications"""
    
    if registration.platform not in ['ios', 'android']:
        raise HTTPException(status_code=400, detail="Platform must be 'ios' or 'android'")
    
    try:
        conn = await get_db_connection()
        
        # Upsert device registration
        await conn.execute("""
            INSERT INTO mobile_devices (device_id, home_id, platform, push_token, last_seen_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (device_id, home_id) 
            DO UPDATE SET 
                platform = EXCLUDED.platform,
                push_token = EXCLUDED.push_token,
                last_seen_at = NOW()
        """, registration.device_id, home_id, registration.platform, registration.push_token)
        
        await conn.close()
        
        logger.info(f"Device registered: {registration.device_id} ({registration.platform}) for home {home_id}")
        
        return {
            "status": "registered",
            "device_id": registration.device_id,
            "home_id": home_id
        }
        
    except Exception as e:
        logger.error(f"Device registration failed: {e}")
        raise HTTPException(status_code=500, detail="Registration failed")

@app.post("/mobile/ack")
async def device_ack(
    ack: DeviceAck,
    home_id: str = Depends(get_current_user)
):
    """Device acknowledgment with battery and thermal status"""
    
    try:
        conn = await get_db_connection()
        
        # Update device status
        result = await conn.execute("""
            UPDATE mobile_devices 
            SET last_seen_at = NOW(),
                battery_level = $1,
                thermal_state = $2
            WHERE device_id = $3 AND home_id = $4
        """, ack.battery, ack.thermal, ack.device_id, home_id)
        
        await conn.close()
        
        if result == "UPDATE 0":
            raise HTTPException(status_code=404, detail="Device not found")
        
        logger.debug(f"Device ack: {ack.device_id} battery={ack.battery} thermal={ack.thermal}")
        
        return {
            "status": "acknowledged",
            "device_id": ack.device_id,
            "timestamp": datetime.utcnow().isoformat()
        }
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Device ack failed: {e}")
        raise HTTPException(status_code=500, detail="Acknowledgment failed")

@app.post("/events")
async def report_event(
    event: EventReport,
    home_id: str = Depends(get_current_user)
):
    """Report processed event from mobile device"""
    
    try:
        # TODO: Store event report in database
        # This endpoint receives results from on-device processing
        
        logger.info(f"Event report from mobile: {event.event_id} for home {home_id}")
        
        # Basic validation
        if not event.event_id:
            raise HTTPException(status_code=400, detail="event_id is required")
        
        # Store in events table or send to processing pipeline
        # For now, just log the event
        event_data = {
            "event_id": event.event_id,
            "home_id": home_id,
            "source": "mobile_device",
            "channels": event.channels,
            "explainer": event.explainer,
            "context": event.context,
            "timestamp": datetime.utcnow().isoformat()
        }
        
        logger.info(f"Mobile event processed: {json.dumps(event_data, indent=2)}")
        
        return {
            "status": "received",
            "event_id": event.event_id,
            "processed_at": datetime.utcnow().isoformat()
        }
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Event report failed: {e}")
        raise HTTPException(status_code=500, detail="Event processing failed")

@app.get("/mobile/devices")
async def list_devices(home_id: str = Depends(get_current_user)):
    """List registered devices for home"""
    
    try:
        conn = await get_db_connection()
        
        devices = await conn.fetch("""
            SELECT device_id, platform, created_at, last_seen_at, 
                   battery_level, thermal_state
            FROM mobile_devices 
            WHERE home_id = $1
            ORDER BY last_seen_at DESC
        """, home_id)
        
        await conn.close()
        
        return {
            "devices": [dict(device) for device in devices],
            "count": len(devices)
        }
        
    except Exception as e:
        logger.error(f"List devices failed: {e}")
        raise HTTPException(status_code=500, detail="Failed to list devices")

@app.get("/mobile/prefs")
async def get_mobile_prefs(home_id: str = Depends(get_current_user)):
    """Get mobile processing preferences for home"""
    
    try:
        conn = await get_db_connection()
        
        prefs = await conn.fetchrow("""
            SELECT process_on_battery, max_px_small, lite_timeout_ms
            FROM mobile_prefs 
            WHERE home_id = $1
        """, home_id)
        
        await conn.close()
        
        if not prefs:
            # Return defaults
            return {
                "process_on_battery": True,
                "max_px_small": 384,
                "lite_timeout_ms": 300
            }
        
        return dict(prefs)
        
    except Exception as e:
        logger.error(f"Get mobile prefs failed: {e}")
        raise HTTPException(status_code=500, detail="Failed to get preferences")

if __name__ == "__main__":
    import uvicorn
    
    logging.basicConfig(level=logging.INFO)
    
    # Test database connection
    import asyncio
    
    async def test_db():
        try:
            conn = await get_db_connection()
            result = await conn.fetchval("SELECT 1")
            await conn.close()
            print("✅ Database connection successful")
            return True
        except Exception as e:
            print(f"❌ Database connection failed: {e}")
            return False
    
    if asyncio.run(test_db()):
        print("🚀 Starting mobile API server...")
        uvicorn.run(app, host="0.0.0.0", port=8001)
    else:
        print("❌ Cannot start server - database connection failed")

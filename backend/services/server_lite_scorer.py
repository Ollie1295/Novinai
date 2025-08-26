#!/usr/bin/env python3
"""
Server-Lite Scorer Microservice
CPU-based fallback for mobile lite processing
Provides lightweight classification when device cannot process
"""

import asyncio
import logging
import json
import time
import tempfile
import os
from datetime import datetime
from typing import Dict, List, Any, Optional
from pathlib import Path
import httpx
from fastapi import FastAPI, HTTPException, Depends, BackgroundTasks
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from pydantic import BaseModel
import redis.asyncio as redis
import numpy as np

# Try to import lightweight models
try:
    import torch
    import torchvision.transforms as transforms
    from torchvision import models
    from PIL import Image
    import cv2
    TORCH_AVAILABLE = True
except ImportError:
    TORCH_AVAILABLE = False

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# FastAPI app
app = FastAPI(title="Server-Lite Scorer", version="1.0.0")
security = HTTPBearer()

# Pydantic models
class LiteProcessingRequest(BaseModel):
    event_id: str
    image_url: str
    user_id: str
    device_id: Optional[str] = None
    fallback_reason: Optional[str] = None  # Why device couldn't process
    priority: int = 2

class LiteProcessingResponse(BaseModel):
    event_id: str
    success: bool
    processing_time_ms: int
    channels: Dict[str, bool]
    confidence: float
    explainer: str
    error_message: Optional[str] = None

class HealthResponse(BaseModel):
    status: str
    version: str
    models_loaded: bool
    uptime_seconds: float
    total_processed: int

# Global state
redis_client = None
models_loaded = False
start_time = time.time()
total_processed = 0

class LiteClassifier:
    """Lightweight classifier for basic object detection"""
    
    def __init__(self):
        self.device = "cpu"  # Server-lite runs on CPU
        self.models = {}
        self.transforms = None
        
    def load_models(self):
        """Load lightweight classification models"""
        global models_loaded
        
        try:
            if TORCH_AVAILABLE:
                logger.info("Loading lightweight PyTorch models...")
                
                # Person classifier (using pretrained model + custom head)
                self.models['person'] = models.mobilenet_v3_small(pretrained=True)
                self.models['person'].classifier = torch.nn.Sequential(
                    torch.nn.Linear(576, 64),
                    torch.nn.ReLU(),
                    torch.nn.Dropout(0.2),
                    torch.nn.Linear(64, 2)  # Binary: person/no_person
                )
                self.models['person'].eval()
                
                # Vehicle classifier
                self.models['vehicle'] = models.mobilenet_v3_small(pretrained=True)
                self.models['vehicle'].classifier = torch.nn.Sequential(
                    torch.nn.Linear(576, 64),
                    torch.nn.ReLU(),
                    torch.nn.Dropout(0.2),
                    torch.nn.Linear(64, 2)  # Binary: vehicle/no_vehicle
                )
                self.models['vehicle'].eval()
                
                # Pet classifier
                self.models['pet'] = models.mobilenet_v3_small(pretrained=True)
                self.models['pet'].classifier = torch.nn.Sequential(
                    torch.nn.Linear(576, 64),
                    torch.nn.ReLU(),
                    torch.nn.Dropout(0.2),
                    torch.nn.Linear(64, 2)  # Binary: pet/no_pet
                )
                self.models['pet'].eval()
                
                # Image transforms
                self.transforms = transforms.Compose([
                    transforms.Resize((224, 224)),
                    transforms.ToTensor(),
                    transforms.Normalize(mean=[0.485, 0.456, 0.406], 
                                       std=[0.229, 0.224, 0.225])
                ])
                
                logger.info("Lightweight models loaded successfully")
            else:
                logger.warning("PyTorch not available, using stub models")
            
            models_loaded = True
            
        except Exception as e:
            logger.error(f"Failed to load models: {e}")
            models_loaded = False
    
    async def classify_image(self, image_path: str) -> Dict[str, Any]:
        """Classify image using lightweight models"""
        try:
            if TORCH_AVAILABLE and self.models:
                return await self._classify_with_torch(image_path)
            else:
                return await self._classify_stub(image_path)
                
        except Exception as e:
            logger.error(f"Classification failed: {e}")
            return {
                "channels": {"person": False, "vehicle": False, "pet": False},
                "confidence": 0.0,
                "explainer": f"Classification error: {e}"
            }
    
    async def _classify_with_torch(self, image_path: str) -> Dict[str, Any]:
        """Classify using actual PyTorch models"""
        try:
            # Load and preprocess image
            image = Image.open(image_path).convert('RGB')
            input_tensor = self.transforms(image).unsqueeze(0)
            
            results = {}
            confidences = []
            
            # Run each classifier
            with torch.no_grad():
                for class_name, model in self.models.items():
                    output = model(input_tensor)
                    probabilities = torch.nn.functional.softmax(output, dim=1)
                    confidence = probabilities[0][1].item()  # Positive class probability
                    
                    results[class_name] = confidence > 0.5
                    confidences.append(confidence)
            
            overall_confidence = np.mean(confidences)
            
            # Generate explainer
            detected = [k for k, v in results.items() if v]
            if detected:
                explainer = f"Detected: {', '.join(detected)}"
            else:
                explainer = "No significant objects detected"
            
            return {
                "channels": results,
                "confidence": overall_confidence,
                "explainer": explainer
            }
            
        except Exception as e:
            logger.error(f"PyTorch classification failed: {e}")
            raise
    
    async def _classify_stub(self, image_path: str) -> Dict[str, Any]:
        """Stub classification for testing"""
        await asyncio.sleep(0.2)  # Simulate processing time
        
        # Mock results based on filename/path
        image_name = Path(image_path).name.lower()
        
        channels = {
            "person": "person" in image_name or "human" in image_name,
            "vehicle": any(v in image_name for v in ["car", "truck", "vehicle", "bike"]),
            "pet": any(p in image_name for p in ["dog", "cat", "pet", "animal"]),
            "linger": "linger" in image_name or hash(image_name) % 10 < 2  # 20% linger
        }
        
        # Calculate exact scoring contract
        score_result = self._calculate_exact_score(channels, image_name)
        
        detected = [k for k, v in channels.items() if v and k != 'linger']
        explainer = f"Detected: {', '.join(detected)}" if detected else "No objects detected"
        if channels.get('linger'):
            explainer += " (lingering)"
        
        return {
            "channels": channels,
            "confidence": score_result['confidence'],
            "explainer": explainer,
            "score": score_result['score'],
            "threshold_met": score_result['threshold_met'],
            "mode": score_result['mode']
        }
    
    def _calculate_exact_score(self, channels: Dict[str, bool], image_name: str, mode: str = "guardian") -> Dict[str, Any]:
        """Calculate exact scoring contract - MUST match device scoring"""
        # base = 1.00*person + 0.70*vehicle + 0.15*linger
        base = (
            1.00 * (1 if channels.get('person', False) else 0) +
            0.70 * (1 if channels.get('vehicle', False) else 0) +
            0.15 * (1 if channels.get('linger', False) else 0)
        )
        
        # pet_factor = (1 - 0.60*pet)
        pet_factor = 1 - 0.60 * (1 if channels.get('pet', False) else 0)
        
        # perimeter_factor = 1.25 if distance_to_perimeter_m < 1.5 else 1.0
        # For stub: mock perimeter distance based on hash
        mock_distance = (hash(image_name) % 50) / 10.0  # 0-5m range
        perimeter_factor = 1.25 if mock_distance < 1.5 else 1.0
        
        # night_factor = 1.15 if is_night else 1.0
        # For stub: mock night based on hash
        is_night = (hash(image_name) % 4) == 0  # 25% night
        night_factor = 1.15 if is_night else 1.0
        
        # mode_factor = {'stealth':0.70,'guardian':1.00,'perimeter':1.30}[mode]
        mode_factors = {
            'stealth': 0.70,
            'guardian': 1.00,
            'perimeter': 1.30
        }
        mode_factor = mode_factors.get(mode, 1.00)
        
        # final_score = base * pet_factor * perimeter_factor * night_factor * mode_factor
        final_score = base * pet_factor * perimeter_factor * night_factor * mode_factor
        
        # Clamp [0,1]
        final_score = max(0.0, min(1.0, final_score))
        
        # Thresholds: Stealth <0.35/0.65, Guardian <0.30/0.60, Perimeter <0.25/0.50
        thresholds = {
            'stealth': (0.35, 0.65),
            'guardian': (0.30, 0.60), 
            'perimeter': (0.25, 0.50)
        }
        low_thresh, high_thresh = thresholds.get(mode, (0.30, 0.60))
        
        if final_score < low_thresh:
            threshold_met = 'low'
        elif final_score < high_thresh:
            threshold_met = 'medium'
        else:
            threshold_met = 'high'
        
        return {
            'score': final_score,
            'confidence': final_score,  # Use score as confidence
            'threshold_met': threshold_met,
            'mode': mode,
            'factors': {
                'base': base,
                'pet_factor': pet_factor,
                'perimeter_factor': perimeter_factor,
                'night_factor': night_factor,
                'mode_factor': mode_factor
            }
        }

# Global classifier instance
classifier = LiteClassifier()

@app.on_event("startup")
async def startup_event():
    """Initialize the service"""
    global redis_client
    
    try:
        # Connect to Redis
        redis_client = redis.from_url("redis://localhost:6379", decode_responses=True)
        
        # Load models
        classifier.load_models()
        
        logger.info("Server-Lite Scorer started successfully")
        
    except Exception as e:
        logger.error(f"Startup failed: {e}")
        raise

@app.on_event("shutdown")
async def shutdown_event():
    """Cleanup on shutdown"""
    global redis_client
    
    if redis_client:
        await redis_client.close()
    
    logger.info("Server-Lite Scorer shut down")

async def verify_token(credentials: HTTPAuthorizationCredentials = Depends(security)):
    """Verify JWT token (simplified for this microservice)"""
    try:
        # In production, verify JWT signature
        # For now, just check if token exists
        if not credentials.credentials:
            raise HTTPException(status_code=401, detail="Invalid token")
        
        return credentials.credentials
        
    except Exception:
        raise HTTPException(status_code=401, detail="Invalid token")

async def download_image(image_url: str, event_id: str) -> Optional[str]:
    """Download image for processing"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(image_url, timeout=30)
            response.raise_for_status()
            
            # Save to temp file
            suffix = Path(image_url).suffix or '.jpg'
            temp_file = tempfile.NamedTemporaryFile(
                delete=False, 
                suffix=suffix,
                prefix=f"lite_{event_id}_"
            )
            
            temp_file.write(response.content)
            temp_file.close()
            
            return temp_file.name
            
    except Exception as e:
        logger.error(f"Failed to download image {image_url}: {e}")
        return None

@app.post("/classify", response_model=LiteProcessingResponse)
async def classify_image(
    request: LiteProcessingRequest,
    background_tasks: BackgroundTasks,
    token: str = Depends(verify_token)
):
    """Classify image using server-lite processing"""
    global total_processed
    
    start_time_local = time.time()
    
    try:
        logger.info(f"Processing lite classification for event {request.event_id}")
        
        # Download image
        image_path = await download_image(request.image_url, request.event_id)
        if not image_path:
            raise HTTPException(status_code=400, detail="Failed to download image")
        
        try:
            # Run classification
            result = await classifier.classify_image(image_path)
            
            processing_time = int((time.time() - start_time_local) * 1000)
            total_processed += 1
            
            # Record metrics
            background_tasks.add_task(
                record_processing_metrics,
                request.event_id,
                request.user_id,
                processing_time,
                True,
                request.fallback_reason
            )
            
            return LiteProcessingResponse(
                event_id=request.event_id,
                success=True,
                processing_time_ms=processing_time,
                channels=result["channels"],
                confidence=result["confidence"],
                explainer=result["explainer"]
            )
            
        finally:
            # Cleanup image file
            try:
                os.unlink(image_path)
            except Exception:
                pass
    
    except HTTPException:
        raise
    except Exception as e:
        processing_time = int((time.time() - start_time_local) * 1000)
        
        # Record failure metrics
        background_tasks.add_task(
            record_processing_metrics,
            request.event_id,
            request.user_id,
            processing_time,
            False,
            request.fallback_reason,
            str(e)
        )
        
        logger.error(f"Classification failed for {request.event_id}: {e}")
        
        return LiteProcessingResponse(
            event_id=request.event_id,
            success=False,
            processing_time_ms=processing_time,
            channels={"person": False, "vehicle": False, "pet": False},
            confidence=0.0,
            explainer="Processing failed",
            error_message=str(e)
        )

@app.get("/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint"""
    global total_processed, start_time
    
    uptime = time.time() - start_time
    
    return HealthResponse(
        status="healthy" if models_loaded else "degraded",
        version="1.0.0",
        models_loaded=models_loaded,
        uptime_seconds=uptime,
        total_processed=total_processed
    )

@app.get("/stats")
async def get_stats(token: str = Depends(verify_token)):
    """Get processing statistics"""
    global total_processed, start_time
    
    try:
        if not redis_client:
            raise HTTPException(status_code=503, detail="Redis not available")
        
        # Get recent metrics
        metrics_data = await redis_client.lrange("lite_scorer_metrics", 0, 99)
        recent_metrics = [json.loads(data) for data in metrics_data]
        
        # Calculate stats
        recent_successes = sum(1 for m in recent_metrics if m.get('success', False))
        recent_failures = len(recent_metrics) - recent_successes
        
        avg_processing_time = 0
        if recent_metrics:
            avg_processing_time = sum(m.get('processing_time_ms', 0) for m in recent_metrics) / len(recent_metrics)
        
        # Fallback reason distribution
        fallback_reasons = {}
        for metric in recent_metrics:
            reason = metric.get('fallback_reason', 'unknown')
            fallback_reasons[reason] = fallback_reasons.get(reason, 0) + 1
        
        return {
            "uptime_seconds": time.time() - start_time,
            "total_processed": total_processed,
            "models_loaded": models_loaded,
            "recent_stats": {
                "successful": recent_successes,
                "failed": recent_failures,
                "success_rate": recent_successes / len(recent_metrics) if recent_metrics else 0,
                "avg_processing_time_ms": avg_processing_time
            },
            "fallback_reasons": fallback_reasons,
            "timestamp": datetime.utcnow().isoformat()
        }
        
    except Exception as e:
        logger.error(f"Failed to get stats: {e}")
        raise HTTPException(status_code=500, detail="Failed to get stats")

@app.post("/batch_classify")
async def batch_classify(
    requests: List[LiteProcessingRequest],
    background_tasks: BackgroundTasks,
    token: str = Depends(verify_token)
):
    """Batch classify multiple images"""
    if len(requests) > 10:
        raise HTTPException(status_code=400, detail="Batch size too large (max 10)")
    
    start_time_local = time.time()
    
    try:
        logger.info(f"Processing batch of {len(requests)} images")
        
        # Process all images in parallel
        tasks = [
            process_single_image(req) 
            for req in requests
        ]
        
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        # Convert results to responses
        responses = []
        for i, result in enumerate(results):
            request = requests[i]
            
            if isinstance(result, Exception):
                response = LiteProcessingResponse(
                    event_id=request.event_id,
                    success=False,
                    processing_time_ms=0,
                    channels={"person": False, "vehicle": False, "pet": False},
                    confidence=0.0,
                    explainer="Batch processing error",
                    error_message=str(result)
                )
            else:
                response = result
            
            responses.append(response)
        
        batch_time = int((time.time() - start_time_local) * 1000)
        
        # Record batch metrics
        background_tasks.add_task(
            record_batch_metrics,
            len(requests),
            sum(1 for r in responses if r.success),
            batch_time
        )
        
        return responses
        
    except Exception as e:
        logger.error(f"Batch processing failed: {e}")
        raise HTTPException(status_code=500, detail="Batch processing failed")

async def process_single_image(request: LiteProcessingRequest) -> LiteProcessingResponse:
    """Process a single image classification request"""
    start_time_local = time.time()
    
    try:
        # Download image
        image_path = await download_image(request.image_url, request.event_id)
        if not image_path:
            raise Exception("Failed to download image")
        
        try:
            # Run classification
            result = await classifier.classify_image(image_path)
            
            processing_time = int((time.time() - start_time_local) * 1000)
            
            return LiteProcessingResponse(
                event_id=request.event_id,
                success=True,
                processing_time_ms=processing_time,
                channels=result["channels"],
                confidence=result["confidence"],
                explainer=result["explainer"]
            )
            
        finally:
            # Cleanup
            try:
                os.unlink(image_path)
            except Exception:
                pass
    
    except Exception as e:
        processing_time = int((time.time() - start_time_local) * 1000)
        
        return LiteProcessingResponse(
            event_id=request.event_id,
            success=False,
            processing_time_ms=processing_time,
            channels={"person": False, "vehicle": False, "pet": False},
            confidence=0.0,
            explainer="Processing failed",
            error_message=str(e)
        )

async def record_processing_metrics(
    event_id: str,
    user_id: str,
    processing_time_ms: int,
    success: bool,
    fallback_reason: Optional[str] = None,
    error_message: Optional[str] = None
):
    """Record processing metrics to Redis"""
    try:
        if not redis_client:
            return
        
        metrics = {
            "event_id": event_id,
            "user_id": user_id,
            "processing_time_ms": processing_time_ms,
            "success": success,
            "fallback_reason": fallback_reason,
            "error_message": error_message,
            "timestamp": datetime.utcnow().isoformat()
        }
        
        await redis_client.lpush("lite_scorer_metrics", json.dumps(metrics))
        await redis_client.ltrim("lite_scorer_metrics", 0, 999)  # Keep last 1000
        
    except Exception as e:
        logger.error(f"Failed to record metrics: {e}")

async def record_batch_metrics(batch_size: int, successful: int, batch_time_ms: int):
    """Record batch processing metrics"""
    try:
        if not redis_client:
            return
        
        metrics = {
            "batch_size": batch_size,
            "successful": successful,
            "failed": batch_size - successful,
            "batch_time_ms": batch_time_ms,
            "throughput": batch_size / (batch_time_ms / 1000) if batch_time_ms > 0 else 0,
            "timestamp": datetime.utcnow().isoformat()
        }
        
        await redis_client.lpush("lite_scorer_batch_metrics", json.dumps(metrics))
        await redis_client.ltrim("lite_scorer_batch_metrics", 0, 99)  # Keep last 100
        
    except Exception as e:
        logger.error(f"Failed to record batch metrics: {e}")

# Integration with mobile API
class LiteScorerClient:
    """Client for calling the lite scorer service"""
    
    def __init__(self, base_url: str = "http://localhost:8002", auth_token: str = None):
        self.base_url = base_url.rstrip('/')
        self.auth_token = auth_token
        self.client = httpx.AsyncClient()
    
    async def classify_image(
        self,
        event_id: str,
        image_url: str,
        user_id: str,
        fallback_reason: str = "device_unavailable"
    ) -> Dict[str, Any]:
        """Classify image via server-lite scorer"""
        try:
            headers = {}
            if self.auth_token:
                headers["Authorization"] = f"Bearer {self.auth_token}"
            
            request_data = {
                "event_id": event_id,
                "image_url": image_url,
                "user_id": user_id,
                "fallback_reason": fallback_reason
            }
            
            response = await self.client.post(
                f"{self.base_url}/classify",
                json=request_data,
                headers=headers,
                timeout=30
            )
            
            response.raise_for_status()
            return response.json()
            
        except Exception as e:
            logger.error(f"Lite scorer request failed: {e}")
            return {
                "success": False,
                "channels": {"person": False, "vehicle": False, "pet": False},
                "confidence": 0.0,
                "explainer": f"Server-lite processing failed: {e}",
                "error_message": str(e)
            }
    
    async def close(self):
        """Close the client"""
        await self.client.aclose()

# Standalone runner
async def run_server():
    """Run the server-lite scorer as a standalone service"""
    import uvicorn
    
    config = uvicorn.Config(
        app,
        host="0.0.0.0",
        port=8002,
        log_level="info",
        reload=False
    )
    
    server = uvicorn.Server(config)
    await server.serve()

async def main():
    """Test the server-lite scorer"""
    # Test the classifier directly
    classifier.load_models()
    
    # Test with a sample image URL
    test_image_url = "https://via.placeholder.com/640x480.jpg"
    
    try:
        # Download test image
        image_path = await download_image(test_image_url, "test_event")
        if image_path:
            # Test classification
            result = await classifier.classify_image(image_path)
            print(f"Classification result: {json.dumps(result, indent=2)}")
            
            # Cleanup
            try:
                os.unlink(image_path)
            except Exception:
                pass
        
        # Test client
        client = LiteScorerClient()
        result = await client.classify_image(
            "test_event_456",
            test_image_url,
            "test_user",
            "testing"
        )
        print(f"Client result: {json.dumps(result, indent=2)}")
        
        await client.close()
        
    except Exception as e:
        logger.error(f"Test failed: {e}")

if __name__ == "__main__":
    import sys
    
    if len(sys.argv) > 1 and sys.argv[1] == "server":
        # Run as server
        asyncio.run(run_server())
    else:
        # Run tests
        asyncio.run(main())

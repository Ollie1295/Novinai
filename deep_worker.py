#!/usr/bin/env python3
"""
Deep Worker - PC-based heavy AI processing for Novin Security
Processes Top-K items per AI Session with YOLOv8 + re-ID/face/plate detection
"""

import os
import json
import time
import asyncio
import logging
import traceback
from typing import List, Dict, Any, Optional
from dataclasses import dataclass
from collections import defaultdict

import redis
import torch
import numpy as np
from PIL import Image
import requests
from io import BytesIO

# Configure logging
logging.basicConfig(
    level=getattr(logging, os.getenv("LOG_LEVEL", "INFO")),
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

@dataclass
class DeepJobItem:
    """Single item in a deep processing job"""
    item_id: str
    image_url: str
    timestamp: float
    roi_hint: Optional[Dict] = None
    priority: str = "normal"

@dataclass
class DeepJob:
    """Deep processing job containing multiple items"""
    job_id: str
    session_id: str
    user_id: str
    items: List[DeepJobItem]
    tier: str
    created_at: float

@dataclass
class DeepResult:
    """Result from deep processing"""
    job_id: str
    item_id: str
    detections: List[Dict]
    faces: List[Dict]
    plates: List[Dict]
    reid_features: Optional[np.ndarray]
    processing_time_ms: float
    confidence_scores: Dict[str, float]

class ModelManager:
    """Manages AI models for deep processing"""
    
    def __init__(self):
        self.device = os.getenv("DEVICE", "cpu")
        self.model_name = os.getenv("MODEL_NAME", "yolov8n-int8")
        self.conf_thresh = float(os.getenv("CONF_THRESH", "0.25"))
        self.iou_thresh = float(os.getenv("IOU_THRESH", "0.5"))
        
        self.detector = None
        self.face_model = None
        self.plate_model = None
        self.reid_model = None
        
        self._load_models()
    
    def _load_models(self):
        """Load AI models based on configuration"""
        logger.info(f"Loading models on device: {self.device}")
        
        try:
            # Load main detector
            if self.model_name.endswith("-trt"):
                # TensorRT engine
                model_path = f"./models/{self.model_name}.engine"
                if os.path.exists(model_path):
                    logger.info(f"Loading TensorRT model: {model_path}")
                    # Load TensorRT model (placeholder - implement based on your TRT setup)
                    self.detector = self._load_tensorrt_model(model_path)
                else:
                    logger.warning(f"TensorRT model not found, falling back to PyTorch")
                    self.detector = self._load_pytorch_model()
            elif self.model_name.endswith("-int8"):
                # ONNX INT8 model
                model_path = f"./models/{self.model_name}.onnx"
                if os.path.exists(model_path):
                    logger.info(f"Loading ONNX model: {model_path}")
                    self.detector = self._load_onnx_model(model_path)
                else:
                    logger.warning(f"ONNX model not found, falling back to PyTorch")
                    self.detector = self._load_pytorch_model()
            else:
                # Standard PyTorch model
                self.detector = self._load_pytorch_model()
            
            # Load specialized models (placeholder implementations)
            self.face_model = self._load_face_model()
            self.plate_model = self._load_plate_model()
            self.reid_model = self._load_reid_model()
            
            logger.info("All models loaded successfully")
            
        except Exception as e:
            logger.error(f"Failed to load models: {e}")
            raise
    
    def _load_pytorch_model(self):
        """Load standard PyTorch YOLOv8 model"""
        try:
            from ultralytics import YOLO
            model_name = self.model_name.replace("-int8", "").replace("-trt", "")
            if not model_name.endswith(".pt"):
                model_name += ".pt"
            
            model = YOLO(model_name)
            if self.device == "cuda" and torch.cuda.is_available():
                model.to("cuda")
            return model
        except ImportError:
            logger.error("ultralytics not installed. Install with: pip install ultralytics")
            raise
    
    def _load_onnx_model(self, model_path: str):
        """Load ONNX model (placeholder)"""
        logger.info(f"ONNX model loading not implemented yet: {model_path}")
        return self._load_pytorch_model()  # Fallback
    
    def _load_tensorrt_model(self, model_path: str):
        """Load TensorRT model (placeholder)"""
        logger.info(f"TensorRT model loading not implemented yet: {model_path}")
        return self._load_pytorch_model()  # Fallback
    
    def _load_face_model(self):
        """Load face detection/recognition model (placeholder)"""
        logger.info("Face model loading - placeholder implementation")
        return None
    
    def _load_plate_model(self):
        """Load license plate detection model (placeholder)"""
        logger.info("Plate model loading - placeholder implementation")
        return None
    
    def _load_reid_model(self):
        """Load person re-identification model (placeholder)"""
        logger.info("ReID model loading - placeholder implementation")
        return None
    
    def process_batch(self, images: List[Image.Image]) -> List[Dict]:
        """Process a batch of images through the detector"""
        if not self.detector:
            raise RuntimeError("Detector model not loaded")
        
        start_time = time.time()
        
        try:
            # Run inference
            results = self.detector(images, conf=self.conf_thresh, iou=self.iou_thresh)
            
            # Process results
            batch_results = []
            for i, result in enumerate(results):
                detections = []
                if hasattr(result, 'boxes') and result.boxes is not None:
                    boxes = result.boxes.xyxy.cpu().numpy()
                    confs = result.boxes.conf.cpu().numpy()
                    classes = result.boxes.cls.cpu().numpy()
                    
                    for j in range(len(boxes)):
                        detections.append({
                            "bbox": boxes[j].tolist(),
                            "confidence": float(confs[j]),
                            "class": int(classes[j]),
                            "class_name": result.names[int(classes[j])]
                        })
                
                batch_results.append({
                    "detections": detections,
                    "image_shape": images[i].size
                })
            
            processing_time = (time.time() - start_time) * 1000
            logger.debug(f"Processed batch of {len(images)} images in {processing_time:.1f}ms")
            
            return batch_results
            
        except Exception as e:
            logger.error(f"Batch processing failed: {e}")
            raise

class DeepWorker:
    """Main deep worker class"""
    
    def __init__(self):
        self.redis_client = redis.from_url(os.getenv("REDIS_URL"))
        self.model_manager = ModelManager()
        
        # Configuration
        self.batch_size = int(os.getenv("BATCH", "16"))
        self.batch_window_ms = int(os.getenv("BATCH_WINDOW_MS", "80"))
        self.max_bytes = int(os.getenv("MAX_BYTES", "5242880"))
        
        # Early exit flags
        self.skip_reid_if_empty = os.getenv("SKIP_REID_IF_EMPTY", "true").lower() == "true"
        self.run_face_only_if_person = os.getenv("RUN_FACE_ONLY_IF_PERSON", "true").lower() == "true"
        
        # Metrics
        self.metrics = defaultdict(int)
        self.last_health_check = time.time()
        
        logger.info(f"Deep worker initialized - batch_size: {self.batch_size}, window: {self.batch_window_ms}ms")
    
    async def run(self):
        """Main worker loop"""
        logger.info("Starting deep worker...")
        
        pending_items = []
        last_batch_time = time.time()
        
        while True:
            try:
                # Check for new jobs
                job_data = self.redis_client.xread(
                    {"stream:deep.jobs": "$"}, 
                    count=1, 
                    block=100
                )
                
                if job_data:
                    stream_name, messages = job_data[0]
                    for message_id, fields in messages:
                        try:
                            job = self._parse_job(fields)
                            pending_items.extend(job.items)
                            logger.debug(f"Added {len(job.items)} items to pending queue")
                        except Exception as e:
                            logger.error(f"Failed to parse job {message_id}: {e}")
                
                # Check if we should process a batch
                current_time = time.time()
                time_since_last_batch = (current_time - last_batch_time) * 1000
                
                should_process = (
                    len(pending_items) >= self.batch_size or
                    (pending_items and time_since_last_batch >= self.batch_window_ms)
                )
                
                if should_process:
                    batch_items = pending_items[:self.batch_size]
                    pending_items = pending_items[self.batch_size:]
                    
                    await self._process_batch(batch_items)
                    last_batch_time = current_time
                    
                    self.metrics["batches_processed"] += 1
                    self.metrics["items_processed"] += len(batch_items)
                
                # Health check
                if current_time - self.last_health_check > 30:
                    self._log_health_metrics()
                    self.last_health_check = current_time
                    
            except KeyboardInterrupt:
                logger.info("Shutting down deep worker...")
                break
            except Exception as e:
                logger.error(f"Worker error: {e}")
                logger.error(traceback.format_exc())
                await asyncio.sleep(1)
    
    def _parse_job(self, fields: Dict) -> DeepJob:
        """Parse job from Redis stream"""
        job_data = json.loads(fields[b'data'].decode())
        
        items = []
        for item_data in job_data.get("items", []):
            items.append(DeepJobItem(
                item_id=item_data["item_id"],
                image_url=item_data["image_url"],
                timestamp=item_data["timestamp"],
                roi_hint=item_data.get("roi_hint"),
                priority=item_data.get("priority", "normal")
            ))
        
        return DeepJob(
            job_id=job_data["job_id"],
            session_id=job_data["session_id"],
            user_id=job_data["user_id"],
            items=items,
            tier=job_data.get("tier", "standard"),
            created_at=job_data.get("created_at", time.time())
        )
    
    async def _process_batch(self, items: List[DeepJobItem]):
        """Process a batch of items"""
        logger.info(f"Processing batch of {len(items)} items")
        
        try:
            # Download images
            images = []
            valid_items = []
            
            for item in items:
                try:
                    image = await self._download_image(item.image_url)
                    if image:
                        images.append(image)
                        valid_items.append(item)
                except Exception as e:
                    logger.warning(f"Failed to download image {item.image_url}: {e}")
                    self.metrics["download_failures"] += 1
            
            if not images:
                logger.warning("No valid images in batch")
                return
            
            # Run detection
            start_time = time.time()
            detection_results = self.model_manager.process_batch(images)
            detection_time = (time.time() - start_time) * 1000
            
            # Process results and apply early exits
            results = []
            for i, (item, detection_result) in enumerate(zip(valid_items, detection_results)):
                detections = detection_result["detections"]
                
                # Early exit: skip expensive processing if no strong detections
                if self.skip_reid_if_empty and not detections:
                    self.metrics["early_exits"] += 1
                    continue
                
                # Run additional processing (face, plate, reid) based on detections
                faces = []
                plates = []
                reid_features = None
                
                if detections:
                    # Check if we have person detections for face processing
                    has_person = any(d["class_name"] == "person" for d in detections)
                    
                    if not self.run_face_only_if_person or has_person:
                        faces = await self._process_faces(images[i], detections)
                    
                    plates = await self._process_plates(images[i], detections)
                    
                    if has_person:
                        reid_features = await self._extract_reid_features(images[i], detections)
                
                result = DeepResult(
                    job_id=item.item_id.split("_")[0],  # Extract job_id from item_id
                    item_id=item.item_id,
                    detections=detections,
                    faces=faces,
                    plates=plates,
                    reid_features=reid_features,
                    processing_time_ms=detection_time / len(valid_items),
                    confidence_scores={
                        "detection": max([d["confidence"] for d in detections], default=0.0),
                        "face": max([f.get("confidence", 0.0) for f in faces], default=0.0),
                        "plate": max([p.get("confidence", 0.0) for p in plates], default=0.0)
                    }
                )
                
                results.append(result)
            
            # Write results to Redis
            await self._write_results(results)
            
            logger.info(f"Batch completed: {len(results)} results, {detection_time:.1f}ms detection time")
            
        except Exception as e:
            logger.error(f"Batch processing failed: {e}")
            logger.error(traceback.format_exc())
            self.metrics["batch_failures"] += 1
    
    async def _download_image(self, url: str) -> Optional[Image.Image]:
        """Download and validate image"""
        try:
            response = requests.get(url, timeout=10)
            response.raise_for_status()
            
            if len(response.content) > self.max_bytes:
                logger.warning(f"Image too large: {len(response.content)} bytes")
                return None
            
            image = Image.open(BytesIO(response.content))
            if image.mode != "RGB":
                image = image.convert("RGB")
            
            return image
            
        except Exception as e:
            logger.warning(f"Image download failed: {e}")
            return None
    
    async def _process_faces(self, image: Image.Image, detections: List[Dict]) -> List[Dict]:
        """Process face detection/recognition (placeholder)"""
        # TODO: Implement face processing
        return []
    
    async def _process_plates(self, image: Image.Image, detections: List[Dict]) -> List[Dict]:
        """Process license plate detection (placeholder)"""
        # TODO: Implement plate processing
        return []
    
    async def _extract_reid_features(self, image: Image.Image, detections: List[Dict]) -> Optional[np.ndarray]:
        """Extract person re-identification features (placeholder)"""
        # TODO: Implement ReID feature extraction
        return None
    
    async def _write_results(self, results: List[DeepResult]):
        """Write results to Redis stream"""
        for result in results:
            result_data = {
                "job_id": result.job_id,
                "item_id": result.item_id,
                "detections": json.dumps(result.detections),
                "faces": json.dumps(result.faces),
                "plates": json.dumps(result.plates),
                "processing_time_ms": result.processing_time_ms,
                "confidence_scores": json.dumps(result.confidence_scores),
                "timestamp": time.time()
            }
            
            if result.reid_features is not None:
                # Convert numpy array to base64 for storage
                import base64
                result_data["reid_features"] = base64.b64encode(result.reid_features.tobytes()).decode()
                result_data["reid_shape"] = json.dumps(result.reid_features.shape)
            
            self.redis_client.xadd("stream:deep.results", result_data)
    
    def _log_health_metrics(self):
        """Log health and performance metrics"""
        logger.info(f"Health metrics: {dict(self.metrics)}")
        
        # Reset counters (optional)
        # self.metrics.clear()

if __name__ == "__main__":
    worker = DeepWorker()
    asyncio.run(worker.run())

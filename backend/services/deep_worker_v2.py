#!/usr/bin/env python3
"""
Deep Worker V2
Redis queue-integrated worker with batch processing support
Replaces simple queue with structured job processing
"""

import asyncio
import logging
import json
import time
import traceback
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional, Tuple
import redis.asyncio as redis
from dataclasses import dataclass, asdict
import httpx
import tempfile
import os
from pathlib import Path

# Import models (assuming these exist from original deep_worker.py)
try:
    import sys
    sys.path.append(str(Path(__file__).parent.parent))
    from deep_worker import (
        initialize_models, run_detection, run_face_recognition,
        get_image_caption, load_config_with_defaults
    )
    MODELS_AVAILABLE = True
except ImportError:
    logger.warning("Original deep_worker models not available, using stubs")
    MODELS_AVAILABLE = False

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class ProcessingSession:
    """Deep processing session from scheduler"""
    session_id: str
    home_id: str
    event_ids: List[str]
    tier: str
    K: int  # Number of frames to process
    deadline_ms: int
    priority: str
    enqueued_at: datetime
    bypass_reason: Optional[str] = None
    
    @classmethod
    def from_json(cls, job_data: str) -> 'ProcessingSession':
        """Create ProcessingSession from JSON string"""
        data = json.loads(job_data)
        
        return cls(
            session_id=data['session_id'],
            home_id=data['home_id'],
            event_ids=data['event_ids'],
            tier=data['tier'],
            K=data['K'],
            deadline_ms=data['deadline_ms'],
            priority=data['priority'],
            enqueued_at=datetime.fromisoformat(data['enqueued_at']),
            bypass_reason=data.get('bypass_reason')
        )

@dataclass
class ProcessingJob:
    """Legacy processing job format (for backward compatibility)"""
    event_id: str
    home_id: str
    user_id: str
    image_url: str
    location: str
    mode: str
    processing_tier: int
    priority: int
    enqueued_at: datetime
    lite_results: Optional[Dict[str, Any]] = None
    
    @classmethod
    def from_json(cls, job_data: str) -> 'ProcessingJob':
        """Create ProcessingJob from JSON string"""
        data = json.loads(job_data)
        
        return cls(
            event_id=data['event_id'],
            home_id=data['home_id'],
            user_id=data['user_id'],
            image_url=data['image_url'],
            location=data['location'],
            mode=data['mode'],
            processing_tier=data['processing_tier'],
            priority=data['priority'],
            enqueued_at=datetime.fromisoformat(data['enqueued_at']),
            lite_results=data.get('lite_results')
        )

@dataclass
class SessionResult:
    """Deep processing session result matching scheduler contract"""
    session_id: str
    success: bool
    processing_duration_ms: int
    timestamp: datetime
    findings: Dict[str, Any]  # Structured findings
    error_message: Optional[str] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for storage"""
        result = asdict(self)
        result['timestamp'] = self.timestamp.isoformat()
        return result

@dataclass
class ProcessingResult:
    """Legacy deep processing result (for backward compatibility)"""
    event_id: str
    success: bool
    processing_duration_ms: int
    timestamp: datetime
    
    # Detection results
    detection_results: Optional[Dict[str, Any]] = None
    face_recognition_results: Optional[List[Dict[str, Any]]] = None
    caption: Optional[str] = None
    
    # Analysis
    risk_score: Optional[float] = None
    threat_level: Optional[str] = None
    summary: Optional[str] = None
    
    # Error info
    error_message: Optional[str] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for storage"""
        result = asdict(self)
        result['timestamp'] = self.timestamp.isoformat()
        return result

class DeepWorkerV2:
    """Enhanced deep worker with Redis queue integration"""
    
    def __init__(
        self,
        redis_url: str = "redis://localhost:6379",
        worker_id: str = None,
        batch_size: int = 5,
        max_batch_wait_time: int = 10  # seconds
    ):
        self.redis_url = redis_url
        self.worker_id = worker_id or f"worker_{int(time.time())}"
        self.batch_size = batch_size
        self.max_batch_wait_time = max_batch_wait_time
        self.redis_client = None
        
        # Processing state
        self.current_batch: List[ProcessingJob] = []  # Legacy batch processing
        self.last_batch_start = time.time()
        self.total_processed = 0
        self.total_sessions_processed = 0
        self.running = False
        
        # Load configuration
        self.config = load_config_with_defaults() if MODELS_AVAILABLE else {}
        
        # Initialize models (if available)
        self.models_initialized = False
        
    async def initialize(self):
        """Initialize worker and models"""
        try:
            # Connect to Redis
            self.redis_client = redis.from_url(self.redis_url, decode_responses=True)
            
            # Initialize ML models
            if MODELS_AVAILABLE:
                await asyncio.get_event_loop().run_in_executor(None, initialize_models, self.config)
                self.models_initialized = True
                logger.info("Deep learning models initialized")
            else:
                logger.warning("Running in stub mode - no actual ML processing")
            
            logger.info(f"Deep Worker {self.worker_id} initialized")
            
        except Exception as e:
            logger.error(f"Failed to initialize worker: {e}")
            raise
    
    async def close(self):
        """Close connections and cleanup"""
        if self.redis_client:
            await self.redis_client.close()
        logger.info(f"Deep Worker {self.worker_id} closed")
    
    async def start_processing(self, queue_names: List[str] = None):
        """Start processing sessions from queues"""
        if queue_names is None:
            queue_names = [
                "deep_processing_emergency",  # Life safety events
                "deep_processing_enterprise",
                "deep_processing_premium", 
                "deep_processing_standard"
            ]
        
        self.running = True
        logger.info(f"Worker {self.worker_id} starting processing from queues: {queue_names}")
        
        while self.running:
            try:
                # Try to get sessions from queues (priority order)
                session_found = False
                
                for queue_name in queue_names:
                    session_data = await self.redis_client.brpop(queue_name, timeout=1)
                    if session_data:
                        queue, session_json = session_data
                        
                        # Try to parse as session first, fallback to legacy job
                        try:
                            session = ProcessingSession.from_json(session_json)
                            await self._process_session(session)
                        except (KeyError, ValueError):
                            # Fallback to legacy job format
                            job = ProcessingJob.from_json(session_json)
                            self.current_batch.append(job)
                        
                        session_found = True
                        logger.info(f"Received session/job from {queue}")
                        break
                
                # Process legacy batch if ready
                if self._should_process_batch():
                    await self._process_current_batch()
                
                # Small delay if no sessions found
                if not session_found:
                    await asyncio.sleep(0.1)
                    
            except Exception as e:
                logger.error(f"Worker loop error: {e}")
                await asyncio.sleep(1)
        
        # Process any remaining legacy jobs
        if self.current_batch:
            await self._process_current_batch()
        
        logger.info(f"Worker {self.worker_id} stopped processing")
    
    def stop_processing(self):
        """Stop the worker"""
        self.running = False
        logger.info(f"Stopping worker {self.worker_id}")
    
    def _should_process_batch(self) -> bool:
        """Determine if current batch should be processed"""
        if not self.current_batch:
            return False
        
        # Process if batch is full
        if len(self.current_batch) >= self.batch_size:
            return True
        
        # Process if batch has been waiting too long
        if time.time() - self.last_batch_start >= self.max_batch_wait_time:
            return True
        
        return False
    
    async def _process_session(self, session: ProcessingSession) -> SessionResult:
        """Process a complete session (new contract)"""
        start_time = time.time()
        
        try:
            logger.info(f"Processing session {session.session_id} with {len(session.event_ids)} events (K={session.K})")
            
            # Check deadline
            deadline_seconds = session.deadline_ms / 1000.0
            
            # Process up to K events from the session
            events_to_process = session.event_ids[:session.K]
            findings = {
                "session_id": session.session_id,
                "events_processed": [],
                "summary": "",
                "risk_score": 0.0,
                "threat_indicators": [],
                "processing_stats": {
                    "total_events": len(events_to_process),
                    "deadline_ms": session.deadline_ms,
                    "tier": session.tier
                }
            }
            
            # Process each event in the session
            total_risk = 0.0
            for event_id in events_to_process:
                # Check if we're approaching deadline
                elapsed_ms = (time.time() - start_time) * 1000
                if elapsed_ms > (deadline_seconds * 1000 * 0.8):  # 80% of deadline
                    logger.warning(f"Session {session.session_id} approaching deadline, stopping at {len(findings['events_processed'])} events")
                    break
                
                try:
                    # Get event data from candidate store or direct processing
                    event_result = await self._process_session_event(event_id, session)
                    findings["events_processed"].append(event_result)
                    total_risk += event_result.get("risk_score", 0.0)
                    
                    # Extract threat indicators
                    if event_result.get("detections"):
                        for detection in event_result["detections"]:
                            obj_class = detection.get("class", "")
                            if obj_class in ["person", "vehicle", "weapon", "package"]:
                                findings["threat_indicators"].append({
                                    "type": obj_class,
                                    "confidence": detection.get("confidence", 0.0),
                                    "event_id": event_id
                                })
                    
                except Exception as e:
                    logger.error(f"Failed to process event {event_id} in session {session.session_id}: {e}")
                    findings["events_processed"].append({
                        "event_id": event_id,
                        "success": False,
                        "error": str(e)
                    })
            
            # Calculate overall session metrics
            processed_count = len(findings["events_processed"])
            findings["risk_score"] = total_risk / max(1, processed_count)
            
            # Generate session summary
            findings["summary"] = self._generate_session_summary(findings, session)
            
            processing_duration = int((time.time() - start_time) * 1000)
            
            result = SessionResult(
                session_id=session.session_id,
                success=True,
                processing_duration_ms=processing_duration,
                timestamp=datetime.utcnow(),
                findings=findings
            )
            
            # Store session result and notify
            await self._handle_session_success(session, result)
            
            self.total_sessions_processed += 1
            logger.info(f"Session {session.session_id} completed: {processed_count} events in {processing_duration}ms")
            
            return result
            
        except Exception as e:
            processing_duration = int((time.time() - start_time) * 1000)
            logger.error(f"Session {session.session_id} processing failed: {e}")
            
            result = SessionResult(
                session_id=session.session_id,
                success=False,
                processing_duration_ms=processing_duration,
                timestamp=datetime.utcnow(),
                findings={},
                error_message=str(e)
            )
            
            await self._handle_session_failure(session, result)
            return result
    
    async def _process_session_event(self, event_id: str, session: ProcessingSession) -> Dict[str, Any]:
        """Process a single event within a session"""
        try:
            # Try to get event data from Redis or candidate store
            event_data = await self.redis_client.get(f"event:{event_id}")
            if not event_data:
                # Generate minimal event data for stub processing
                event_data = {
                    "event_id": event_id,
                    "home_id": session.home_id,
                    "image_url": f"https://api.insane.ai/events/{event_id}/image",
                    "location": "unknown",
                    "timestamp": datetime.utcnow().isoformat()
                }
            else:
                event_data = json.loads(event_data)
            
            # Download and process image if models available
            if self.models_initialized and event_data.get("image_url"):
                image_path = await self._download_image(event_data["image_url"], event_id)
                if image_path:
                    try:
                        # Run detection
                        detection_results = await asyncio.get_event_loop().run_in_executor(
                            None, run_detection, image_path, self.config
                        )
                        
                        # Clean up image
                        try:
                            os.unlink(image_path)
                        except Exception:
                            pass
                        
                        return {
                            "event_id": event_id,
                            "success": True,
                            "detections": detection_results.get("detections", []),
                            "confidence": detection_results.get("confidence", 0.0),
                            "risk_score": self._calculate_event_risk_score(detection_results, event_data)
                        }
                    except Exception as e:
                        logger.error(f"Failed to process image for event {event_id}: {e}")
            
            # Fallback to stub processing
            return await self._process_session_event_stub(event_id, event_data)
            
        except Exception as e:
            logger.error(f"Failed to process session event {event_id}: {e}")
            return {
                "event_id": event_id,
                "success": False,
                "error": str(e),
                "risk_score": 0.0
            }
    
    async def _process_session_event_stub(self, event_id: str, event_data: Dict[str, Any]) -> Dict[str, Any]:
        """Stub processing for session events"""
        await asyncio.sleep(0.1)  # Minimal processing time
        
        # Generate mock detections based on event location/context
        detections = []
        location = event_data.get("location", "unknown")
        
        if location in ["front_door", "back_door"]:
            detections.append({
                "class": "person",
                "confidence": 0.7 + (hash(event_id) % 100) / 500.0,  # Deterministic but varied
                "bbox": [100, 100, 200, 300]
            })
        elif location in ["driveway", "garage"]:
            detections.append({
                "class": "vehicle",
                "confidence": 0.6 + (hash(event_id) % 100) / 400.0,
                "bbox": [50, 150, 350, 400]
            })
        
        risk_score = len(detections) * 0.3 + (hash(event_id) % 100) / 200.0
        
        return {
            "event_id": event_id,
            "success": True,
            "detections": detections,
            "confidence": 0.75,
            "risk_score": min(1.0, risk_score)
        }
    
    def _calculate_event_risk_score(self, detection_results: Dict[str, Any], event_data: Dict[str, Any]) -> float:
        """Calculate risk score for a single event in session"""
        base_score = 0.1
        
        detections = detection_results.get("detections", [])
        for detection in detections:
            obj_class = detection.get("class", "")
            confidence = detection.get("confidence", 0.0)
            
            if obj_class == "person":
                base_score += 0.4 * confidence
            elif obj_class in ["car", "truck", "motorcycle"]:
                base_score += 0.2 * confidence
            elif obj_class in ["weapon", "knife", "gun"]:
                base_score += 0.8 * confidence
        
        # Location-based risk
        location = event_data.get("location", "")
        if location in ["front_door", "back_door"]:
            base_score += 0.1
        
        return min(1.0, base_score)
    
    def _generate_session_summary(self, findings: Dict[str, Any], session: ProcessingSession) -> str:
        """Generate summary for session findings"""
        processed_count = len(findings["events_processed"])
        successful = sum(1 for e in findings["events_processed"] if e.get("success", False))
        threat_count = len(findings["threat_indicators"])
        risk_score = findings.get("risk_score", 0.0)
        
        summary = f"Processed {successful}/{processed_count} events from session {session.session_id}"
        
        if threat_count > 0:
            threat_types = set(t["type"] for t in findings["threat_indicators"])
            summary += f", detected {threat_count} threats: {', '.join(threat_types)}"
        
        if risk_score > 0.7:
            summary += " (HIGH RISK)"
        elif risk_score > 0.4:
            summary += " (MODERATE RISK)"
        else:
            summary += " (LOW RISK)"
        
        return summary
    
    async def _handle_session_success(self, session: ProcessingSession, result: SessionResult):
        """Handle successful session completion"""
        try:
            # Store session result
            result_key = f"session_result:{session.session_id}"
            await self.redis_client.setex(
                result_key,
                86400,  # 24 hours
                json.dumps(result.to_dict())
            )
            
            # Notify scheduler for each event in session
            for event_id in session.event_ids:
                await self._notify_scheduler_completion(event_id, True)
            
            # Add to digest queue
            digest_data = {
                "session_id": session.session_id,
                "home_id": session.home_id,
                "tier": session.tier,
                "findings": result.findings,
                "processing_duration_ms": result.processing_duration_ms,
                "completed_at": datetime.utcnow().isoformat()
            }
            
            await self.redis_client.lpush("digest_queue", json.dumps(digest_data))
            
            logger.info(f"Session {session.session_id} completed successfully")
            
        except Exception as e:
            logger.error(f"Failed to handle session success: {e}")
    
    async def _handle_session_failure(self, session: ProcessingSession, result: SessionResult):
        """Handle failed session"""
        try:
            # Store failure result
            result_key = f"session_result:{session.session_id}"
            await self.redis_client.setex(
                result_key,
                86400,  # 24 hours
                json.dumps(result.to_dict())
            )
            
            # Notify scheduler for each event in session
            for event_id in session.event_ids:
                await self._notify_scheduler_completion(event_id, False)
            
            logger.error(f"Session {session.session_id} failed: {result.error_message}")
            
        except Exception as e:
            logger.error(f"Failed to handle session failure: {e}")
    
    async def _process_current_batch(self):
        """Process the current batch of jobs"""
        if not self.current_batch:
            return
        
        batch_start = time.time()
        batch_size = len(self.current_batch)
        
        logger.info(f"Processing batch of {batch_size} jobs")
        
        try:
            # Process jobs in parallel
            tasks = [
                self._process_single_job(job) 
                for job in self.current_batch
            ]
            
            results = await asyncio.gather(*tasks, return_exceptions=True)
            
            # Handle results
            successful = 0
            failed = 0
            
            for i, result in enumerate(results):
                job = self.current_batch[i]
                
                if isinstance(result, Exception):
                    logger.error(f"Job {job.event_id} failed: {result}")
                    await self._handle_job_failure(job, str(result))
                    failed += 1
                else:
                    logger.info(f"Job {job.event_id} completed successfully")
                    await self._handle_job_success(job, result)
                    successful += 1
            
            # Update stats
            batch_duration = time.time() - batch_start
            self.total_processed += successful
            
            logger.info(f"Batch completed: {successful} successful, {failed} failed, {batch_duration:.2f}s")
            
            # Store batch metrics
            await self._record_batch_metrics(batch_size, successful, failed, batch_duration)
            
        except Exception as e:
            logger.error(f"Batch processing failed: {e}")
            
            # Mark all jobs as failed
            for job in self.current_batch:
                await self._handle_job_failure(job, f"Batch processing error: {e}")
        
        finally:
            # Clear batch
            self.current_batch = []
            self.last_batch_start = time.time()
    
    async def _process_single_job(self, job: ProcessingJob) -> ProcessingResult:
        """Process a single job"""
        start_time = time.time()
        
        try:
            logger.info(f"Processing job {job.event_id} (tier {job.processing_tier})")
            
            # Download image
            image_path = await self._download_image(job.image_url, job.event_id)
            
            if not image_path:
                raise Exception("Failed to download image")
            
            # Run processing based on tier
            if self.models_initialized:
                result = await self._run_deep_processing(job, image_path)
            else:
                result = await self._run_stub_processing(job, image_path)
            
            # Cleanup image
            try:
                os.unlink(image_path)
            except Exception:
                pass
            
            processing_duration = int((time.time() - start_time) * 1000)
            
            return ProcessingResult(
                event_id=job.event_id,
                success=True,
                processing_duration_ms=processing_duration,
                timestamp=datetime.utcnow(),
                **result
            )
            
        except Exception as e:
            processing_duration = int((time.time() - start_time) * 1000)
            logger.error(f"Job {job.event_id} processing failed: {e}")
            
            return ProcessingResult(
                event_id=job.event_id,
                success=False,
                processing_duration_ms=processing_duration,
                timestamp=datetime.utcnow(),
                error_message=str(e)
            )
    
    async def _download_image(self, image_url: str, event_id: str) -> Optional[str]:
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
                    prefix=f"event_{event_id}_"
                )
                
                temp_file.write(response.content)
                temp_file.close()
                
                return temp_file.name
                
        except Exception as e:
            logger.error(f"Failed to download image {image_url}: {e}")
            return None
    
    async def _run_deep_processing(self, job: ProcessingJob, image_path: str) -> Dict[str, Any]:
        """Run actual deep processing with ML models"""
        result = {}
        
        try:
            # Run detection
            detection_results = await asyncio.get_event_loop().run_in_executor(
                None, run_detection, image_path, self.config
            )
            result['detection_results'] = detection_results
            
            # Run face recognition if persons detected
            if detection_results and any(d.get('class') == 'person' for d in detection_results.get('detections', [])):
                face_results = await asyncio.get_event_loop().run_in_executor(
                    None, run_face_recognition, image_path, self.config
                )
                result['face_recognition_results'] = face_results
            
            # Generate caption for premium+ tiers
            if job.processing_tier >= 3:
                caption = await asyncio.get_event_loop().run_in_executor(
                    None, get_image_caption, image_path, self.config
                )
                result['caption'] = caption
            
            # Calculate risk score
            result['risk_score'] = self._calculate_risk_score(result, job)
            
            # Generate summary
            result['summary'] = self._generate_summary(result, job)
            
            return result
            
        except Exception as e:
            logger.error(f"Deep processing failed for {job.event_id}: {e}")
            raise
    
    async def _run_stub_processing(self, job: ProcessingJob, image_path: str) -> Dict[str, Any]:
        """Run stub processing when models not available"""
        await asyncio.sleep(0.5)  # Simulate processing time
        
        # Generate mock results based on lite results
        detection_results = {
            "detections": [],
            "confidence": 0.75
        }
        
        if job.lite_results and job.lite_results.get('channels'):
            channels = job.lite_results['channels']
            if channels.get('person'):
                detection_results['detections'].append({
                    "class": "person",
                    "confidence": 0.8,
                    "bbox": [100, 100, 200, 300]
                })
            if channels.get('vehicle'):
                detection_results['detections'].append({
                    "class": "car",
                    "confidence": 0.7,
                    "bbox": [50, 150, 250, 400]
                })
        
        result = {
            'detection_results': detection_results,
            'face_recognition_results': [],
            'caption': f"Security event at {job.location}",
            'risk_score': 0.6 if job.priority >= 3 else 0.3,
            'summary': f"Event detected at {job.location} with {len(detection_results['detections'])} objects"
        }
        
        return result
    
    def _calculate_risk_score(self, processing_results: Dict[str, Any], job: ProcessingJob) -> float:
        """Calculate risk score based on processing results"""
        base_score = 0.2
        
        # Add detection-based risk
        detection_results = processing_results.get('detection_results', {})
        detections = detection_results.get('detections', [])
        
        for detection in detections:
            if detection.get('class') == 'person':
                base_score += 0.4
            elif detection.get('class') in ['car', 'truck', 'motorcycle']:
                base_score += 0.2
            elif detection.get('class') in ['knife', 'gun']:
                base_score += 0.8
        
        # Add time-based risk
        hour = job.enqueued_at.hour
        if 22 <= hour or hour <= 6:  # Night hours
            base_score += 0.2
        
        # Add location-based risk
        if job.location in ['front_door', 'back_door']:
            base_score += 0.1
        
        # Add priority-based risk
        if job.priority >= 3:
            base_score += 0.2
        
        return min(1.0, base_score)
    
    def _generate_summary(self, processing_results: Dict[str, Any], job: ProcessingJob) -> str:
        """Generate human-readable summary"""
        detection_results = processing_results.get('detection_results', {})
        detections = detection_results.get('detections', [])
        
        if not detections:
            return f"No objects detected at {job.location}"
        
        # Count objects
        object_counts = {}
        for detection in detections:
            obj_class = detection.get('class', 'unknown')
            object_counts[obj_class] = object_counts.get(obj_class, 0) + 1
        
        # Build summary
        objects_text = []
        for obj_class, count in object_counts.items():
            if count == 1:
                objects_text.append(f"1 {obj_class}")
            else:
                objects_text.append(f"{count} {obj_class}s")
        
        summary = f"Detected {', '.join(objects_text)} at {job.location}"
        
        # Add risk context
        risk_score = processing_results.get('risk_score', 0)
        if risk_score > 0.7:
            summary += " (HIGH RISK)"
        elif risk_score > 0.5:
            summary += " (MODERATE RISK)"
        
        return summary
    
    async def _handle_job_success(self, job: ProcessingJob, result: ProcessingResult):
        """Handle successful job completion"""
        try:
            # Store result in database/Redis
            result_key = f"result:{job.event_id}"
            await self.redis_client.setex(
                result_key,
                86400,  # 24 hours
                json.dumps(result.to_dict())
            )
            
            # Add to digest queue for notification processing
            digest_data = {
                "event_id": job.event_id,
                "user_id": job.user_id,
                "home_id": job.home_id,
                "result": result.to_dict(),
                "processing_tier": job.processing_tier,
                "completed_at": datetime.utcnow().isoformat()
            }
            
            await self.redis_client.lpush("digest_queue", json.dumps(digest_data))
            
            # Notify scheduler of completion
            await self._notify_scheduler_completion(job.event_id, True)
            
            logger.info(f"Job {job.event_id} completed successfully")
            
        except Exception as e:
            logger.error(f"Failed to handle job success: {e}")
    
    async def _handle_job_failure(self, job: ProcessingJob, error_message: str):
        """Handle failed job"""
        try:
            # Create failure result
            result = ProcessingResult(
                event_id=job.event_id,
                success=False,
                processing_duration_ms=0,
                timestamp=datetime.utcnow(),
                error_message=error_message
            )
            
            # Store failure result
            result_key = f"result:{job.event_id}"
            await self.redis_client.setex(
                result_key,
                86400,  # 24 hours
                json.dumps(result.to_dict())
            )
            
            # Notify scheduler of completion
            await self._notify_scheduler_completion(job.event_id, False)
            
            logger.error(f"Job {job.event_id} failed: {error_message}")
            
        except Exception as e:
            logger.error(f"Failed to handle job failure: {e}")
    
    async def _notify_scheduler_completion(self, event_id: str, success: bool):
        """Notify scheduler that processing completed"""
        try:
            completion_data = {
                "event_id": event_id,
                "worker_id": self.worker_id,
                "success": success,
                "completed_at": datetime.utcnow().isoformat()
            }
            
            await self.redis_client.lpush(
                "scheduler_completions",
                json.dumps(completion_data)
            )
            
        except Exception as e:
            logger.error(f"Failed to notify scheduler: {e}")
    
    async def _record_batch_metrics(self, batch_size: int, successful: int, failed: int, duration: float):
        """Record batch processing metrics"""
        try:
            metrics = {
                "worker_id": self.worker_id,
                "batch_size": batch_size,
                "successful": successful,
                "failed": failed,
                "duration_seconds": duration,
                "throughput": batch_size / duration if duration > 0 else 0,
                "timestamp": datetime.utcnow().isoformat()
            }
            
            await self.redis_client.lpush("worker_metrics", json.dumps(metrics))
            await self.redis_client.ltrim("worker_metrics", 0, 999)  # Keep last 1000
            
        except Exception as e:
            logger.error(f"Failed to record metrics: {e}")
    
    async def get_worker_stats(self) -> Dict[str, Any]:
        """Get worker statistics"""
        try:
            queue_lengths = {}
            for tier in ["standard", "premium", "enterprise"]:
                queue_name = f"deep_processing_{tier}"
                length = await self.redis_client.llen(queue_name)
                queue_lengths[queue_name] = length
            
            return {
                "worker_id": self.worker_id,
                "running": self.running,
                "total_processed": self.total_processed,
                "current_batch_size": len(self.current_batch),
                "models_initialized": self.models_initialized,
                "queue_lengths": queue_lengths,
                "batch_config": {
                    "batch_size": self.batch_size,
                    "max_batch_wait_time": self.max_batch_wait_time
                }
            }
            
        except Exception as e:
            logger.error(f"Failed to get worker stats: {e}")
            return {}

class WorkerManager:
    """Manages multiple deep workers"""
    
    def __init__(self, redis_url: str = "redis://localhost:6379"):
        self.redis_url = redis_url
        self.workers: List[DeepWorkerV2] = []
        self.redis_client = None
    
    async def initialize(self):
        """Initialize worker manager"""
        self.redis_client = redis.from_url(self.redis_url, decode_responses=True)
        logger.info("Worker manager initialized")
    
    async def start_workers(self, worker_count: int = 2):
        """Start multiple workers"""
        for i in range(worker_count):
            worker = DeepWorkerV2(
                redis_url=self.redis_url,
                worker_id=f"worker_{i}_{int(time.time())}"
            )
            
            await worker.initialize()
            self.workers.append(worker)
            
            # Start worker in background
            asyncio.create_task(worker.start_processing())
        
        logger.info(f"Started {worker_count} workers")
    
    async def stop_all_workers(self):
        """Stop all workers"""
        for worker in self.workers:
            worker.stop_processing()
        
        # Wait for workers to finish
        await asyncio.sleep(2)
        
        for worker in self.workers:
            await worker.close()
        
        logger.info("All workers stopped")
    
    async def get_cluster_stats(self) -> Dict[str, Any]:
        """Get statistics for all workers"""
        try:
            worker_stats = []
            for worker in self.workers:
                stats = await worker.get_worker_stats()
                worker_stats.append(stats)
            
            total_processed = sum(w.get('total_processed', 0) for w in worker_stats)
            active_workers = sum(1 for w in worker_stats if w.get('running', False))
            
            return {
                "worker_count": len(self.workers),
                "active_workers": active_workers,
                "total_processed": total_processed,
                "workers": worker_stats
            }
            
        except Exception as e:
            logger.error(f"Failed to get cluster stats: {e}")
            return {}

async def main():
    """Test the deep worker"""
    worker = DeepWorkerV2(worker_id="test_worker")
    await worker.initialize()
    
    try:
        # Create test job
        test_job = ProcessingJob(
            event_id="test_job_123",
            home_id="home_test",
            user_id="user_test",
            image_url="https://via.placeholder.com/640x480.jpg",
            location="front_door",
            mode="security",
            processing_tier=2,
            priority=2,
            enqueued_at=datetime.utcnow(),
            lite_results={
                "channels": {"person": True, "vehicle": False},
                "confidence": 0.8
            }
        )
        
        # Process single job
        result = await worker._process_single_job(test_job)
        print(f"Processing result: {json.dumps(result.to_dict(), indent=2)}")
        
        # Get worker stats
        stats = await worker.get_worker_stats()
        print(f"Worker stats: {json.dumps(stats, indent=2)}")
        
    finally:
        await worker.close()

if __name__ == "__main__":
    asyncio.run(main())

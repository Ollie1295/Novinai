#!/usr/bin/env python3
"""
Simple GPU Worker for 3080 PC
This script runs on the 3080 box, processes camera feeds with YOLO, 
and sends events to the VPS gateway.
"""
import time
import cv2
import torch
import json
import hashlib
import socket
import httpx
from ultralytics import YOLO
from datetime import datetime
import threading
import queue
import redis
import numpy as np

# Configuration - EDIT THESE VALUES
VPS_IP = "YOUR_VPS_IP"  # Change to your actual VPS IP
DEVICE_MAC = "DEVICE_MAC"  # Change to your real MAC address (get with 'ip link show')
CAMERAS = [
    "rtsp://admin:password@192.168.1.100/stream1",  # Replace with your camera URLs
    # "rtsp://admin:password@192.168.1.101/stream2",
]

# YOLO model settings
REFLEX_MODEL = "yolov8n.pt"  # Fast, lightweight model for reflex detection
DEEP_MODEL = "yolov8s.pt"   # Slower, more accurate model for deep analysis
CONFIDENCE_THRESHOLD = 0.5

class GPUWorker:
    def __init__(self):
        self.device = "cuda" if torch.cuda.is_available() else "cpu"
        print(f"🚀 GPU Worker starting on device: {self.device}")
        
        # Load YOLO models
        self.reflex_model = YOLO(REFLEX_MODEL)
        self.deep_model = YOLO(DEEP_MODEL)
        
        # Move models to GPU
        if self.device == "cuda":
            self.reflex_model.to(self.device)
            self.deep_model.to(self.device)
        
        # HTTP client for VPS communication
        self.http_client = httpx.Client(timeout=30.0)
        
        # Processing queues
        self.deep_queue = queue.Queue(maxsize=10)
        
        # Start deep analysis worker thread
        self.deep_worker_thread = threading.Thread(target=self.deep_analysis_worker, daemon=True)
        self.deep_worker_thread.start()
        
    def get_device_token(self):
        """Register device and get JWT token"""
        try:
            response = self.http_client.post(f"http://{VPS_IP}:8000/device/register", 
                json={
                    "mac_address": DEVICE_MAC,
                    "device_type": "gpu_worker",
                    "capabilities": ["yolo_detection", "deep_analysis"]
                })
            if response.status_code == 200:
                return response.json()["token"]
            else:
                print(f"❌ Failed to register device: {response.text}")
                return None
        except Exception as e:
            print(f"❌ Device registration error: {e}")
            return None
    
    def process_frame_reflex(self, frame, camera_id):
        """Fast reflex detection - should complete in <120ms"""
        start_time = time.time()
        
        # Resize frame for speed
        height, width = frame.shape[:2]
        if width > 640:
            scale = 640 / width
            new_width = int(width * scale)
            new_height = int(height * scale)
            frame = cv2.resize(frame, (new_width, new_height))
        
        # YOLO detection
        results = self.reflex_model(frame, conf=CONFIDENCE_THRESHOLD, verbose=False)
        
        detections = []
        for r in results:
            boxes = r.boxes
            if boxes is not None:
                for box in boxes:
                    cls = int(box.cls)
                    conf = float(box.conf)
                    xyxy = box.xyxy[0].cpu().numpy()
                    
                    detection = {
                        "class_id": cls,
                        "class_name": self.reflex_model.names[cls],
                        "confidence": conf,
                        "bbox": xyxy.tolist(),
                        "timestamp": datetime.now().isoformat()
                    }
                    detections.append(detection)
        
        processing_time = (time.time() - start_time) * 1000
        print(f"⚡ Reflex detection completed in {processing_time:.1f}ms, found {len(detections)} objects")
        
        return detections, processing_time
    
    def should_trigger_deep_analysis(self, detections):
        """Determine if we should trigger deep analysis"""
        priority_classes = ["person", "car", "truck", "bicycle", "motorcycle"]
        
        for detection in detections:
            if detection["class_name"] in priority_classes:
                return True
            if detection["confidence"] > 0.8:
                return True
        
        return len(detections) > 3
    
    def send_reflex_event(self, camera_id, detections, processing_time, token):
        """Send reflex event to VPS"""
        event_data = {
            "device_mac": DEVICE_MAC,
            "camera_id": camera_id,
            "event_type": "reflex",
            "detections": detections,
            "processing_time_ms": processing_time,
            "timestamp": datetime.now().isoformat(),
            "content_hash": hashlib.md5(json.dumps(detections, sort_keys=True).encode()).hexdigest()
        }
        
        try:
            response = self.http_client.post(
                f"http://{VPS_IP}:8000/events/reflex",
                json=event_data,
                headers={"Authorization": f"Bearer {token}"}
            )
            if response.status_code == 200:
                print(f"✅ Reflex event sent for camera {camera_id}")
                return True
            else:
                print(f"❌ Failed to send reflex event: {response.text}")
                return False
        except Exception as e:
            print(f"❌ Error sending reflex event: {e}")
            return False
    
    def queue_deep_analysis(self, camera_id, frame, detections, token):
        """Queue frame for deep analysis"""
        try:
            _, buffer = cv2.imencode('.jpg', frame, [cv2.IMWRITE_JPEG_QUALITY, 85])
            frame_data = buffer.tobytes()
            
            deep_job = {
                "camera_id": camera_id,
                "frame_data": frame_data,
                "reflex_detections": detections,
                "timestamp": datetime.now().isoformat(),
                "token": token
            }
            
            self.deep_queue.put(deep_job, block=False)
            print(f"🔄 Queued deep analysis job for camera {camera_id}")
        except queue.Full:
            print(f"⚠️ Deep analysis queue full, dropping frame from camera {camera_id}")
    
    def deep_analysis_worker(self):
        """Worker thread for deep analysis"""
        while True:
            try:
                job = self.deep_queue.get(timeout=1.0)
                self.process_deep_analysis(job)
                self.deep_queue.task_done()
            except queue.Empty:
                continue
            except Exception as e:
                print(f"❌ Deep analysis worker error: {e}")
    
    def process_deep_analysis(self, job):
        """Process deep analysis with heavier model"""
        start_time = time.time()
        
        frame_data = job["frame_data"]
        frame = cv2.imdecode(np.frombuffer(frame_data, np.uint8), cv2.IMREAD_COLOR)
        
        results = self.deep_model(frame, conf=0.3, verbose=False)
        
        detections = []
        for r in results:
            boxes = r.boxes
            if boxes is not None:
                for box in boxes:
                    cls = int(box.cls)
                    conf = float(box.conf)
                    xyxy = box.xyxy[0].cpu().numpy()
                    
                    detection = {
                        "class_id": cls,
                        "class_name": self.deep_model.names[cls],
                        "confidence": conf,
                        "bbox": xyxy.tolist(),
                        "timestamp": datetime.now().isoformat()
                    }
                    detections.append(detection)
        
        processing_time = (time.time() - start_time) * 1000
        print(f"🔍 Deep analysis completed in {processing_time:.1f}ms, found {len(detections)} objects")
        
        narrative = self.generate_narrative(detections)
        self.send_deep_event(job["camera_id"], detections, narrative, processing_time, job["token"])
    
    def generate_narrative(self, detections):
        """Generate human-readable narrative from detections"""
        if not detections:
            return "No objects detected in deep analysis."
        
        class_counts = {}
        for det in detections:
            class_name = det["class_name"]
            class_counts[class_name] = class_counts.get(class_name, 0) + 1
        
        parts = []
        for class_name, count in class_counts.items():
            if count == 1:
                parts.append(f"1 {class_name}")
            else:
                parts.append(f"{count} {class_name}s")
        
        if len(parts) == 1:
            narrative = f"Detected {parts[0]}"
        elif len(parts) == 2:
            narrative = f"Detected {parts[0]} and {parts[1]}"
        else:
            narrative = f"Detected {', '.join(parts[:-1])} and {parts[-1]}"
        
        high_conf_objects = [d for d in detections if d["confidence"] > 0.8]
        if high_conf_objects:
            narrative += f" (high confidence: {len(high_conf_objects)} objects)"
        
        return narrative
    
    def send_deep_event(self, camera_id, detections, narrative, processing_time, token):
        """Send deep analysis event to VPS"""
        event_data = {
            "device_mac": DEVICE_MAC,
            "camera_id": camera_id,
            "event_type": "deep",
            "detections": detections,
            "narrative": narrative,
            "processing_time_ms": processing_time,
            "timestamp": datetime.now().isoformat(),
            "content_hash": hashlib.md5(json.dumps(detections, sort_keys=True).encode()).hexdigest()
        }
        
        try:
            response = self.http_client.post(
                f"http://{VPS_IP}:8000/events/deep",
                json=event_data,
                headers={"Authorization": f"Bearer {token}"}
            )
            if response.status_code == 200:
                print(f"✅ Deep event sent for camera {camera_id}: {narrative}")
            else:
                print(f"❌ Failed to send deep event: {response.text}")
        except Exception as e:
            print(f"❌ Error sending deep event: {e}")
    
    def process_camera(self, camera_url, camera_id, token):
        """Process single camera feed"""
        print(f"📹 Starting camera {camera_id}: {camera_url}")
        
        cap = cv2.VideoCapture(camera_url)
        if not cap.isOpened():
            print(f"❌ Failed to open camera {camera_id}")
            return
        
        frame_skip = 5
        frame_count = 0
        
        while True:
            try:
                ret, frame = cap.read()
                if not ret:
                    print(f"❌ Failed to read from camera {camera_id}")
                    break
                
                frame_count += 1
                if frame_count % frame_skip != 0:
                    continue
                
                detections, processing_time = self.process_frame_reflex(frame, camera_id)
                
                if detections:
                    self.send_reflex_event(camera_id, detections, processing_time, token)
                    
                    if self.should_trigger_deep_analysis(detections):
                        self.queue_deep_analysis(camera_id, frame, detections, token)
                
                time.sleep(0.1)
                
            except KeyboardInterrupt:
                print(f"🛑 Stopping camera {camera_id}")
                break
            except Exception as e:
                print(f"❌ Error processing camera {camera_id}: {e}")
                time.sleep(1)
        
        cap.release()
        print(f"📹 Camera {camera_id} stopped")
    
    def run(self):
        """Main execution loop"""
        print("🚀 Getting device token...")
        token = self.get_device_token()
        if not token:
            print("❌ Could not get device token, exiting")
            return
        print("✅ Device token received")
        
        camera_threads = []
        for i, camera_url in enumerate(CAMERAS):
            thread = threading.Thread(
                target=self.process_camera,
                args=(camera_url, i, token),
                daemon=True
            )
            thread.start()
            camera_threads.append(thread)
        
        print(f"📹 Started {len(camera_threads)} camera threads")
        
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            print("\n🛑 Shutting down GPU worker...")
            
        for thread in camera_threads:
            thread.join(timeout=5)
        
        print("✅ GPU worker shut down")

if __name__ == "__main__":
    print("🔥 Starting AI Security GPU Worker")
    print(f"CUDA Available: {torch.cuda.is_available()}")
    if torch.cuda.is_available():
        print(f"CUDA Device: {torch.cuda.get_device_name()}")
    
    if VPS_IP == "YOUR_VPS_IP":
        print("❌ ERROR: Please configure VPS_IP in the script")
        exit(1)
    
    if DEVICE_MAC == "DEVICE_MAC":
        print("❌ ERROR: Please configure DEVICE_MAC in the script")
        print("Run 'ip link show' to get your MAC address")
        exit(1)
    
    if not CAMERAS or CAMERAS[0].startswith("rtsp://admin:password@"):
        print("❌ ERROR: Please configure CAMERAS with your real RTSP URLs")
        exit(1)
    
    worker = GPUWorker()
    worker.run()

#!/usr/bin/env python3
"""
Ollama Event Summarizer for AI Security Pipeline
Generates natural language summaries of detection events and decisions
"""

import asyncio
import json
import logging
import time
from typing import Dict, List, Optional, Any
from datetime import datetime
import aiohttp
import subprocess

class OllamaSummarizer:
    """Async Ollama client for generating event summaries"""
    
    def __init__(self, 
                 model: str = "llama3.2:1b",
                 ollama_url: str = "http://localhost:11434",
                 max_retries: int = 3,
                 timeout: int = 10):
        self.model = model
        self.ollama_url = ollama_url
        self.max_retries = max_retries
        self.timeout = timeout
        self.logger = logging.getLogger(__name__)
        
        # Performance tracking
        self.summary_count = 0
        self.total_time = 0.0
        self.errors = 0
    
    async def ensure_ollama_running(self) -> bool:
        """Ensure Ollama service is running"""
        try:
            async with aiohttp.ClientSession(timeout=aiohttp.ClientTimeout(total=5)) as session:
                async with session.get(f"{self.ollama_url}/api/tags") as response:
                    if response.status == 200:
                        return True
        except Exception as e:
            self.logger.warning(f"Ollama not running, attempting to start: {e}")
            
        # Try to start Ollama service
        try:
            subprocess.Popen(["ollama", "serve"], 
                           stdout=subprocess.DEVNULL, 
                           stderr=subprocess.DEVNULL)
            await asyncio.sleep(2)  # Give it time to start
            return True
        except Exception as e:
            self.logger.error(f"Failed to start Ollama: {e}")
            return False
    
    async def generate_summary(self, event_data: Dict[str, Any]) -> Optional[str]:
        """Generate a natural language summary of the security event"""
        start_time = time.time()
        
        try:
            if not await self.ensure_ollama_running():
                self.errors += 1
                return "⚠️ Summary unavailable (Ollama offline)"
            
            # Create context-aware prompt
            prompt = self._create_summary_prompt(event_data)
            
            # Generate summary with retries
            for attempt in range(self.max_retries):
                try:
                    summary = await self._call_ollama(prompt)
                    if summary:
                        # Track performance
                        elapsed = time.time() - start_time
                        self.summary_count += 1
                        self.total_time += elapsed
                        
                        return summary.strip()
                
                except Exception as e:
                    self.logger.warning(f"Ollama attempt {attempt + 1} failed: {e}")
                    if attempt < self.max_retries - 1:
                        await asyncio.sleep(0.5)  # Brief delay before retry
            
            self.errors += 1
            return "⚠️ Summary generation failed"
            
        except Exception as e:
            self.logger.error(f"Summary generation error: {e}")
            self.errors += 1
            return "⚠️ Summary error"
    
    def _create_summary_prompt(self, event_data: Dict[str, Any]) -> str:
        """Create an intelligent prompt based on event data"""
        
        # Extract key information
        timestamp = event_data.get('timestamp', 'Unknown time')
        camera_id = event_data.get('camera_id', 'Unknown camera')
        decision = event_data.get('decision', 'Unknown')
        confidence = event_data.get('confidence', 0.0)
        detections = event_data.get('detections', [])
        motion_detected = event_data.get('motion_detected', False)
        
        # Build context
        detection_list = []
        for det in detections:
            obj_class = det.get('class', 'object')
            obj_conf = det.get('confidence', 0.0)
            detection_list.append(f"{obj_class} ({obj_conf:.1%} confidence)")
        
        detections_text = ", ".join(detection_list) if detection_list else "no objects"
        motion_text = "with motion detected" if motion_detected else "without motion"
        
        # Determine event severity for appropriate tone
        is_threat = decision.lower() in ['alert', 'threat', 'suspicious']
        severity = "high priority" if is_threat else "routine"
        
        prompt = f"""
You are an AI security analyst. Summarize this security event in 1-2 concise sentences.
Focus on what happened, where, and the significance.

Event Details:
- Time: {timestamp}
- Camera: {camera_id}
- Decision: {decision} ({confidence:.1%} confidence)
- Detections: {detections_text}
- Motion: {motion_text}
- Severity: {severity}

Generate a professional, clear summary suitable for security personnel:
"""
        
        return prompt
    
    async def _call_ollama(self, prompt: str) -> Optional[str]:
        """Make async call to Ollama API"""
        
        payload = {
            "model": self.model,
            "prompt": prompt,
            "stream": False,
            "options": {
                "temperature": 0.1,  # Low temperature for consistent summaries
                "top_p": 0.9,
                "num_ctx": 2048,     # Context window
                "stop": ["\n\n", "---", "END"]  # Stop tokens
            }
        }
        
        timeout = aiohttp.ClientTimeout(total=self.timeout)
        
        async with aiohttp.ClientSession(timeout=timeout) as session:
            async with session.post(f"{self.ollama_url}/api/generate", 
                                  json=payload) as response:
                
                if response.status == 200:
                    result = await response.json()
                    return result.get('response', '').strip()
                else:
                    error_text = await response.text()
                    self.logger.error(f"Ollama API error {response.status}: {error_text}")
                    return None
    
    def get_performance_stats(self) -> Dict[str, float]:
        """Get performance statistics"""
        avg_time = self.total_time / max(self.summary_count, 1)
        error_rate = self.errors / max(self.summary_count + self.errors, 1)
        
        return {
            "summaries_generated": self.summary_count,
            "average_time_ms": avg_time * 1000,
            "error_rate": error_rate,
            "total_errors": self.errors
        }

class EventSummaryManager:
    """Manages async event summarization with queue and batching"""
    
    def __init__(self, 
                 max_queue_size: int = 100,
                 batch_size: int = 5,
                 batch_timeout: float = 2.0):
        
        self.summarizer = OllamaSummarizer()
        self.max_queue_size = max_queue_size
        self.batch_size = batch_size
        self.batch_timeout = batch_timeout
        
        # Async queue for pending summaries
        self.summary_queue = asyncio.Queue(maxsize=max_queue_size)
        self.running = False
        self.logger = logging.getLogger(__name__)
    
    async def start(self):
        """Start the background summarization worker"""
        self.running = True
        asyncio.create_task(self._summary_worker())
        self.logger.info("Event summary manager started")
    
    async def stop(self):
        """Stop the background worker"""
        self.running = False
        self.logger.info("Event summary manager stopped")
    
    async def request_summary(self, event_data: Dict[str, Any], 
                            callback: callable = None) -> str:
        """Request an async summary for an event"""
        
        try:
            # Add to queue with callback
            await self.summary_queue.put({
                'event_data': event_data,
                'callback': callback,
                'timestamp': time.time()
            })
            
            return "🔄 Summary pending..."
            
        except asyncio.QueueFull:
            self.logger.warning("Summary queue full, skipping summarization")
            return "⚠️ Summary queue full"
    
    async def _summary_worker(self):
        """Background worker that processes summary requests"""
        
        while self.running:
            try:
                # Collect batch of requests
                batch = []
                
                # Get first item (blocking)
                try:
                    first_item = await asyncio.wait_for(
                        self.summary_queue.get(), 
                        timeout=self.batch_timeout
                    )
                    batch.append(first_item)
                except asyncio.TimeoutError:
                    continue
                
                # Get additional items (non-blocking)
                while len(batch) < self.batch_size and not self.summary_queue.empty():
                    try:
                        item = self.summary_queue.get_nowait()
                        batch.append(item)
                    except asyncio.QueueEmpty:
                        break
                
                # Process batch
                await self._process_batch(batch)
                
            except Exception as e:
                self.logger.error(f"Summary worker error: {e}")
                await asyncio.sleep(1)
    
    async def _process_batch(self, batch: List[Dict[str, Any]]):
        """Process a batch of summary requests"""
        
        tasks = []
        for item in batch:
            task = asyncio.create_task(
                self._process_single_summary(item)
            )
            tasks.append(task)
        
        # Process all summaries concurrently
        await asyncio.gather(*tasks, return_exceptions=True)
    
    async def _process_single_summary(self, item: Dict[str, Any]):
        """Process a single summary request"""
        
        try:
            event_data = item['event_data']
            callback = item.get('callback')
            
            # Generate summary
            summary = await self.summarizer.generate_summary(event_data)
            
            # Call callback if provided
            if callback and summary:
                try:
                    if asyncio.iscoroutinefunction(callback):
                        await callback(event_data, summary)
                    else:
                        callback(event_data, summary)
                except Exception as e:
                    self.logger.error(f"Summary callback error: {e}")
        
        except Exception as e:
            self.logger.error(f"Single summary processing error: {e}")

# Global instance for easy import
summary_manager = EventSummaryManager()

async def main():
    """Test the summarizer"""
    
    logging.basicConfig(level=logging.INFO)
    
    # Test event data
    test_event = {
        "timestamp": datetime.now().isoformat(),
        "camera_id": "CAM_001_ENTRANCE",
        "decision": "ALERT",
        "confidence": 0.95,
        "detections": [
            {"class": "person", "confidence": 0.92, "bbox": [100, 100, 200, 300]},
            {"class": "backpack", "confidence": 0.78, "bbox": [150, 150, 180, 200]}
        ],
        "motion_detected": True,
        "processing_time_ms": 87
    }
    
    summarizer = OllamaSummarizer()
    
    print("🤖 Testing Ollama Event Summarization...")
    print(f"Event: {test_event}")
    print("\nGenerating summary...")
    
    summary = await summarizer.generate_summary(test_event)
    print(f"\n📝 Summary: {summary}")
    
    stats = summarizer.get_performance_stats()
    print(f"\n📊 Performance: {stats}")

if __name__ == "__main__":
    asyncio.run(main())

#!/usr/bin/env python3
"""
Integration Test Script - Tests Novin + GPU Worker integration
"""

import asyncio
import subprocess
import time
import httpx
import json
import base64
import cv2
import numpy as np

async def test_novin_server():
    """Test Novin server availability"""
    print("🧪 Testing Novin server on port 8001...")
    
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get("http://localhost:8001/health", timeout=5)
            if response.status_code == 200:
                print("✅ Novin server is healthy")
                return True
            else:
                print(f"❌ Novin server unhealthy: {response.status_code}")
                return False
    except Exception as e:
        print(f"❌ Novin server not available: {e}")
        return False

def check_gpu():
    """Check GPU availability"""
    print("🔍 Checking GPU...")
    
    try:
        import torch
        if torch.cuda.is_available():
            print(f"✅ CUDA: {torch.cuda.get_device_name()}")
            return True
        else:
            print("⚠️ CUDA not available")
            return False
    except ImportError:
        print("❌ PyTorch not installed")
        return False

def test_yolo():
    """Test YOLO models"""
    print("🧠 Testing YOLO...")
    
    try:
        from ultralytics import YOLO
        model = YOLO('yolov8n.pt')
        print("✅ YOLO models work")
        return True
    except Exception as e:
        print(f"❌ YOLO test failed: {e}")
        return False

async def main():
    print("🚀 NOVIN + GPU WORKER INTEGRATION TEST")
    print("=" * 50)
    
    # Test components
    gpu_ok = check_gpu()
    yolo_ok = test_yolo()
    novin_ok = await test_novin_server()
    
    print("\n📊 RESULTS:")
    print(f"GPU/CUDA: {'✅' if gpu_ok else '❌'}")
    print(f"YOLO: {'✅' if yolo_ok else '❌'}")
    print(f"Novin Server: {'✅' if novin_ok else '❌'}")
    
    all_ok = gpu_ok and yolo_ok and novin_ok
    
    if all_ok:
        print("\n🎯 ✅ INTEGRATION READY!")
        print("Your enhanced pipeline is ready for brutal-fast AI processing!")
    else:
        print("\n⚠️ Issues detected:")
        if not novin_ok:
            print("   - Start Novin: cd /mnt/c/novin && python server.py")
        if not gpu_ok:
            print("   - Install CUDA/PyTorch")
        if not yolo_ok:
            print("   - Install ultralytics")

if __name__ == "__main__":
    asyncio.run(main())

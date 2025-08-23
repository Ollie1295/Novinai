#!/usr/bin/env python3
"""
Configuration helper for GPU Worker
Run this to easily set your VPS IP and camera URLs
"""
import re
import sys

def update_config():
    print("🔧 GPU Worker Configuration")
    print("=" * 40)
    
    # Get VPS IP
    vps_ip = input("Enter your VPS IP address: ").strip()
    if not vps_ip:
        print("❌ VPS IP is required")
        sys.exit(1)
    
    # Validate IP format (basic)
    ip_pattern = r'^(\d{1,3}\.){3}\d{1,3}$'
    if not re.match(ip_pattern, vps_ip):
        print("❌ Invalid IP format")
        sys.exit(1)
    
    # Get camera URL (at least one required)
    print("\nEnter camera RTSP URLs (press Enter with empty line to finish):")
    cameras = []
    i = 1
    while True:
        camera_url = input(f"Camera {i} RTSP URL: ").strip()
        if not camera_url:
            break
        
        zone = input(f"Camera {i} zone name (e.g., front_door, backyard): ").strip() or f"zone_{i}"
        
        cameras.append({
            "device_id": f"cam_{i}",
            "rtsp_url": camera_url,
            "zone": zone
        })
        i += 1
    
    if not cameras:
        print("❌ At least one camera is required")
        sys.exit(1)
    
    # Read current file
    with open('gpu_worker_simple.py', 'r') as f:
        content = f.read()
    
    # Update VPS IP
    content = re.sub(
        r'VPS_BASE_URL = "http://YOUR_VPS_IP:8000"',
        f'VPS_BASE_URL = "http://{vps_ip}:8000"',
        content
    )
    
    # Update camera config
    cameras_str = "[\n"
    for cam in cameras:
        cameras_str += f'''    {{
        "device_id": "{cam['device_id']}",
        "rtsp_url": "{cam['rtsp_url']}",
        "zone": "{cam['zone']}"
    }},
'''
    cameras_str = cameras_str.rstrip(',\n') + '\n]'
    
    # Replace camera config
    camera_pattern = r'CAMERA_CONFIGS = \[.*?\]'
    content = re.sub(camera_pattern, f'CAMERA_CONFIGS = {cameras_str}', content, flags=re.DOTALL)
    
    # Write back
    with open('gpu_worker_simple.py', 'w') as f:
        f.write(content)
    
    print("\n✅ Configuration updated!")
    print(f"   VPS IP: {vps_ip}")
    print(f"   Cameras configured: {len(cameras)}")
    
    for i, cam in enumerate(cameras, 1):
        print(f"     Camera {i}: {cam['zone']} -> {cam['rtsp_url']}")
    
    print("\nYou can now run: ./start_gpu_worker.sh")

if __name__ == "__main__":
    update_config()

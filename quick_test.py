#!/usr/bin/env python3
"""
Quick VPS connection test without external dependencies
Uses only built-in Python libraries
"""

import socket
import sys
import os

def test_port_connectivity(host, port, service_name):
    """Test if a port is reachable"""
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(5)
        result = sock.connect_ex((host, port))
        sock.close()
        
        if result == 0:
            print(f"✅ {service_name} port {port} is reachable")
            return True
        else:
            print(f"❌ {service_name} port {port} is not reachable")
            return False
    except Exception as e:
        print(f"❌ {service_name} connection test failed: {e}")
        return False

def main():
    """Test basic connectivity to VPS services"""
    print("🚀 Quick VPS Connectivity Test")
    print("=" * 40)
    
    vps_ip = "95.179.193.224"
    tests_passed = 0
    total_tests = 2
    
    # Test Redis port
    if test_port_connectivity(vps_ip, 6379, "Redis"):
        tests_passed += 1
    
    # Test PostgreSQL port
    if test_port_connectivity(vps_ip, 5432, "PostgreSQL"):
        tests_passed += 1
    
    print()
    print("=" * 40)
    print(f"📊 Connectivity Results: {tests_passed}/{total_tests} ports reachable")
    
    if tests_passed == total_tests:
        print("🎉 Basic connectivity successful!")
        print("Next: Install redis and psycopg2-binary to test full functionality")
        print("pip install redis psycopg2-binary")
        return 0
    else:
        print("❌ Some ports are not reachable")
        print("Check VPS firewall and service status")
        return 1

if __name__ == "__main__":
    sys.exit(main())

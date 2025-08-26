#!/usr/bin/env python3
"""
Test VPS connection from PC Deep Worker
Validates Redis and PostgreSQL connectivity before starting the worker
"""

import os
import sys
import redis
import psycopg2
from urllib.parse import urlparse

def test_redis_connection():
    """Test Redis connection"""
    try:
        redis_url = os.getenv("REDIS_URL", "redis://:novin_redis_2024@95.179.193.224:6379/0")
        print(f"🔍 Testing Redis connection: {redis_url}")
        
        r = redis.from_url(redis_url)
        result = r.ping()
        
        if result:
            print("✅ Redis connection successful")
            
            # Test basic operations
            r.set("test_key", "test_value", ex=10)
            value = r.get("test_key")
            if value == b"test_value":
                print("✅ Redis read/write operations working")
            else:
                print("❌ Redis read/write test failed")
                
            return True
        else:
            print("❌ Redis ping failed")
            return False
            
    except Exception as e:
        print(f"❌ Redis connection failed: {e}")
        return False

def test_postgres_connection():
    """Test PostgreSQL connection"""
    try:
        pg_dsn = os.getenv("PG_DSN", "postgres://novin:novin@95.179.193.224:5432/novin")
        print(f"🔍 Testing PostgreSQL connection: {pg_dsn}")
        
        # Parse connection string
        parsed = urlparse(pg_dsn)
        
        conn = psycopg2.connect(
            host=parsed.hostname,
            port=parsed.port or 5432,
            database=parsed.path[1:],  # Remove leading slash
            user=parsed.username,
            password=parsed.password
        )
        
        cursor = conn.cursor()
        cursor.execute("SELECT version();")
        version = cursor.fetchone()
        
        print(f"✅ PostgreSQL connection successful")
        print(f"   Version: {version[0][:50]}...")
        
        # Test basic operations
        cursor.execute("SELECT NOW();")
        timestamp = cursor.fetchone()
        print(f"   Current time: {timestamp[0]}")
        
        cursor.close()
        conn.close()
        return True
        
    except Exception as e:
        print(f"❌ PostgreSQL connection failed: {e}")
        return False

def test_stream_operations():
    """Test Redis stream operations for deep worker"""
    try:
        redis_url = os.getenv("REDIS_URL", "redis://:novin_redis_2024@95.179.193.224:6379/0")
        r = redis.from_url(redis_url)
        
        print("🔍 Testing Redis stream operations...")
        
        # Test writing to deep.jobs stream
        test_job = {
            "job_id": "test_job_123",
            "session_id": "test_session",
            "user_id": "test_user",
            "data": '{"test": true}'
        }
        
        stream_id = r.xadd("stream:deep.jobs.test", test_job)
        print(f"✅ Successfully wrote to test stream: {stream_id}")
        
        # Test reading from stream
        messages = r.xread({"stream:deep.jobs.test": "0"}, count=1)
        if messages:
            print("✅ Successfully read from test stream")
            
            # Cleanup test stream
            r.delete("stream:deep.jobs.test")
            print("✅ Test stream cleaned up")
        else:
            print("❌ Failed to read from test stream")
            return False
            
        return True
        
    except Exception as e:
        print(f"❌ Stream operations test failed: {e}")
        return False

def main():
    """Run all connection tests"""
    print("🚀 Testing VPS connections for PC Deep Worker")
    print("=" * 50)
    
    # Load environment variables from .env.pc if available
    env_file = ".env.pc"
    if os.path.exists(env_file):
        print(f"📄 Loading environment from {env_file}")
        with open(env_file, 'r') as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#') and '=' in line:
                    key, value = line.split('=', 1)
                    os.environ[key] = value
    
    tests_passed = 0
    total_tests = 3
    
    # Test Redis
    if test_redis_connection():
        tests_passed += 1
    
    print()
    
    # Test PostgreSQL
    if test_postgres_connection():
        tests_passed += 1
    
    print()
    
    # Test stream operations
    if test_stream_operations():
        tests_passed += 1
    
    print()
    print("=" * 50)
    print(f"📊 Test Results: {tests_passed}/{total_tests} passed")
    
    if tests_passed == total_tests:
        print("🎉 All tests passed! PC Deep Worker is ready to connect to VPS")
        return 0
    else:
        print("❌ Some tests failed. Check VPS configuration and network connectivity")
        print("\n🔧 Troubleshooting steps:")
        print("1. Run vps_setup_commands.sh on your VPS")
        print("2. Check firewall settings (ports 6379, 5432)")
        print("3. Verify your PC can reach 95.179.193.224")
        print("4. Check Redis and PostgreSQL service status on VPS")
        return 1

if __name__ == "__main__":
    sys.exit(main())

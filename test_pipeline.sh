#!/bin/bash

echo "🧪 TESTING NOVINAI PIPELINE"
echo "=========================="

# Test 1: Health Check
echo "📊 Testing health endpoint..."
curl -s http://127.0.0.1:3000/health | jq .
echo

# Test 2: Submit Event
echo "📡 Submitting test event..."
curl -X POST http://127.0.0.1:3000/api/events \
  -H "Content-Type: application/json" \
  -d '{
    "sensor_id": "front_door_camera",
    "data": "base64_encoded_image_data_here",
    "user_id": "user123",
    "home_id": "home456",
    "api_key": "test_api_key",
    "subscription_tier": "Premium"
  }' | jq .
echo

# Test 3: Get Events
echo "📚 Getting events for home..."
curl -s http://127.0.0.1:3000/api/events/home456 | jq .
echo

echo "✅ PIPELINE TEST COMPLETE!"

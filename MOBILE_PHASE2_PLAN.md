# 📱 Mobile Phase 2: On-Device AI Integration

## **7-Day Implementation Plan**

### **🎯 Non-Negotiables (Scope Guard)**
- **Lite on every event** runs on device when possible; server has CPU fallback
- **Never do face/re-ID/plates** on device. Presence + basic classes only
- **Silent push → download thumbnail ≤2 MB → ≤384px → run ≤300ms inference → POST channels/explainer → done**
- **If phone can't run** (sleep/thermal/battery), server lite still runs so UX never breaks

## **Day 1-2: Backend Infrastructure**

### **Database Setup**
```bash
# Run on VPS PostgreSQL
psql -U novin -d novin -f mobile_backend_setup.sql
```

### **API Endpoints Deployment**
```bash
# Deploy mobile API server
python mobile_api_endpoints.py
# Runs on port 8001
```

### **Push Service Configuration**
```bash
# Set environment variables
export APNS_TEAM="YOUR_TEAM_ID"
export APNS_KEY_ID="YOUR_KEY_ID"  
export APNS_BUNDLE="uk.novin.security"
export APNS_P8_PATH="/path/to/AuthKey.p8"
export FCM_KEY="YOUR_FCM_SERVER_KEY"

# Test push service
python push_service.py
```

## **Day 3-4: Event Fan-out Integration**

### **Fan-out Service Deployment**
```bash
# Deploy event fan-out with mobile push
python event_fanout_service.py
```

### **Integration with Event Reception**
- Modify existing event reception server
- Add mobile device lookup
- Implement silent push on new camera events
- Add 3-5s timeout with server fallback

## **Day 5-6: Mobile Model Conversion**

### **Convert Models for Mobile**
```bash
# Generate CoreML and TensorFlow Lite models
python mobile_model_conversion.py
```

**Output:**
- `mobile_models/yolo_lite_320.mlmodel` (iOS)
- `mobile_models/yolo_lite_384.mlmodel` (iOS)
- `mobile_models/yolo_lite_320.tflite` (Android)
- `mobile_models/yolo_lite_384.tflite` (Android)

### **Model Distribution**
- Upload models to R2/S3 storage
- Create signed download URLs
- Implement model versioning

## **Day 7: Testing & Validation**

### **Acceptance Tests**

**1. Device Registration**
```bash
curl -X POST http://your-vps:8001/mobile/register \
  -H "Authorization: Bearer test_token" \
  -H "Content-Type: application/json" \
  -d '{"device_id":"test-device-123","platform":"ios","push_token":"test_token"}'
```

**2. Silent Push Flow**
- Trigger camera event
- Verify silent push sent to registered devices
- Confirm server fallback if no mobile response within 5s

**3. On-Device Processing**
- Download thumbnail (≤2MB, ≤384px)
- Run inference (≤300ms)
- POST results to `/events` endpoint

**4. Battery/Thermal Policies**
- Test with low battery (≤20%)
- Test with thermal throttling
- Verify server fallback activation

## **📱 Mobile App Scaffolds**

### **iOS App Structure**
```
NovinSecurity/
├── Models/
│   ├── YOLOLite.mlmodel
│   └── ModelManager.swift
├── Services/
│   ├── PushNotificationService.swift
│   ├── InferenceService.swift
│   └── APIService.swift
├── Background/
│   └── BackgroundTaskManager.swift
└── Info.plist (background processing)
```

### **Android App Structure**
```
app/src/main/
├── assets/
│   └── yolo_lite.tflite
├── java/uk/novin/security/
│   ├── services/
│   │   ├── PushService.java
│   │   ├── InferenceService.java
│   │   └── APIService.java
│   └── background/
│       └── BackgroundProcessor.java
└── AndroidManifest.xml (background permissions)
```

## **🔧 Technical Implementation**

### **iOS CoreML Integration**
```swift
import CoreML
import Vision

class InferenceService {
    private let model: VNCoreMLModel
    
    func processImage(_ image: UIImage) async -> InferenceResult {
        // Resize to 384x384
        // Run CoreML inference
        // Extract basic classes (person, vehicle, etc.)
        // Return channels + explainer
    }
}
```

### **Android TensorFlow Lite Integration**
```java
import org.tensorflow.lite.Interpreter;

public class InferenceService {
    private Interpreter tflite;
    
    public InferenceResult processImage(Bitmap image) {
        // Resize to 384x384
        // Run TFLite inference  
        // Extract basic classes
        // Return channels + explainer
    }
}
```

## **📊 Performance Targets**

| Metric | Target | Fallback |
|--------|--------|----------|
| **Inference Time** | ≤300ms | Server lite |
| **Download Size** | ≤2MB | Skip processing |
| **Battery Threshold** | >20% | Server fallback |
| **Thermal State** | Normal/Fair | Server fallback |
| **Push Delivery** | <2s | Server fallback |

## **🚀 Deployment Checklist**

- [ ] **Database tables** created (mobile_devices, mobile_prefs)
- [ ] **API endpoints** deployed and tested (/mobile/register, /mobile/ack, /events)
- [ ] **Push service** configured (APNs + FCM credentials)
- [ ] **Event fan-out** integrated with camera events
- [ ] **Mobile models** converted and distributed (CoreML + TFLite)
- [ ] **iOS app** scaffold with CoreML inference
- [ ] **Android app** scaffold with TFLite inference
- [ ] **Silent push flow** tested end-to-end
- [ ] **Server fallback** validated for all failure modes
- [ ] **Battery/thermal policies** implemented and tested

## **🎯 Success Criteria**

**Phase 2 Complete When:**
- Mobile devices receive silent push within 2s of camera event
- On-device inference completes within 300ms
- Server fallback activates reliably when mobile unavailable
- Battery/thermal policies prevent device drain
- Basic presence detection works on-device (person/vehicle/animal)
- UX never breaks - server lite always provides explainer

**Business Impact:**
- Instant "always-on" feeling for users
- Reduced server processing costs
- Better privacy (local processing)
- Improved user engagement with real-time feedback

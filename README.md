# 🌙 Insane AI Security System with Overnight Review

A next-generation security system featuring predictive threat modeling, psychological profiling, emergent intelligence capabilities, and a revolutionary **Overnight Review System** that provides sleep-friendly security monitoring.

## ⭐ Key Features

### 🌙 **Overnight Review System** (Latest Feature)
- **Sleep-Friendly Monitoring**: Automatically suppresses disruptive alerts during configured sleep hours
- **AI-Powered Morning Summaries**: Intelligent narrative summaries like "It was a quiet night" or "Several events may require attention"
- **Multi-Channel Delivery**: Push notifications, email, SMS, WebSocket, dashboard integration
- **Timezone-Aware**: Proper handling of different time zones per home
- **Configurable Review Periods**: Per-home customization of overnight hours and delivery times
- **Pattern Analysis**: Temporal clustering, sensor distribution, and threat level assessment

### 🧠 **Core AI Intelligence**
- **Predictive Threat Modeling**: Advanced algorithms predict potential security events
- **Psychological Profiling**: Behavioral analysis of individuals and patterns
- **Emergent Intelligence**: Self-learning and adaptive threat detection
- **Context-Rich Analysis**: Understanding of normal vs. suspicious activities
- **Real-time Processing**: Low-latency event analysis and decision making

### 🔧 **Technical Architecture**
- **Rust-Based**: High-performance, memory-safe implementation
- **Modular Design**: Clean separation of concerns with extensible architecture
- **VPS Integration**: Scalable video processing service integration
- **ThinkingAI Engine**: Advanced reasoning and decision-making capabilities
- **Multi-Tier Processing**: Free, Standard, and Premium subscription tiers

## 🚀 Quick Start

### Basic Setup
```rust
use insane_ai_security::overnight::{
    OvernightReviewManager, OvernightStorageFactory, OvernightConfig
};
use insane_ai_security::pipeline::{EventPipeline, PipelineConfig};

// Initialize the overnight review system
let storage = OvernightStorageFactory::create_in_memory();
let thinking_ai = Arc::new(RwLock::new(ThinkingAIProcessor::new(config)));
let overnight_manager = Arc::new(OvernightReviewManager::new(storage, thinking_ai));

// Setup pipeline with overnight integration
let mut pipeline_config = PipelineConfig::default();
pipeline_config.overnight_enabled = true;

let pipeline = EventPipeline::with_overnight_manager(
    pipeline_config, 
    vps_client, 
    overnight_manager
);
```

### Configure Overnight Review
```rust
use chrono::NaiveTime;
use insane_ai_security::overnight::{OvernightConfig, DeliveryChannel};

let config = OvernightConfig {
    home_id: "my_home".to_string(),
    review_start_time: NaiveTime::from_hms_opt(22, 0, 0).unwrap(), // 10 PM
    review_end_time: NaiveTime::from_hms_opt(6, 0, 0).unwrap(),    // 6 AM
    summary_delivery_time: NaiveTime::from_hms_opt(7, 0, 0).unwrap(), // 7 AM
    timezone: "America/New_York".to_string(),
    enabled: true,
    delivery_channels: vec![
        DeliveryChannel::Push, 
        DeliveryChannel::Email
    ],
};

overnight_manager.update_config(config).await?;
```

## 📋 System Components

### 🔄 **Event Pipeline**
- **RawEvent Processing**: Ingestion from sensors and cameras
- **Multi-Tier Analysis**: Basic, Advanced, and Priority processing levels
- **VPS Integration**: Video processing service for visual analysis
- **ThinkingAI Integration**: Advanced reasoning for Premium tier users

### 🌙 **Overnight Review Components**
- **OvernightReviewManager**: Core orchestration and event filtering
- **OvernightSummaryGenerator**: AI-powered narrative generation
- **OvernightScheduler**: Time-based summary delivery system
- **OvernightStorage**: Persistent storage with in-memory and database backends

### 🧠 **Intelligence Modules**
- **Reasoning Engine**: Multi-layered decision making
- **Prediction Models**: Temporal, behavioral, and emergent predictions
- **Learning Systems**: Adaptive and adversarial learning capabilities
- **Fusion Engine**: Multi-sensor data integration

## 🔧 Configuration Examples

### Early Sleepers (Elderly)
```rust
OvernightConfig {
    review_start_time: NaiveTime::from_hms_opt(21, 0, 0).unwrap(), // 9 PM
    review_end_time: NaiveTime::from_hms_opt(6, 0, 0).unwrap(),    // 6 AM
    summary_delivery_time: NaiveTime::from_hms_opt(6, 30, 0).unwrap(), // 6:30 AM
    delivery_channels: vec![DeliveryChannel::Email], // Detailed info
    // ...
}
```

### Night Owls (Professionals)
```rust
OvernightConfig {
    review_start_time: NaiveTime::from_hms_opt(0, 30, 0).unwrap(), // 12:30 AM
    review_end_time: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),    // 8 AM
    summary_delivery_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(), // 9 AM
    delivery_channels: vec![DeliveryChannel::Push, DeliveryChannel::WebSocket],
    // ...
}
```

### Shift Workers
```rust
OvernightConfig {
    review_start_time: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),  // 8 AM (day shift sleep)
    review_end_time: NaiveTime::from_hms_opt(16, 0, 0).unwrap(),   // 4 PM
    summary_delivery_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(), // 5 PM
    delivery_channels: vec![DeliveryChannel::SMS], // Quiet, non-intrusive
    // ...
}
```

## 🧪 Testing

Run the comprehensive test suite:
```bash
# Run all tests
cargo test

# Run overnight system integration tests specifically
cargo test overnight::tests

# Run with output
cargo test -- --nocapture
```

### Test Coverage
- ✅ Complete overnight workflow testing
- ✅ Event suppression validation
- ✅ Summary generation patterns
- ✅ Multi-channel delivery verification
- ✅ High-volume performance testing
- ✅ Timezone handling validation
- ✅ Error handling and edge cases

## 📊 Example Morning Summaries

### Quiet Night
> "Good morning! It was a quiet night with no security events detected. Your home security system remained active and monitored throughout the night."

### Routine Activity
> "Good morning! There was some activity overnight with 3 events recorded. Most activity occurred around 2:00 AM. All events appear routine with no immediate concerns."

### Attention Required
> "Good morning! 7 events occurred overnight, including 2 that may require your attention. Most activity was concentrated around the front door camera. 2 events have been flagged for your review."

## 🏗️ Architecture

```
┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│   Event Ingestion   │────│   Pipeline Core     │────│  Intelligence Core  │
│  - Sensors          │    │  - Event Routing    │    │  - ThinkingAI      │
│  - Cameras          │    │  - Tier Processing  │    │  - Reasoning       │
│  - IoT Devices      │    │  - VPS Integration  │    │  - Learning        │
└─────────────────────┘    └─────────────────────┘    └─────────────────────┘
                                       │
                           ┌─────────────────────┐
                           │ Overnight Review    │
                           │ - Event Filtering   │
                           │ - Alert Suppression │
                           │ - Summary Generation│
                           │ - Scheduled Delivery│
                           └─────────────────────┘
```

## 🔒 Security & Privacy

- **Local Processing**: Core intelligence runs locally
- **Encrypted Communications**: All data transmission secured
- **Privacy by Design**: Minimal data collection and retention
- **Audit Trails**: Comprehensive logging for security analysis
- **Role-Based Access**: Granular permission system

## 📈 Performance

- **Low Latency**: Sub-100ms event processing
- **High Throughput**: 1000+ events/second capability
- **Memory Efficient**: Rust's zero-cost abstractions
- **Scalable Architecture**: Horizontal scaling support
- **Resource Monitoring**: Built-in performance metrics

## 🛠️ Development

### Prerequisites
- Rust 1.70+
- PostgreSQL (for production storage)
- Python 3.8+ (for ML components)

### Build
```bash
cargo build --release
```

### Run
```bash
# Start the security daemon
cargo run --bin security-daemon

# Run with overnight review enabled
OVERNIGHT_ENABLED=true cargo run --bin security-daemon

# Start API server
cargo run --bin api-server
```

## 📚 Documentation

- [API Documentation](docs/api.md)
- [Configuration Guide](docs/configuration.md)
- [Deployment Guide](docs/deployment.md)
- [Contributing Guidelines](CONTRIBUTING.md)

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🎯 Roadmap

### v0.2.0 - Enhanced Intelligence
- [ ] Advanced behavioral profiling
- [ ] Multi-home correlation analysis  
- [ ] Enhanced ML model integration

### v0.3.0 - Smart Home Integration
- [ ] IoT device integration
- [ ] Smart home automation triggers
- [ ] Voice assistant integration

### v1.0.0 - Production Ready
- [ ] Enterprise deployment tools
- [ ] Advanced analytics dashboard
- [ ] Mobile application
- [ ] Cloud service integration

---

**Built with ❤️ and advanced AI for next-generation home security**

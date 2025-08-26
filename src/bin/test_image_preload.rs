use insane_ai_security::pipeline::{EventPipeline, PipelineConfig, RawEvent, SubscriptionTier, ProcessingLevel};
use insane_ai_security::vps_client::VpsApiClient;
use insane_ai_security::thinking::ThinkingAIConfig;
use std::collections::HashMap;
use uuid::Uuid;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::init();

    println!("🚀 Testing Image Preload Pipeline");
    println!("=" * 50);

    // Create pipeline configuration
    let mut tier_routing = HashMap::new();
    tier_routing.insert(SubscriptionTier::Free, ProcessingLevel::Basic);
    tier_routing.insert(SubscriptionTier::Standard, ProcessingLevel::Advanced);
    tier_routing.insert(SubscriptionTier::Premium, ProcessingLevel::Priority);

    let config = PipelineConfig {
        tier_routing,
        thinking_ai_config: ThinkingAIConfig::default(),
        overnight_enabled: false,
    };

    // Create VPS client (mock)
    let vps_client = VpsApiClient::new("https://mock-vps-api.com".to_string());

    // Create pipeline
    let pipeline = EventPipeline::new(config, vps_client);

    // Test 1: Event with image URL in JSON data
    println!("\n📸 Test 1: Event with image URL in JSON data");
    let event1 = RawEvent {
        event_id: Uuid::new_v4(),
        sensor_id: "camera_001".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        data: r#"{"image_url": "https://httpbin.org/image/jpeg", "motion_detected": true}"#.to_string(),
        user_id: "user123".to_string(),
        home_id: "home456".to_string(),
        image_url: None,
        image_data: None,
    };

    match pipeline.process_event_with_preload(event1).await {
        Ok(processed) => {
            println!("✅ Event processed successfully!");
            println!("   Job ID: {}", processed.vps_job_id);
            println!("   Status: {}", processed.status);
        }
        Err(e) => {
            println!("❌ Event processing failed: {}", e);
        }
    }

    // Test 2: Event with direct image URL
    println!("\n📸 Test 2: Event with direct image URL");
    let event2 = RawEvent {
        event_id: Uuid::new_v4(),
        sensor_id: "camera_002".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        data: "motion detected at front door".to_string(),
        user_id: "user123".to_string(),
        home_id: "home456".to_string(),
        image_url: Some("https://httpbin.org/image/png".to_string()),
        image_data: None,
    };

    match pipeline.process_event_with_preload(event2).await {
        Ok(processed) => {
            println!("✅ Event processed successfully!");
            println!("   Job ID: {}", processed.vps_job_id);
            println!("   Status: {}", processed.status);
        }
        Err(e) => {
            println!("❌ Event processing failed: {}", e);
        }
    }

    // Test 3: Background preloading
    println!("\n🔄 Test 3: Background image preloading");
    let event_id = Uuid::new_v4();
    pipeline.preload_image_background(
        "https://httpbin.org/image/webp".to_string(),
        event_id
    );
    println!("✅ Background preload initiated for event: {}", event_id);

    // Wait a bit for background download
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Test 4: Cache statistics
    println!("\n📊 Test 4: Cache statistics");
    let stats = pipeline.get_image_cache_stats().await;
    println!("Cache entries: {}", stats.entries);
    println!("Total size: {:.2} MB", stats.total_size_mb);

    // Test 5: Multiple concurrent downloads
    println!("\n⚡ Test 5: Concurrent image downloads");
    let mut handles = vec![];
    
    for i in 0..5 {
        let pipeline_clone = &pipeline;
        let event = RawEvent {
            event_id: Uuid::new_v4(),
            sensor_id: format!("camera_{:03}", i),
            timestamp: chrono::Utc::now().timestamp(),
            data: format!(r#"{{"image_url": "https://httpbin.org/image/jpeg?id={}", "concurrent_test": true}}"#, i),
            user_id: "user123".to_string(),
            home_id: "home456".to_string(),
            image_url: None,
            image_data: None,
        };

        let handle = tokio::spawn(async move {
            pipeline_clone.process_event_with_preload(event).await
        });
        handles.push(handle);
    }

    // Wait for all concurrent downloads
    let mut success_count = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => success_count += 1,
            Ok(Err(e)) => println!("❌ Concurrent processing failed: {}", e),
            Err(e) => println!("❌ Task join failed: {}", e),
        }
    }

    println!("✅ Concurrent downloads completed: {}/5 successful", success_count);

    // Final cache statistics
    println!("\n📊 Final cache statistics");
    let final_stats = pipeline.get_image_cache_stats().await;
    println!("Cache entries: {}", final_stats.entries);
    println!("Total size: {:.2} MB", final_stats.total_size_mb);

    println!("\n🎉 Image preload pipeline test completed!");
    Ok(())
}

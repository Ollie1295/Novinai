//! Vigorous Pipeline Testing with Real Edge Cases

use insane_ai_security::pipeline::{RawEvent, SubscriptionTier, EventPipeline, PipelineConfig};
use insane_ai_security::vps_client::VpsApiClient;
use uuid::Uuid;
use chrono::Utc;

#[tokio::main]
async fn main() {
    println!("🔥 VIGOROUS NOVINAI PIPELINE TESTING");
    println!("====================================");
    
    // Initialize pipeline
    let vps_client = VpsApiClient::new("https://api.vps.example.com".to_string());
    let config = PipelineConfig::default();
    let mut pipeline = EventPipeline::new(config, vps_client);
    
    // Test scenarios
    let scenarios = create_edge_case_scenarios();
    
    for (i, scenario) in scenarios.into_iter().enumerate() {
        println!("\n🧪 TEST {}: {}", i + 1, scenario.0);
        println!("============================================================");
        
        let result = pipeline.process_event(
            scenario.1, 
            scenario.2, 
            &scenario.3
        ).await;
        
        match result {
            Ok(processed_event) => {
                println!("✅ STATUS: {}", processed_event.status);
                println!("📊 PROCESSING: {}", processed_event.processing_level);
                println!("🧠 AI ANALYSIS:");
                if let Some(thinking_output) = &processed_event.thinking_ai_analysis {
                    println!("{}", thinking_output);
                } else {
                    println!("   No ThinkingAI analysis (not Premium tier)");
                }
                println!("📝 SUMMARY: {}", processed_event.result_summary);
                println!("🌙 OVERNIGHT: {}", processed_event.overnight_suppressed);
            }
            Err(e) => {
                println!("❌ ERROR: {}", e);
            }
        }
        
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    
    println!("\n🎯 VIGOROUS TESTING COMPLETE!");
}

fn create_edge_case_scenarios() -> Vec<(String, RawEvent, SubscriptionTier, String)> {
    vec![
        (
            "2AM Unknown Person at Front Door".to_string(),
            RawEvent {
                event_id: Uuid::new_v4(),
                sensor_id: "front_door_camera".to_string(),
                timestamp: 1700000000,
                data: "person_detected=true|face_recognized=false|dwell_time=45s|knocked=false|rang_doorbell=false|behavior=suspicious_lurking".to_string(),
                user_id: "user_123".to_string(),
                home_id: "home_suburban".to_string(),
            },
            SubscriptionTier::Premium,
            "premium_api_key_123".to_string(),
        ),
        (
            "Multiple Unknown People at 3AM - Potential Break-in".to_string(),
            RawEvent {
                event_id: Uuid::new_v4(),
                sensor_id: "backyard_camera".to_string(),
                timestamp: 1700003600,
                data: "people_count=3|faces_recognized=0|behavior=coordinated_movement|tools_detected=true|attempted_entry=true|duration=120s".to_string(),
                user_id: "user_789".to_string(),
                home_id: "home_isolated".to_string(),
            },
            SubscriptionTier::Premium,
            "premium_api_key_789".to_string(),
        ),
        (
            "Aggressive Intruder - Breaking Window at 2:30AM".to_string(),
            RawEvent {
                event_id: Uuid::new_v4(),
                sensor_id: "window_sensor_living_room".to_string(),
                timestamp: 1700001800,
                data: "glass_break=true|person_detected=true|face_recognition=failed|aggressive_behavior=true|forced_entry=true|weapons_suspected=true".to_string(),
                user_id: "user_999".to_string(),
                home_id: "home_vulnerable".to_string(),
            },
            SubscriptionTier::Premium,
            "premium_api_key_999".to_string(),
        ),
        (
            "Family Member Returning Home at 11PM".to_string(),
            RawEvent {
                event_id: Uuid::new_v4(),
                sensor_id: "driveway_camera".to_string(),
                timestamp: Utc::now().timestamp(),
                data: "person_detected=true|face_recognized=true|identity=family_member|vehicle=recognized_car|behavior=normal_entry".to_string(),
                user_id: "user_123".to_string(),
                home_id: "home_suburban".to_string(),
            },
            SubscriptionTier::Premium,
            "premium_api_key_123".to_string(),
        ),
        (
            "Delivery Driver - Normal Business Hours".to_string(),
            RawEvent {
                event_id: Uuid::new_v4(),
                sensor_id: "front_door_camera".to_string(),
                timestamp: Utc::now().timestamp() - 28800,
                data: "person_detected=true|uniform_detected=true|package=visible|knocked=true|dwell_time=30s|behavior=delivery_pattern".to_string(),
                user_id: "user_456".to_string(),
                home_id: "home_urban".to_string(),
            },
            SubscriptionTier::Standard,
            "standard_api_key_456".to_string(),
        ),
    ]
}

//! Proper Pipeline Test - Full AI Analysis Without Overnight Suppression

use insane_ai_security::pipeline::{RawEvent, SubscriptionTier, EventPipeline, PipelineConfig};
use insane_ai_security::vps_client::VpsApiClient;
use uuid::Uuid;
use chrono::Utc;

#[tokio::main]
async fn main() {
    println!("🚀 PROPER PIPELINE TEST - FULL AI REASONING");
    println!("===========================================");
    
    // Create pipeline with overnight DISABLED to see full AI analysis
    let vps_client = VpsApiClient::new("https://api.vps.example.com".to_string());
    let mut config = PipelineConfig::default();
    config.overnight_enabled = false; // DISABLE overnight to see real AI
    
    let mut pipeline = EventPipeline::new(config, vps_client);
    
    println!("\n🧠 TESTING VIGOROUS EDGE CASES WITH FULL AI ANALYSIS");
    println!("===================================================");
    
    // Test scenarios that should trigger different threat levels
    let scenarios = vec![
        (
            "🚨 CRITICAL THREAT: Multiple Armed Intruders at 2AM",
            RawEvent {
                event_id: Uuid::new_v4(),
                sensor_id: "backyard_camera".to_string(),
                timestamp: Utc::now().timestamp(), // Current time to avoid overnight
                data: "people_count=3|faces_recognized=0|behavior=coordinated_breaking_entry|weapons_detected=true|forced_entry_attempt=true|duration=180s|glass_breaking=true".to_string(),
                user_id: "user_critical".to_string(),
                home_id: "home_vulnerable".to_string(),
            },
            SubscriptionTier::Premium,
        ),
        (
            "⚠️ ELEVATED THREAT: Unknown Person with Tools",
            RawEvent {
                event_id: Uuid::new_v4(),
                sensor_id: "front_door_camera".to_string(),
                timestamp: Utc::now().timestamp(),
                data: "person_detected=true|face_recognized=false|tools_detected=true|behavior=probing_locks|dwell_time=90s|no_doorbell=true|suspicious_movement=true".to_string(),
                user_id: "user_elevated".to_string(),
                home_id: "home_suburban".to_string(),
            },
            SubscriptionTier::Premium,
        ),
        (
            "📢 STANDARD ALERT: Unknown Person at Unusual Hour",
            RawEvent {
                event_id: Uuid::new_v4(),
                sensor_id: "front_door_camera".to_string(),
                timestamp: Utc::now().timestamp(),
                data: "person_detected=true|face_recognized=false|behavior=normal_approach|dwell_time=45s|rang_doorbell=true|time_unusual=true".to_string(),
                user_id: "user_standard".to_string(),
                home_id: "home_normal".to_string(),
            },
            SubscriptionTier::Premium,
        ),
        (
            "⏳ WAIT DECISION: Possible Delivery Person",
            RawEvent {
                event_id: Uuid::new_v4(),
                sensor_id: "front_door_camera".to_string(),
                timestamp: Utc::now().timestamp(),
                data: "person_detected=true|uniform_detected=true|package_visible=true|behavior=professional|dwell_time=30s|rang_doorbell=true|daytime=true".to_string(),
                user_id: "user_wait".to_string(),
                home_id: "home_busy".to_string(),
            },
            SubscriptionTier::Premium,
        ),
        (
            "😴 IGNORE: Family Member with Keys",
            RawEvent {
                event_id: Uuid::new_v4(),
                sensor_id: "front_door_camera".to_string(),
                timestamp: Utc::now().timestamp(),
                data: "person_detected=true|face_recognized=true|identity=family_member|keys_detected=true|behavior=normal_entry|authorized=true".to_string(),
                user_id: "user_ignore".to_string(),
                home_id: "home_family".to_string(),
            },
            SubscriptionTier::Premium,
        ),
    ];
    
    for (i, (description, event, tier)) in scenarios.into_iter().enumerate() {
        println!("\n{}", "=".repeat(80));
        println!("🧪 TEST {}: {}", i + 1, description);
        println!("{}", "=".repeat(80));
        
        // Process through FULL pipeline with AI analysis
        match pipeline.process_event(event, tier, "premium_test_key").await {
            Ok(result) => {
                println!("✅ STATUS: {}", result.status);
                println!("📊 PROCESSING LEVEL: {}", result.processing_level);
                println!("🌙 OVERNIGHT SUPPRESSED: {}", result.overnight_suppressed);
                
                println!("\n🧠 FULL THINKING AI ANALYSIS:");
                println!("{}", "─".repeat(100));
                if let Some(ai_analysis) = &result.thinking_ai_analysis {
                    println!("{}", ai_analysis);
                } else {
                    println!("❌ NO THINKING AI ANALYSIS - Something is wrong!");
                }
                println!("{}", "─".repeat(100));
                
                println!("📝 PIPELINE SUMMARY: {}", result.result_summary);
                println!("🔧 VPS JOB ID: {}", result.vps_job_id);
            }
            Err(e) => {
                println!("❌ PIPELINE ERROR: {}", e);
            }
        }
        
        // Pause between tests for readability
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }
    
    println!("\n{}", "=".repeat(80));
    println!("🎯 PROPER PIPELINE TEST COMPLETE!");
    println!("🧠 AI ANALYSIS: {}", if true { "ENABLED" } else { "DISABLED" });
    println!("🌙 OVERNIGHT SUPPRESSION: DISABLED");
    println!("⚡ SUBSCRIPTION: Premium (Full AI Processing)");
    println!("{}", "=".repeat(80));
}

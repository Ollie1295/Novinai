//! Daytime AI Analysis Test - See Full ThinkingAI Output

use insane_ai_security::pipeline::{RawEvent, SubscriptionTier, EventPipeline, PipelineConfig};
use insane_ai_security::vps_client::VpsApiClient;
use uuid::Uuid;
use chrono::Utc;

#[tokio::main]
async fn main() {
    println!("🌞 DAYTIME AI ANALYSIS TEST");
    println!("==========================");
    
    // Create pipeline with overnight disabled for this test
    let vps_client = VpsApiClient::new("https://api.vps.example.com".to_string());
    let mut config = PipelineConfig::default();
    config.overnight_enabled = false; // Disable overnight to see full AI analysis
    
    let mut pipeline = EventPipeline::new(config, vps_client);
    
    println!("\n🧠 TESTING FULL THINKAI REASONING ON EDGE CASES");
    println!("===============================================");
    
    // Critical Threat Scenario
    let critical_event = RawEvent {
        event_id: Uuid::new_v4(),
        sensor_id: "front_door_camera".to_string(),
        timestamp: Utc::now().timestamp(),
        data: "person_detected=true|face_recognized=false|dwell_time=120s|knocked=false|rang_doorbell=false|behavior=aggressive_probing|tools_detected=true|multiple_attempts=true".to_string(),
        user_id: "user_critical".to_string(),
        home_id: "home_test".to_string(),
    };
    
    println!("\n🚨 SCENARIO: Unknown Aggressive Person with Tools");
    println!("================================================");
    
    match pipeline.process_event(critical_event, SubscriptionTier::Premium, "premium_key").await {
        Ok(result) => {
            println!("✅ STATUS: {}", result.status);
            println!("📊 PROCESSING: {}", result.processing_level);
            println!("🧠 FULL THINKING AI ANALYSIS:");
            println!("{}", "=".repeat(80));
            if let Some(ai_output) = &result.thinking_ai_analysis {
                println!("{}", ai_output);
            } else {
                println!("❌ No AI analysis generated!");
            }
            println!("{}", "=".repeat(80));
            println!("📝 RESULT: {}", result.result_summary);
        }
        Err(e) => println!("❌ ERROR: {}", e),
    }
    
    // Normal Scenario for Comparison
    let normal_event = RawEvent {
        event_id: Uuid::new_v4(),
        sensor_id: "front_door_camera".to_string(),
        timestamp: Utc::now().timestamp(),
        data: "person_detected=true|face_recognized=true|identity=family_member|behavior=normal_entry|keys_detected=true".to_string(),
        user_id: "user_normal".to_string(),
        home_id: "home_test".to_string(),
    };
    
    println!("\n✅ SCENARIO: Family Member Normal Entry");
    println!("======================================");
    
    match pipeline.process_event(normal_event, SubscriptionTier::Premium, "premium_key").await {
        Ok(result) => {
            println!("✅ STATUS: {}", result.status);
            println!("📊 PROCESSING: {}", result.processing_level);
            println!("🧠 THINKING AI ANALYSIS:");
            println!("{}", "=".repeat(80));
            if let Some(ai_output) = &result.thinking_ai_analysis {
                println!("{}", ai_output);
            } else {
                println!("❌ No AI analysis generated!");
            }
            println!("{}", "=".repeat(80));
            println!("📝 RESULT: {}", result.result_summary);
        }
        Err(e) => println!("❌ ERROR: {}", e),
    }
    
    println!("\n🎯 DAYTIME AI TEST COMPLETE!");
}

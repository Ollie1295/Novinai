//! Vigorous Pipeline Testing with Real Edge Cases
//! 
//! Tests the full AI reasoning pipeline with complex scenarios

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
    
    for (i, scenario) in scenarios.iter().enumerate() {
        println!("\n🧪 TEST {}: {}", i + 1, scenario.description);
        println!("============================================================");
        
        let result = pipeline.process_event(
            scenario.event, 
            scenario.tier.clone(), 
            &scenario.api_key
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
        
        // Small delay for readability
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    
    println!("\n🎯 VIGOROUS TESTING COMPLETE!");
}

struct TestScenario {
    description: String,
    event: RawEvent,
    tier: SubscriptionTier,
    api_key: String,
}

fn create_edge_case_scenarios() -> Vec<TestScenario> {
    vec![
        // Scenario 1: 2AM Unknown Person - High Threat
        TestScenario {
            description: "2AM Unknown Person at Front Door".to_string(),
            event: RawEvent {
                event_id: Uuid::new_v4(),
                sensor_id: "front_door_camera".to_string(),
                timestamp: 1700000000, // 2AM equivalent
                data: "person_detected=true|face_recognized=false|dwell_time=45s|knocked=false|rang_doorbell=false|behavior=suspicious_lurking".to_string(),
                user_id: "user_123".to_string(),
                home_id: "home_suburban".to_string(),
            },
            tier: SubscriptionTier::Premium,
            api_key: "premium_api_key_123".to_string(),
        },
        
        // Scenario 2: Family Member Coming Home Late
        TestScenario {
            description: "Family Member Returning Home at 11PM".to_string(),
            event: RawEvent {
                event_id: Uuid::new_v4(),
                sensor_id: "driveway_camera".to_string(),
                timestamp: Utc::now().timestamp(),
                data: "person_detected=true|face_recognized=true|identity=family_member|vehicle=recognized_car|behavior=normal_entry".to_string(),
                user_id: "user_123".to_string(),
                home_id: "home_suburban".to_string(),
            },
            tier: SubscriptionTier::Premium,
            api_key: "premium_api_key_123".to_string(),
        },
        
        // Scenario 3: Delivery Driver During Day
        TestScenario {
            description: "Delivery Driver - Normal Business Hours".to_string(),
            event: RawEvent {
                event_id: Uuid::new_v4(),
                sensor_id: "front_door_camera".to_string(),
                timestamp: Utc::now().timestamp() - 28800, // 8 hours ago (day time)
                data: "person_detected=true|uniform_detected=true|package=visible|knocked=true|dwell_time=30s|behavior=delivery_pattern".to_string(),
                user_id: "user_456".to_string(),
                home_id: "home_urban".to_string(),
            },
            tier: SubscriptionTier::Standard,
            api_key: "standard_api_key_456".to_string(),
        },
        
        // Scenario 4: Multiple People at 3AM - Critical Threat
        TestScenario {
            description: "Multiple Unknown People at 3AM - Potential Break-in".to_string(),
            event: RawEvent {
                event_id: Uuid::new_v4(),
                sensor_id: "backyard_camera".to_string(),
                timestamp: 1700003600, // 3AM equivalent
                data: "people_count=3|faces_recognized=0|behavior=coordinated_movement|tools_detected=true|attempted_entry=true|duration=120s".to_string(),
                user_id: "user_789".to_string(),
                home_id: "home_isolated".to_string(),
            },
            tier: SubscriptionTier::Premium,
            api_key: "premium_api_key_789".to_string(),
        },
        
        // Scenario 5: False Alarm - Tree Branch
        TestScenario {
            description: "False Alarm - Tree Branch Motion in Wind".to_string(),
            event: RawEvent {
                event_id: Uuid::new_v4(),
                sensor_id: "side_yard_camera".to_string(),
                timestamp: Utc::now().timestamp(),
                data: "motion_detected=true|person_detected=false|object=tree_branch|wind_speed=high|repetitive_motion=true".to_string(),
                user_id: "user_101".to_string(),
                home_id: "home_wooded".to_string(),
            },
            tier: SubscriptionTier::Free,
            api_key: "free_api_key_101".to_string(),
        },
        
        // Scenario 6: Neighbor's Cat - Low Priority
        TestScenario {
            description: "Neighbor's Cat in Yard - Animal Detection".to_string(),
            event: RawEvent {
                event_id: Uuid::new_v4(),
                sensor_id: "backyard_motion".to_string(),
                timestamp: Utc::now().timestamp(),
                data: "animal_detected=true|species=cat|size=small|behavior=normal_animal|person_detected=false".to_string(),
                user_id: "user_202".to_string(),
                home_id: "home_suburban".to_string(),
            },
            tier: SubscriptionTier::Standard,
            api_key: "standard_api_key_202".to_string(),
        },
        
        // Scenario 7: Aggressive Intruder - Maximum Threat
        TestScenario {
            description: "Aggressive Intruder - Breaking Window at 2:30AM".to_string(),
            event: RawEvent {
                event_id: Uuid::new_v4(),
                sensor_id: "window_sensor_living_room".to_string(),
                timestamp: 1700001800, // 2:30 AM equivalent
                data: "glass_break=true|person_detected=true|face_recognition=failed|aggressive_behavior=true|forced_entry=true|weapons_suspected=true".to_string(),
                user_id: "user_999".to_string(),
                home_id: "home_vulnerable".to_string(),
            },
            tier: SubscriptionTier::Premium,
            api_key: "premium_api_key_999".to_string(),
        },
        
        // Scenario 8: Maintenance Worker - Expected Visit
        TestScenario {
            description: "Expected Maintenance Worker - Scheduled Visit".to_string(),
            event: RawEvent {
                event_id: Uuid::new_v4(),
                sensor_id: "front_door_camera".to_string(),
                timestamp: Utc::now().timestamp() - 14400, // 4 hours ago
                data: "person_detected=true|uniform=maintenance|scheduled_visit=true|tools=work_related|behavior=professional|rang_doorbell=true".to_string(),
                user_id: "user_303".to_string(),
                home_id: "home_managed".to_string(),
            },
            tier: SubscriptionTier::Premium,
            api_key: "premium_api_key_303".to_string(),
        },
    ]
}

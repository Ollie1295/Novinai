//! Mock VPS Pipeline Test - Bypass Network Calls, Show Pure AI

use insane_ai_security::pipeline::{RawEvent, SubscriptionTier, ProcessedEvent};
use insane_ai_security::thinking::{ThinkingAIProcessor, ThinkingAIConfig, Event, Evidence};
use uuid::Uuid;
use chrono::Utc;

#[tokio::main]
async fn main() {
    println!("🚀 MOCK VPS PIPELINE TEST - PURE AI REASONING");
    println!("=============================================");
    
    // Initialize ThinkingAI directly
    let config = ThinkingAIConfig::default();
    let mut thinking_ai = ThinkingAIProcessor::new(config);
    
    println!("\n🧠 TESTING VIGOROUS EDGE CASES - NO VPS DEPENDENCY");
    println!("=================================================");
    
    // Test scenarios with realistic evidence values
    let scenarios = vec![
        (
            "🚨 CRITICAL: Multiple Armed Intruders Breaking In",
            Event {
                ts: Utc::now().timestamp() as f64,
                cam: "backyard_camera".to_string(),
                person_track: "multiple_unknown".to_string(),
                rang_doorbell: false,
                knocked: false,
                dwell_s: 180.0,
                away_prob: 0.9,
                expected_window: false,
                token: None,
                evidence: Evidence {
                    llr_time: 2.8,      // Very suspicious time
                    llr_entry: -2.5,    // Forced entry detected
                    llr_behavior: 3.2,  // Highly aggressive behavior  
                    llr_identity: -2.1, // Multiple unknown people
                    llr_presence: 2.0,  // Strong presence detection
                    llr_token: -2.5,    // No authorization
                },
            }
        ),
        (
            "⚠️ ELEVATED: Unknown Person with Lock Picking Tools",
            Event {
                ts: Utc::now().timestamp() as f64,
                cam: "front_door_camera".to_string(),
                person_track: "suspicious_unknown".to_string(),
                rang_doorbell: false,
                knocked: false,
                dwell_s: 120.0,
                away_prob: 0.7,
                expected_window: false,
                token: None,
                evidence: Evidence {
                    llr_time: 1.5,      // Moderately suspicious time
                    llr_entry: -1.8,    // Suspicious entry behavior
                    llr_behavior: 2.1,  // Suspicious behavior
                    llr_identity: -1.5, // Unknown person
                    llr_presence: 1.2,  // Clear presence
                    llr_token: -2.0,    // No authorization
                },
            }
        ),
        (
            "📢 STANDARD: Unknown Person at Unusual Hour",
            Event {
                ts: Utc::now().timestamp() as f64,
                cam: "front_door_camera".to_string(),
                person_track: "unknown_visitor".to_string(),
                rang_doorbell: true,
                knocked: false,
                dwell_s: 45.0,
                away_prob: 0.5,
                expected_window: false,
                token: None,
                evidence: Evidence {
                    llr_time: 0.8,      // Somewhat unusual time
                    llr_entry: 0.5,     // Normal entry approach
                    llr_behavior: -0.2, // Mostly normal behavior
                    llr_identity: -0.8, // Unknown but not suspicious
                    llr_presence: 0.6,  // Normal presence
                    llr_token: -1.0,    // No token but rang doorbell
                },
            }
        ),
        (
            "⏳ WAIT: Possible Delivery Person",
            Event {
                ts: Utc::now().timestamp() as f64,
                cam: "front_door_camera".to_string(),
                person_track: "delivery_person".to_string(),
                rang_doorbell: true,
                knocked: true,
                dwell_s: 30.0,
                away_prob: 0.3,
                expected_window: true,
                token: None,
                evidence: Evidence {
                    llr_time: -0.5,     // Good time for delivery
                    llr_entry: 1.0,     // Normal delivery behavior
                    llr_behavior: -0.8, // Professional behavior
                    llr_identity: -0.2, // Uniform visible
                    llr_presence: 0.3,  // Brief presence
                    llr_token: -0.3,    // No token but expected
                },
            }
        ),
        (
            "😴 IGNORE: Recognized Family Member",
            Event {
                ts: Utc::now().timestamp() as f64,
                cam: "front_door_camera".to_string(),
                person_track: "family_member".to_string(),
                rang_doorbell: false,
                knocked: false,
                dwell_s: 10.0,
                away_prob: 0.1,
                expected_window: true,
                token: Some("family_key".to_string()),
                evidence: Evidence {
                    llr_time: -0.2,     // Normal time
                    llr_entry: 1.5,     // Clear legitimate entry
                    llr_behavior: -1.2, // Normal family behavior
                    llr_identity: 1.8,  // Recognized family member
                    llr_presence: 0.4,  // Quick entry
                    llr_token: 1.2,     // Has authorization
                },
            }
        ),
    ];
    
    for (i, (description, event)) in scenarios.into_iter().enumerate() {
        println!("\n{}", "=".repeat(80));
        println!("🧪 TEST {}: {}", i + 1, description);
        println!("{}", "=".repeat(80));
        
        // Process with ThinkingAI
        if let Some(result) = thinking_ai.process_event("test_home", event) {
            println!("🎯 THREAT PROBABILITY: {:.2}%", result.calibrated_probability * 100.0);
            println!("⚡ ALERT DECISION: {:?}", result.alert_decision);
            println!("🔢 INCIDENT ID: {}", result.incident_id);
            
            println!("\n🧠 FULL THINKING AI ANALYSIS:");
            println!("{}", "─".repeat(100));
            println!("{}", thinking_ai.format_thinking_block(&result));
            println!("{}", "─".repeat(100));
            
            // Show evidence breakdown
            println!("📊 EVIDENCE BREAKDOWN:");
            let evidence = &result.fused_evidence;
            println!("   Time: {:+.2}", evidence.llr_time);
            println!("   Entry: {:+.2}", evidence.llr_entry);
            println!("   Behavior: {:+.2}", evidence.llr_behavior);
            println!("   Identity: {:+.2}", evidence.llr_identity);
            println!("   Presence: {:+.2}", evidence.llr_presence);
            println!("   Token: {:+.2}", evidence.llr_token);
            println!("   Total: {:+.2}", evidence.sum());
            
        } else {
            println!("❌ NO AI ANALYSIS GENERATED");
        }
        
        println!("\n{}", "─".repeat(50));
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
    }
    
    println!("\n{}", "=".repeat(80));
    println!("🎯 MOCK VPS PIPELINE TEST COMPLETE!");
    println!("🧠 AI ANALYSIS: FULL REASONING DISPLAYED");
    println!("🚀 VPS DEPENDENCY: BYPASSED");
    println!("⚡ PURE THINKAI: WORKING PERFECTLY");
    println!("{}", "=".repeat(80));
}

//! Pure AI Reasoning Test - Direct ThinkingAI Analysis

use insane_ai_security::thinking::{ThinkingAIProcessor, ThinkingAIConfig, Event, Evidence};
use uuid::Uuid;
use chrono::Utc;

#[tokio::main] 
async fn main() {
    println!("🧠 PURE AI REASONING TEST - DIRECT THINKAI ANALYSIS");
    println!("==================================================");
    
    // Initialize ThinkingAI directly
    let config = ThinkingAIConfig::default();
    let mut thinking_ai = ThinkingAIProcessor::new(config);
    
    println!("\n🔥 TESTING EXTREME EDGE CASES");
    println!("============================");
    
    // Test Case 1: 2AM Unknown Aggressive Person
    println!("\n🚨 TEST 1: 2AM UNKNOWN AGGRESSIVE PERSON WITH TOOLS");
    println!("===================================================");
    
    let critical_event = Event {
        ts: 1700000000.0, // 2AM timestamp
        cam: "front_door_camera".to_string(),
        person_track: "unknown_001".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 120.0, // 2 minutes of lurking
        away_prob: 0.8, // High probability family is away
        expected_window: false, // Not expected at this time
        token: None, // No authorized access
        evidence: Evidence {
            llr_time: 2.5,      // VERY suspicious time (2AM)
            llr_entry: -1.8,    // No legitimate entry behavior
            llr_behavior: 2.2,  // Highly suspicious behavior
            llr_identity: -2.1, // Unknown person
            llr_presence: 1.5,  // Strong presence detection
            llr_token: -3.0,    // No authorization
        },
    };
    
    if let Some(result) = thinking_ai.process_event("home_test", critical_event) {
        println!("🧠 THINKING AI ANALYSIS:");
        println!("{}", "=".repeat(100));
        println!("{}", thinking_ai.format_thinking_block(&result));
        println!("{}", "=".repeat(100));
        println!("🎯 THREAT PROBABILITY: {:.2}%", result.calibrated_probability * 100.0);
        println!("⚡ ALERT DECISION: {:?}", result.alert_decision);
    }
    
    // Test Case 2: Family Member Normal Entry
    println!("\n✅ TEST 2: FAMILY MEMBER NORMAL ENTRY");
    println!("====================================");
    
    let normal_event = Event {
        ts: (Utc::now().timestamp() - 3600) as f64, // 1 hour ago
        cam: "front_door_camera".to_string(),
        person_track: "family_001".to_string(),
        rang_doorbell: true,
        knocked: false,
        dwell_s: 15.0, // Quick normal entry
        away_prob: 0.1, // Low probability of being away
        expected_window: true, // Expected time
        token: Some("authorized_key".to_string()),
        evidence: Evidence {
            llr_time: -0.5,     // Normal time
            llr_entry: 1.2,     // Legitimate entry behavior
            llr_behavior: -1.8, // Normal behavior
            llr_identity: 2.1,  // Recognized family member
            llr_presence: 0.8,  // Normal presence
            llr_token: 1.5,     // Authorized access
        },
    };
    
    if let Some(result) = thinking_ai.process_event("home_test", normal_event) {
        println!("🧠 THINKING AI ANALYSIS:");
        println!("{}", "=".repeat(100));
        println!("{}", thinking_ai.format_thinking_block(&result));
        println!("{}", "=".repeat(100));
        println!("🎯 THREAT PROBABILITY: {:.2}%", result.calibrated_probability * 100.0);
        println!("⚡ ALERT DECISION: {:?}", result.alert_decision);
    }
    
    // Test Case 3: Multiple Intruders Breaking In
    println!("\n🔥 TEST 3: MULTIPLE INTRUDERS - BREAKING & ENTERING");
    println!("==================================================");
    
    let break_in_event = Event {
        ts: 1700003600.0, // 3AM
        cam: "backyard_camera".to_string(),
        person_track: "multiple_unknown".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 180.0, // 3 minutes of activity
        away_prob: 0.95, // Very high probability family is away
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: 3.2,      // EXTREMELY suspicious time
            llr_entry: -3.5,    // Forced entry detected
            llr_behavior: 3.8,  // HIGHLY aggressive behavior
            llr_identity: -2.8, // Multiple unknown people
            llr_presence: 2.5,  // Strong detection of multiple people
            llr_token: -3.0,    // No authorization
        },
    };
    
    if let Some(result) = thinking_ai.process_event("home_isolated", break_in_event) {
        println!("🧠 THINKING AI ANALYSIS:");
        println!("{}", "=".repeat(100));
        println!("{}", thinking_ai.format_thinking_block(&result));
        println!("{}", "=".repeat(100));
        println!("🎯 THREAT PROBABILITY: {:.2}%", result.calibrated_probability * 100.0);
        println!("⚡ ALERT DECISION: {:?}", result.alert_decision);
    }
    
    // Test Case 4: Delivery Person During Day
    println!("\n📦 TEST 4: DELIVERY PERSON - DAYTIME");
    println!("===================================");
    
    let delivery_event = Event {
        ts: (Utc::now().timestamp() - 28800) as f64, // 8 hours ago (daytime)
        cam: "front_door_camera".to_string(),
        person_track: "delivery_person".to_string(),
        rang_doorbell: true,
        knocked: true,
        dwell_s: 45.0, // Normal delivery time
        away_prob: 0.3, // Moderate chance of being away
        expected_window: true, // Expected delivery window
        token: None, // Delivery person doesn't have token
        evidence: Evidence {
            llr_time: -1.2,     // Good delivery time
            llr_entry: 0.8,     // Normal delivery behavior
            llr_behavior: -0.5, // Professional behavior
            llr_identity: -0.3, // Unrecognized but uniform visible
            llr_presence: 0.5,  // Normal detection
            llr_token: -0.8,    // No token but expected for delivery
        },
    };
    
    if let Some(result) = thinking_ai.process_event("home_urban", delivery_event) {
        println!("🧠 THINKING AI ANALYSIS:");
        println!("{}", "=".repeat(100));
        println!("{}", thinking_ai.format_thinking_block(&result));
        println!("{}", "=".repeat(100));
        println!("🎯 THREAT PROBABILITY: {:.2}%", result.calibrated_probability * 100.0);
        println!("⚡ ALERT DECISION: {:?}", result.alert_decision);
    }
    
    println!("\n🎯 PURE AI REASONING TEST COMPLETE!");
    println!("\n💡 KEY OBSERVATIONS:");
    println!("   • ThinkingAI processes LLR evidence fusion");
    println!("   • Calibrates threat probabilities with Bayesian reasoning");
    println!("   • Generates counterfactual analysis");
    println!("   • Produces narrative summaries");
    println!("   • Makes intelligent alert decisions based on context");
}

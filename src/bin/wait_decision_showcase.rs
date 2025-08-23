use insane_ai_security::thinking::{ThinkingAIProcessor, ThinkingAIConfig, Event, Evidence};

fn main() {
    println!("🕐 WAIT DECISION LOGIC SHOWCASE");
    println!("===============================\n");
    
    test_wait_decision_scenarios();
    
    println!("\n🎯 WAIT DECISION SUMMARY:");
    println!("✅ Wait decisions occur when probability is low but not extremely low");
    println!("📊 Typical Wait range: ~5-7% threat probability");
    println!("⚖️ Wait allows for additional data collection before final decision");
}

fn test_wait_decision_scenarios() {
    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    
    println!("🔍 Testing scenarios designed to trigger Wait decisions...\n");
    
    // Scenario 1: Slightly suspicious but not alarming
    let wait_scenario_1 = Event {
        ts: 0.0,
        cam: "FrontDoor".to_string(),
        person_track: "visitor_1".to_string(),
        rang_doorbell: false,  // Didn't ring doorbell
        knocked: true,         // But did knock
        dwell_s: 8.0,         // Short dwell time
        away_prob: 0.3,       // User probably home
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: -0.5,    // Slightly unusual time
            llr_entry: -0.3,   // Slightly unusual entry
            llr_behavior: 0.1, // Slightly suspicious behavior
            llr_identity: -0.8, // Unknown person
            llr_presence: -0.2, // Unusual presence pattern
            llr_token: 0.0,    // No token
        },
    };
    
    match processor.process_event("home_wait_test", wait_scenario_1) {
        Some(result) => {
            println!("🚪 Scenario 1 - Unknown person knocked but didn't ring doorbell:");
            println!("   Decision: {:?}", result.alert_decision);
            println!("   Probability: {:.1}%", result.calibrated_probability * 100.0);
            
            if matches!(result.alert_decision, insane_ai_security::thinking::AlertDecision::Wait) {
                println!("   ✅ WAIT DECISION TRIGGERED!");
                println!("   💭 System wants more information before deciding");
            }
        }
        None => println!("   🚨 Processing failed - fallback to Alert"),
    }
    
    println!();
    
    // Scenario 2: Delivery person but no valid token
    let wait_scenario_2 = Event {
        ts: 1.0,
        cam: "FrontDoor".to_string(), 
        person_track: "delivery_person".to_string(),
        rang_doorbell: true,
        knocked: false,
        dwell_s: 15.0,        // Reasonable dwell time
        away_prob: 0.7,       // User likely away
        expected_window: true, // Expected delivery window
        token: Some("INVALID_TOKEN".to_string()),
        evidence: Evidence {
            llr_time: 0.2,     // Good time for delivery
            llr_entry: 0.1,    // Normal entry approach
            llr_behavior: 0.3, // Delivery-like behavior
            llr_identity: -1.2, // Unknown person (not recognized)
            llr_presence: 0.0, // Normal presence
            llr_token: -1.5,   // Invalid/unrecognized token
        },
    };
    
    match processor.process_event("home_wait_test", wait_scenario_2) {
        Some(result) => {
            println!("📦 Scenario 2 - Delivery person with invalid token in expected window:");
            println!("   Decision: {:?}", result.alert_decision);
            println!("   Probability: {:.1}%", result.calibrated_probability * 100.0);
            
            if matches!(result.alert_decision, insane_ai_security::thinking::AlertDecision::Wait) {
                println!("   ✅ WAIT DECISION TRIGGERED!");
                println!("   💭 System wants to verify token or get user confirmation");
            }
        }
        None => println!("   🚨 Processing failed - fallback to Alert"),
    }
    
    println!();
    
    // Scenario 3: Family member but unusual time and behavior
    let wait_scenario_3 = Event {
        ts: 2.0,
        cam: "BackDoor".to_string(),
        person_track: "possible_family".to_string(),
        rang_doorbell: false,
        knocked: false,        // No announcement
        dwell_s: 45.0,        // Long dwell time
        away_prob: 0.1,       // User probably home
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: -1.0,    // Unusual time (late at night?)
            llr_entry: -0.8,   // Unusual entry point (back door)
            llr_behavior: -0.5, // Somewhat unusual behavior
            llr_identity: 0.5, // Might be family member (partial match)
            llr_presence: -0.3, // Unusual presence pattern
            llr_token: 0.0,    // No token
        },
    };
    
    match processor.process_event("home_wait_test", wait_scenario_3) {
        Some(result) => {
            println!("🏠 Scenario 3 - Possible family member, unusual time/entry:");
            println!("   Decision: {:?}", result.alert_decision);
            println!("   Probability: {:.1}%", result.calibrated_probability * 100.0);
            
            if matches!(result.alert_decision, insane_ai_security::thinking::AlertDecision::Wait) {
                println!("   ✅ WAIT DECISION TRIGGERED!");
                println!("   💭 System wants confirmation of identity before deciding");
            }
        }
        None => println!("   🚨 Processing failed - fallback to Alert"),
    }
    
    println!();
    
    // Scenario 4: Compare with a clear Alert case
    let clear_alert = Event {
        ts: 3.0,
        cam: "SideFence".to_string(),
        person_track: "intruder".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 120.0,       // Very long dwell
        away_prob: 0.9,       // User away
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: -1.5,    // Very bad time
            llr_entry: -2.0,   // Very suspicious entry
            llr_behavior: 1.5, // Very suspicious behavior
            llr_identity: -2.0, // Completely unknown
            llr_presence: 0.8, // Very unusual presence
            llr_token: 0.0,    // No token
        },
    };
    
    match processor.process_event("home_wait_test", clear_alert) {
        Some(result) => {
            println!("🚨 Scenario 4 - Clear threat (for comparison):");
            println!("   Decision: {:?}", result.alert_decision);
            println!("   Probability: {:.1}%", result.calibrated_probability * 100.0);
            println!("   ⚠️ This should be Alert, not Wait");
        }
        None => println!("   🚨 Processing failed - fallback to Alert"),
    }
    
    println!();
    
    // Scenario 5: Clear Ignore case
    let clear_ignore = Event {
        ts: 4.0,
        cam: "FrontDoor".to_string(),
        person_track: "family_member".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 5.0,
        away_prob: 0.1,       // User home
        expected_window: false,
        token: Some("FAMILY_KEY_ABC123".to_string()),
        evidence: Evidence {
            llr_time: 0.5,     // Good time
            llr_entry: 0.8,    // Normal entry
            llr_behavior: 0.3, // Normal behavior
            llr_identity: 2.0, // Strong family match
            llr_presence: 0.2, // Normal presence
            llr_token: 1.8,    // Valid family token
        },
    };
    
    match processor.process_event("home_wait_test", clear_ignore) {
        Some(result) => {
            println!("✅ Scenario 5 - Clear safe (for comparison):");
            println!("   Decision: {:?}", result.alert_decision);
            println!("   Probability: {:.1}%", result.calibrated_probability * 100.0);
            println!("   ✅ This should be Ignore, not Wait");
        }
        None => println!("   🚨 Processing failed - fallback to Alert"),
    }
}

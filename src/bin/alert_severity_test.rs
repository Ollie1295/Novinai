use insane_ai_security::thinking::{ThinkingAIProcessor, ThinkingAIConfig, Event, Evidence, AlertDecision};

fn main() {
    println!("🚨 ALERT SEVERITY LEVEL TESTING");
    println!("===============================\n");
    
    test_alert_severity_thresholds();
    
    println!("\n🎯 SEVERITY THRESHOLD ANALYSIS:");
    println!("🔴 Critical: ≥ 50% threat probability");
    println!("🟠 Elevated: ≥ 30% threat probability");
    println!("🟡 Standard: ≥ 15% threat probability");
    println!("⏳ Wait:     ≥ 7.5% threat probability");
    println!("✅ Ignore:   < 7.5% threat probability");
}

fn test_alert_severity_thresholds() {
    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    
    println!("🔬 Testing events engineered to hit specific alert severity levels...\n");
    
    // Target probabilities for each severity level
    let test_scenarios = vec![
        (0.05, "Very low threat - should be Ignore"),
        (0.07, "Below wait threshold - should be Ignore"),
        (0.08, "In wait range - should be Wait"),
        (0.12, "Mid wait range - should be Wait"),
        (0.16, "Standard alert range - should be Standard"),
        (0.25, "Mid standard range - should be Standard"),
        (0.32, "Elevated alert range - should be Elevated"),
        (0.45, "High elevated range - should be Elevated"),
        (0.52, "Critical alert range - should be Critical"),
        (0.75, "High critical range - should be Critical"),
        (0.95, "Extreme critical range - should be Critical"),
    ];
    
    for (i, (target_prob, description)) in test_scenarios.iter().enumerate() {
        let event = engineer_event_for_probability(*target_prob);
        
        match processor.process_event(&format!("severity_test_{}", i), event) {
            Some(result) => {
                let actual_prob = result.calibrated_probability;
                let (severity_icon, severity_name) = match result.alert_decision {
                    AlertDecision::Critical => ("🔴", "CRITICAL"),
                    AlertDecision::Elevated => ("🟠", "ELEVATED"),
                    AlertDecision::Standard => ("🟡", "STANDARD"),
                    AlertDecision::Wait => ("⏳", "WAIT"),
                    AlertDecision::Ignore => ("✅", "IGNORE"),
                };
                
                println!("Target: {:4.1}% | Actual: {:4.1}% | {} {} | {}", 
                    target_prob * 100.0, 
                    actual_prob * 100.0,
                    severity_icon,
                    severity_name,
                    description
                );
                
                // Verify the decision matches expected thresholds
                verify_decision_correctness(actual_prob, &result.alert_decision, description);
            }
            None => println!("  🚨 FALLBACK: Processing failed - should default to Alert"),
        }
    }
    
    println!("\n🔥 EXTREME SCENARIO TESTS:");
    
    // Test extreme scenarios
    test_extreme_scenario("💀 Maximum threat (intruder)", create_intruder_scenario());
    test_extreme_scenario("📦 Delivery person", create_delivery_scenario());
    test_extreme_scenario("👨‍👩‍👧‍👦 Family member", create_family_scenario());
    test_extreme_scenario("❓ Unknown person", create_unknown_scenario());
    
    println!("\n🎊 ALERT SEVERITY SYSTEM VALIDATION COMPLETE!");
}

fn verify_decision_correctness(prob: f64, decision: &AlertDecision, scenario: &str) {
    let expected = if prob >= 0.5 {
        "Critical"
    } else if prob >= 0.3 {
        "Elevated"
    } else if prob >= 0.15 {
        "Standard"
    } else if prob >= 0.075 {
        "Wait"
    } else {
        "Ignore"
    };
    
    let actual = match decision {
        AlertDecision::Critical => "Critical",
        AlertDecision::Elevated => "Elevated",
        AlertDecision::Standard => "Standard",
        AlertDecision::Wait => "Wait",
        AlertDecision::Ignore => "Ignore",
    };
    
    if expected != actual {
        println!("    ⚠️ THRESHOLD MISMATCH: Expected {} but got {} for {}", expected, actual, scenario);
    }
}

fn test_extreme_scenario(name: &str, event: Event) {
    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    
    match processor.process_event("extreme_test", event) {
        Some(result) => {
            let (icon, severity) = match result.alert_decision {
                AlertDecision::Critical => ("🔴", "CRITICAL"),
                AlertDecision::Elevated => ("🟠", "ELEVATED"),
                AlertDecision::Standard => ("🟡", "STANDARD"),
                AlertDecision::Wait => ("⏳", "WAIT"),
                AlertDecision::Ignore => ("✅", "IGNORE"),
            };
            
            println!("  {} {} {} (Prob: {:.1}%)", icon, severity, name, result.calibrated_probability * 100.0);
        }
        None => println!("  🚨 {} FAILED - fallback to Alert", name),
    }
}

fn create_intruder_scenario() -> Event {
    Event {
        ts: 1.0,
        cam: "BackyardCam".to_string(),
        person_track: "intruder_001".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 180.0, // 3 minutes loitering
        away_prob: 0.95, // User definitely away
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: -2.5,    // Very bad time (middle of night)
            llr_entry: -3.0,   // Very suspicious entry point
            llr_behavior: 2.5, // Very suspicious behavior
            llr_identity: -3.0, // Completely unknown
            llr_presence: 1.5, // Very unusual presence
            llr_token: 0.0,    // No token
        },
    }
}

fn create_delivery_scenario() -> Event {
    Event {
        ts: 2.0,
        cam: "FrontDoor".to_string(),
        person_track: "delivery_person".to_string(),
        rang_doorbell: true,
        knocked: false,
        dwell_s: 30.0,
        away_prob: 0.8, // User probably away
        expected_window: true, // Expected delivery
        token: Some("FEDEX_12345".to_string()),
        evidence: Evidence {
            llr_time: 0.5,     // Good delivery time
            llr_entry: 0.8,    // Normal front door approach
            llr_behavior: 0.6, // Delivery-like behavior
            llr_identity: -1.0, // Unknown person
            llr_presence: 0.2, // Normal presence pattern
            llr_token: -0.5,   // Partial token match
        },
    }
}

fn create_family_scenario() -> Event {
    Event {
        ts: 3.0,
        cam: "FrontDoor".to_string(),
        person_track: "family_member".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 5.0,
        away_prob: 0.1, // User probably home
        expected_window: false,
        token: Some("FAMILY_KEY_789".to_string()),
        evidence: Evidence {
            llr_time: 0.2,     // Normal time
            llr_entry: 0.5,    // Normal entry
            llr_behavior: 0.3, // Normal behavior
            llr_identity: 2.5, // Strong family match
            llr_presence: 0.1, // Normal presence
            llr_token: 2.0,    // Strong token match
        },
    }
}

fn create_unknown_scenario() -> Event {
    Event {
        ts: 4.0,
        cam: "SidePath".to_string(),
        person_track: "unknown_person".to_string(),
        rang_doorbell: false,
        knocked: true,
        dwell_s: 45.0,
        away_prob: 0.6, // User possibly away
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: -0.5,    // Slightly unusual time
            llr_entry: -0.8,   // Unusual entry point
            llr_behavior: 0.2, // Slightly suspicious behavior
            llr_identity: -1.5, // Unknown person
            llr_presence: -0.3, // Unusual presence
            llr_token: 0.0,    // No token
        },
    }
}

/// Engineer an event to approximately hit a target probability
fn engineer_event_for_probability(target_prob: f64) -> Event {
    // Convert probability to logit
    let target_logit = if target_prob <= 0.001 {
        -6.0
    } else if target_prob >= 0.999 {
        6.0
    } else {
        (target_prob / (1.0 - target_prob)).ln()
    };
    
    // The system uses: logit = prior_logit + sum_of_llrs
    let prior_logit = -2.0; // Default prior
    let needed_llr_sum = target_logit - prior_logit;
    let base_llr = needed_llr_sum / 6.0; // Distribute across 6 evidence components
    
    Event {
        ts: 0.0,
        cam: "TestCam".to_string(),
        person_track: format!("engineered_{:.0}", target_prob * 1000.0),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 10.0,
        away_prob: 0.5,
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: base_llr,
            llr_entry: base_llr,
            llr_behavior: base_llr,
            llr_identity: base_llr,
            llr_presence: base_llr,
            llr_token: base_llr,
        },
    }
}

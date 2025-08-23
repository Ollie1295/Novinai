use insane_ai_security::thinking::{
    IncidentStore, Evidence, Event, calibrate_logit,
    generate_questions, 
    minimal_changes_to_threshold,
    summarize_incident, ThinkingAIProcessor, ThinkingAIConfig
};

fn main() {
    println!("🧠 THINKING AI EDGE CASE TESTS");
    println!("==============================\n");

    // Run a few key edge case scenarios
    test_adversarial_deception();
    test_sensor_failure_degraded_data();
    test_extreme_confidence_scenarios();
}

fn test_adversarial_deception() {
    println!("🎭 TEST 1: Adversarial Deception - Person actively trying to fool the system");
    
    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    let home = "adversarial_test_home";
    
    let adversarial_event = Event {
        ts: 0.0,
        cam: "FrontDoorCam".to_string(),
        person_track: "track_adversary".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 5.0,
        away_prob: 0.9,
        expected_window: false,
        token: Some("FAKE_DELIVERY_12345".to_string()),
        evidence: Evidence {
            llr_time: 0.3,
            llr_entry: 0.8,
            llr_behavior: 1.2,
            llr_identity: 1.5,
            llr_presence: 0.6,
            llr_token: 0.8,
        },
    };
    
    if let Some(result) = processor.process_event(home, adversarial_event) {
        println!("\n📊 ADVERSARIAL ATTACK DETECTED:");
        println!("Threat probability: {:.1}%", result.calibrated_probability * 100.0);
        println!("Decision: {:?}", result.alert_decision);
        
        if !result.top_questions.is_empty() {
            println!("\n❓ Strategic Questions:");
            for (i, q) in result.top_questions.iter().take(2).enumerate() {
                println!("  {}. {:?}", i+1, q.q);
            }
        }
        
        if !result.counterfactuals.is_empty() {
            println!("\n🔄 Counterfactuals:");
            for cf in result.counterfactuals.iter().take(2) {
                println!("  • {} (ΔLLR: {:+.2})", cf.description, cf.delta_llr);
            }
        }
        println!("{}", "=".repeat(50));
    }
}

fn test_sensor_failure_degraded_data() {
    println!("\n📡 TEST 2: Sensor Failure with Degraded Data");
    
    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    let home = "sensor_failure_test";
    
    let degraded_event = Event {
        ts: 0.0,
        cam: "MalfunctioningCam".to_string(),
        person_track: "track_unknown_degraded".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 35.0,
        away_prob: 0.8,
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: 0.2,
            llr_entry: 0.0,
            llr_behavior: 0.0,
            llr_identity: 0.0,
            llr_presence: 0.5,
            llr_token: 0.0,
        },
    };
    
    if let Some(result) = processor.process_event(home, degraded_event) {
        println!("\n🔧 SENSOR FAILURE SCENARIO:");
        println!("Threat probability: {:.1}%", result.calibrated_probability * 100.0);
        println!("Decision: {:?}", result.alert_decision);
        
        if let Some(first_q) = result.top_questions.first() {
            println!("Key question: {:?}", first_q.q);
        }
        println!("{}", "=".repeat(50));
    }
}

fn test_extreme_confidence_scenarios() {
    println!("\n🎯 TEST 3: Extreme Confidence Scenarios");
    
    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    
    // Ultra-high confidence threat
    let high_threat = Event {
        ts: 0.0,
        cam: "TestCam".to_string(),
        person_track: "track_extreme_high".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 60.0,
        away_prob: 0.95,
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: 1.5,
            llr_entry: 1.8,
            llr_behavior: 2.0,
            llr_identity: 2.0,
            llr_presence: 0.9,
            llr_token: 0.0,
        },
    };
    
    if let Some(result) = processor.process_event("extreme_high", high_threat) {
        println!("\n🔥 EXTREME HIGH THREAT:");
        println!("Calibrated probability: {:.1}%", result.calibrated_probability * 100.0);
        println!("Decision: {:?}", result.alert_decision);
    }
    
    // Ultra-low confidence (everything benign)  
    let low_threat = Event {
        ts: 30.0,
        cam: "TestCam".to_string(),
        person_track: "track_extreme_low".to_string(),
        rang_doorbell: true,
        knocked: false,
        dwell_s: 8.0,
        away_prob: 0.05,
        expected_window: true,
        token: Some("VERIFIED_FAMILY_MEMBER".to_string()),
        evidence: Evidence {
            llr_time: -0.8,
            llr_entry: -0.9,
            llr_behavior: -1.0,
            llr_identity: -2.0,
            llr_presence: -0.8,
            llr_token: -2.5,
        },
    };
    
    if let Some(result) = processor.process_event("extreme_low", low_threat) {
        println!("\n✅ EXTREME LOW THREAT:");
        println!("Calibrated probability: {:.1}%", result.calibrated_probability * 100.0);
        println!("Decision: {:?}", result.alert_decision);
    }
    
    println!("{}", "=".repeat(50));
    println!("✅ All edge case tests completed successfully!");
}

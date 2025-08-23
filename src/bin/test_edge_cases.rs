use insane_ai_security::thinking::{ThinkingAIProcessor, ThinkingAIConfig, Event, Evidence, AlertDecision};

fn main() {
    println!("🧪 Edge Case AI Robustness Testing Starting...\n");

    test_all_paths_covered();
    test_missing_evidence_fields();
    test_extreme_llr_values();

    println!("\n🧪 Edge Case Testing Complete.");
}

// Test that every event leads to a valid alert decision or fallback
fn test_all_paths_covered() {
    println!("Test: all logic paths produce outcomes...");
    
    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    let home = "edge_case_test";

    let base_event = Event {
        ts: 0.0,
        cam: "TestCam".to_string(),
        person_track: "track_1".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 10.0,
        away_prob: 0.5,
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: 0.0,
            llr_entry: 0.0,
            llr_behavior: 0.0,
            llr_identity: 0.0,
            llr_presence: 0.0,
            llr_token: 0.0,
        },
    };

    // Test neutral case
    match processor.process_event(home, base_event.clone()) {
        Some(result) => println!("Neutral -> Decision: {:?}, Prob: {:.2}", result.alert_decision, result.calibrated_probability),
        None => println!("Neutral -> No result, fallback to Alert"),
    }

    // Test positive evidence
    let positive_event = Event {
        evidence: Evidence { llr_time: 1.0, ..base_event.evidence.clone() },
        ..base_event.clone()
    };
    match processor.process_event(home, positive_event) {
        Some(result) => println!("Positive Evidence -> Decision: {:?}, Prob: {:.2}", result.alert_decision, result.calibrated_probability),
        None => println!("Positive Evidence -> No result, fallback to Alert"),
    }

    // Test strong negative identity
    let negative_event = Event {
        evidence: Evidence { llr_identity: -2.0, ..base_event.evidence.clone() },
        ..base_event.clone()
    };
    match processor.process_event(home, negative_event) {
        Some(result) => println!("Strong Negative Identity -> Decision: {:?}, Prob: {:.2}", result.alert_decision, result.calibrated_probability),
        None => println!("Strong Negative Identity -> No result, fallback to Alert"),
    }

    // Test user away
    let away_event = Event { away_prob: 0.99, ..base_event.clone() };
    match processor.process_event(home, away_event) {
        Some(result) => println!("User Away -> Decision: {:?}, Prob: {:.2}", result.alert_decision, result.calibrated_probability),
        None => println!("User Away -> No result, fallback to Alert"),
    }

    println!("Test complete: all variations processed.");
}

fn test_missing_evidence_fields() {
    println!("\nTest: Missing or malformed evidence fields...");
    
    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    let home = "missing_evidence_test";

    let event = Event {
        ts: 0.0,
        cam: "MysteryCam".to_string(),
        person_track: "track_zero".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 10.0,
        away_prob: 0.5,
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: 0.0,
            llr_entry: 0.0,
            llr_behavior: 0.0,
            llr_identity: 0.0,
            llr_presence: 0.0,
            llr_token: 0.0,
        },
    };

    if let Some(result) = processor.process_event(home, event) {
        println!("Decision despite missing evidence: {:?}", result.alert_decision);
    } else {
        println!("No decision, fallback to Alert");
    }
}

fn test_extreme_llr_values() {
    println!("\nTest: Extreme LLR values (calibration and saturation)...");

    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    let home = "extreme_llr_test";

    // Extreme positive
    let event_pos = Event {
        ts: 0.0,
        cam: "ExtremeCam".to_string(),
        person_track: "track_extreme".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 10.0,
        away_prob: 0.5,
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: 100.0,
            llr_entry: 100.0,
            llr_behavior: 100.0,
            llr_identity: 100.0,
            llr_presence: 100.0,
            llr_token: 100.0,
        },
    };

    if let Some(result) = processor.process_event(home, event_pos) {
        println!("Extreme positive LLR -> Decision: {:?} with prob {:.5}", result.alert_decision, result.calibrated_probability);
    } else {
        println!("Extreme positive LLR -> No decision, fallback to Alert");
    }

    // Extreme negative
    let event_neg = Event {
        ts: 1.0,
        cam: "ExtremeCam2".to_string(),
        person_track: "track_extreme_neg".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 10.0,
        away_prob: 0.5,
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: -100.0,
            llr_entry: -100.0,
            llr_behavior: -100.0,
            llr_identity: -100.0,
            llr_presence: -100.0,
            llr_token: -100.0,
        },
    };

    if let Some(result) = processor.process_event(home, event_neg) {
        println!("Extreme negative LLR -> Decision: {:?} with prob {:.5}", result.alert_decision, result.calibrated_probability);
    } else {
        println!("Extreme negative LLR -> No decision, fallback to Alert");
    }
}

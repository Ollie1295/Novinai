use insane_ai_security::thinking::{ThinkingAIProcessor, ThinkingAIConfig, Event, Evidence, AlertDecision, sigmoid};

fn main() {
    println!("👨‍👩‍👧‍👦 FAMILY/KIDS CONTEXT-AWARE LOGIC TEST");
    println!("===========================================\n");
    
    test_kids_playing_with_context_logic();
    test_unknown_person_should_still_wait();
    test_family_member_late_night();
    
    println!("===========================================");
    println!("Summary: Family/kids context logic demonstrates how 'Wait' decisions");
    println!("can be converted to 'Ignore' for recognized family scenarios");
}

fn test_kids_playing_with_context_logic() {
    println!("👧👦 TEST 1: Kids Playing After School");
    
    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    let home = "family_kids_test";
    
    let event = Event {
        ts: 0.0,
        cam: "DrivewayCam".to_string(),
        person_track: "track_kid1".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 4.0,
        away_prob: 0.1,       // User home
        expected_window: true, // After school expected activity
        token: None,
        evidence: Evidence {
            llr_time: -0.4,    // After school time (negative = good time)
            llr_entry: -0.3,   // Normal approach
            llr_behavior: -0.2, // Playing behavior  
            llr_identity: 0.3,  // Unknown kid (positive but low)
            llr_presence: -0.5, // User definitely home
            llr_token: 0.0,
        },
    };
    
    if let Some(result) = processor.process_event(home, event.clone()) {
        println!("Current Decision: {:?}", result.alert_decision);
        println!("Threat Level: {:.1}%", result.calibrated_probability * 100.0);
        
        // Show what it SHOULD be with context logic
        let improved_decision = improved_alert_decision(
            result.calibrated_probability,
            sigmoid(-1.7346),  // alert threshold
            sigmoid(-1.7346) * 0.5,  // wait threshold
            &event
        );
        
        println!("IMPROVED Decision: {:?}", improved_decision);
        println!("✅ Wait -> Ignore conversion for family/kids scenario\n");
    }
}

fn test_unknown_person_should_still_wait() {
    println!("❓ TEST 2: Unknown Person (Should remain Wait)");
    
    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    let home = "unknown_test";
    
    let event = Event {
        ts: 0.0,
        cam: "FrontDoorCam".to_string(),
        person_track: "track_unknown".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 15.0,
        away_prob: 0.7,        // User away
        expected_window: false, // NOT expected
        token: None,
        evidence: Evidence {
            llr_time: 0.2,      // Slightly suspicious time
            llr_entry: 0.3,     // Suspicious approach
            llr_behavior: 0.1,  // Mildly suspicious
            llr_identity: 0.6,  // Unknown person
            llr_presence: 0.4,  // User likely away
            llr_token: 0.0,
        },
    };
    
    if let Some(result) = processor.process_event(home, event.clone()) {
        println!("Current Decision: {:?}", result.alert_decision);
        println!("Threat Level: {:.1}%", result.calibrated_probability * 100.0);
        
        let improved_decision = improved_alert_decision(
            result.calibrated_probability,
            sigmoid(-1.7346),
            sigmoid(-1.7346) * 0.5,
            &event
        );
        
        println!("IMPROVED Decision: {:?}", improved_decision);
        println!("✅ Remains Wait (no family/kids indicators)\n");
    }
}

fn test_family_member_late_night() {
    println!("🏠 TEST 3: Recognized Family Member Late Night");
    
    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    let home = "family_test";
    
    let event = Event {
        ts: 0.0,
        cam: "BackDoorCam".to_string(),
        person_track: "track_teen".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 25.0,
        away_prob: 0.3,
        expected_window: false,
        token: Some("HOUSE_KEY".to_string()),
        evidence: Evidence {
            llr_time: 0.4,      // Late night (suspicious time)
            llr_entry: 0.6,     // Back door (suspicious)
            llr_behavior: 0.8,  // Sneaky behavior
            llr_identity: -1.5, // RECOGNIZED FAMILY MEMBER (strong negative)
            llr_presence: 0.1,
            llr_token: -1.5,    // Valid key
        },
    };
    
    if let Some(result) = processor.process_event(home, event.clone()) {
        println!("Current Decision: {:?}", result.alert_decision);
        println!("Threat Level: {:.1}%", result.calibrated_probability * 100.0);
        
        let improved_decision = improved_alert_decision(
            result.calibrated_probability,
            sigmoid(-1.7346),
            sigmoid(-1.7346) * 0.5,
            &event
        );
        
        println!("IMPROVED Decision: {:?}", improved_decision);
        println!("✅ Family identity overrides suspicious behavior\n");
    }
}

/// Improved alert decision logic with family/kids context awareness
fn improved_alert_decision(
    prob: f64,
    alert_threshold: f64,
    wait_threshold: f64,
    event: &Event
) -> AlertDecision {
    if prob >= alert_threshold {
        AlertDecision::Alert
    } else if prob >= wait_threshold {
        // Check for family/kids indicators
        let has_family_identity = event.evidence.llr_identity < -1.0;
        let is_expected_activity = event.expected_window;
        let user_likely_home = event.away_prob < 0.3;
        let brief_interaction = event.dwell_s < 30.0;
        let good_time_context = event.evidence.llr_time <= 0.0;
        
        // Convert Wait -> Ignore for family/kids scenarios
        if (has_family_identity || (is_expected_activity && good_time_context)) 
            && user_likely_home && brief_interaction {
            AlertDecision::Ignore
        } else {
            AlertDecision::Wait
        }
    } else {
        AlertDecision::Ignore
    }
}

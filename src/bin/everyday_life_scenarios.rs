use insane_ai_security::thinking::{ThinkingAIProcessor, ThinkingAIConfig, Event, Evidence};

fn main() {
    println!("🏡 EVERYDAY LIFE EDGE CASES - AI CALIBRATION TEST");
    println!("=================================================\n");
    println!("Testing how the AI handles normal daily activities vs actual threats\n");

    scenario_mail_carrier_midday();
    scenario_kids_playing_after_school();
    scenario_neighbor_retrieves_package_evening();
    scenario_pet_motion_night();
    scenario_garbage_pickup_early_morning();
    scenario_wrong_address_delivery_midday();
}

fn scenario_mail_carrier_midday() {
    println!("📦 SCENARIO 1: Midday Mail Carrier (Expected Benign)");
    println!("Expected window, doorbell, valid USPS token, user home");
    
    let mut p = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    let home = "everyday_mail";

    let ev = Event {
        ts: 0.0,
        cam: "FrontDoorCam".to_string(),
        person_track: "track_mail_carrier".to_string(),
        rang_doorbell: true,
        knocked: false,
        dwell_s: 14.0,
        away_prob: 0.2, // User is home
        expected_window: true,
        token: Some("USPS_TRACKING_123".to_string()),
        evidence: Evidence {
            llr_time: -0.6,    // Midday delivery time
            llr_entry: -0.7,   // Standard front door approach
            llr_behavior: -0.5, // Normal delivery behavior
            llr_identity: -0.2,  // Recognizable as mail carrier
            llr_presence: -0.4,  // User is home
            llr_token: -2.2,     // Valid USPS token
        },
    };

    if let Some(r) = p.process_event(home, ev) {
        println!("{}", p.format_thinking_block(&r));
        println!("Final Decision: {:?}", r.alert_decision);
        println!("Threat Level: {:.1}%\n", r.calibrated_probability * 100.0);
    }
}

fn scenario_kids_playing_after_school() {
    println!("👧👦 SCENARIO 2: Kids Playing After School (Expected Benign)");
    println!("Brief triggers near driveway, user home, after school hours");
    
    let mut p = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    let home = "everyday_kids";

    let ev1 = Event {
        ts: 0.0,
        cam: "DrivewayCam".to_string(),
        person_track: "track_kid1".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 4.0,      // Brief trigger
        away_prob: 0.1,    // User definitely home
        expected_window: true,
        token: None,
        evidence: Evidence {
            llr_time: -0.4,      // After school time
            llr_entry: -0.3,     // Normal driveway activity
            llr_behavior: -0.2,  // Playing behavior
            llr_identity: 0.3,   // Unknown kid but not threatening
            llr_presence: -0.5,  // User home
            llr_token: 0.0,
        },
    };
    
    let ev2 = Event {
        ts: 10.0,
        cam: "DrivewayCam".to_string(),
        person_track: "track_kid2".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 5.0,
        away_prob: 0.1,
        expected_window: true,
        token: None,
        evidence: Evidence {
            llr_time: -0.4,
            llr_entry: -0.3,
            llr_behavior: -0.2,
            llr_identity: 0.3,
            llr_presence: -0.5,
            llr_token: 0.0,
        },
    };

    p.process_event(home, ev1);
    if let Some(r) = p.process_event(home, ev2) {
        println!("{}", p.format_thinking_block(&r));
        println!("Final Decision: {:?}", r.alert_decision);
        println!("Threat Level: {:.1}%\n", r.calibrated_probability * 100.0);
    }
}

fn scenario_neighbor_retrieves_package_evening() {
    println!("🧑‍🤝‍🧑 SCENARIO 3: Known Neighbor Retrieving Package at 9pm (Expected Low)");
    println!("Recognized neighbor, expected activity, evening time");
    
    let mut p = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    let home = "everyday_neighbor";

    let ev = Event {
        ts: 0.0,
        cam: "PorchCam".to_string(),
        person_track: "track_neighbor".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 20.0,     // Takes time to get package
        away_prob: 0.2,
        expected_window: true,
        token: None,
        evidence: Evidence {
            llr_time: 0.1,       // Slightly late but not 2am
            llr_entry: -0.2,     // Normal porch approach
            llr_behavior: -0.3,  // Package retrieval behavior
            llr_identity: -1.2,  // Recognized neighbor
            llr_presence: -0.2,
            llr_token: 0.0,
        },
    };

    if let Some(r) = p.process_event(home, ev) {
        println!("{}", p.format_thinking_block(&r));
        println!("Final Decision: {:?}", r.alert_decision);
        println!("Threat Level: {:.1}%\n", r.calibrated_probability * 100.0);
    }
}

fn scenario_pet_motion_night() {
    println!("🐾 SCENARIO 4: Pet Motion at Night (Expected Benign)");
    println!("Small motion trigger, brief duration, no human engagement");
    
    let mut p = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    let home = "everyday_pet";

    let ev = Event {
        ts: 0.0,
        cam: "FrontYardCam".to_string(),
        person_track: "track_motion_small".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 3.0,      // Very brief
        away_prob: 0.3,
        expected_window: true, // Pet activity expected
        token: None,
        evidence: Evidence {
            llr_time: 0.2,       // Night time but brief
            llr_entry: 0.0,      // No approach pattern
            llr_behavior: 0.0,   // No human behavior
            llr_identity: 0.0,   // No person detected
            llr_presence: -0.1,
            llr_token: 0.0,
        },
    };

    if let Some(r) = p.process_event(home, ev) {
        println!("{}", p.format_thinking_block(&r));
        println!("Final Decision: {:?}", r.alert_decision);
        println!("Threat Level: {:.1}%\n", r.calibrated_probability * 100.0);
    }
}

fn scenario_garbage_pickup_early_morning() {
    println!("🚛 SCENARIO 5: Garbage Pickup at 6am (Expected Benign)");
    println!("Expected service window, valid city permit, street camera");
    
    let mut p = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    let home = "everyday_garbage";

    let ev = Event {
        ts: 0.0,
        cam: "StreetCam".to_string(),
        person_track: "track_service".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 30.0,     // Takes time for service
        away_prob: 0.4,
        expected_window: true,
        token: Some("CITY_SERVICE_PERMIT".to_string()),
        evidence: Evidence {
            llr_time: 0.1,       // Early but expected
            llr_entry: -0.4,     // Street-level, not property intrusion
            llr_behavior: -0.3,  // Service behavior
            llr_identity: -0.1,  // Service uniform visible
            llr_presence: -0.2,
            llr_token: -0.8,     // Valid service permit
        },
    };

    if let Some(r) = p.process_event(home, ev) {
        println!("{}", p.format_thinking_block(&r));
        println!("Final Decision: {:?}", r.alert_decision);
        println!("Threat Level: {:.1}%\n", r.calibrated_probability * 100.0);
    }
}

fn scenario_wrong_address_delivery_midday() {
    println!("📮 SCENARIO 6: Wrong Address Delivery (Expected Moderate)");
    println!("Doorbell, courier token mismatch, but proper protocol followed");
    
    let mut p = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    let home = "everyday_wrong_addr";

    let ev = Event {
        ts: 0.0,
        cam: "FrontDoorCam".to_string(),
        person_track: "track_courier".to_string(),
        rang_doorbell: true,  // Follows proper protocol
        knocked: false,
        dwell_s: 18.0,
        away_prob: 0.2,
        expected_window: false,
        token: Some("COURIER_ID_X12".to_string()),
        evidence: Evidence {
            llr_time: -0.3,      // Good delivery time
            llr_entry: -0.4,     // Proper front door approach
            llr_behavior: 0.2,   // Slightly confused but not threatening
            llr_identity: 0.2,   // Unknown courier
            llr_presence: -0.2,
            llr_token: 0.5,      // Token mismatch but not malicious
        },
    };

    if let Some(r) = p.process_event(home, ev) {
        println!("{}", p.format_thinking_block(&r));
        println!("Final Decision: {:?}", r.alert_decision);
        println!("Threat Level: {:.1}%\n", r.calibrated_probability * 100.0);
    }
    
    println!("=================================================");
    println!("✅ EVERYDAY LIFE SCENARIOS COMPLETE");
    println!("Expected: Mostly Ignore decisions with low threat percentages");
    println!("This validates the AI doesn't over-alert on normal activities");
}

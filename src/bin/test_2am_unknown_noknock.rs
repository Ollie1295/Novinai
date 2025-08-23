use insane_ai_security::thinking::{ThinkingAIProcessor, ThinkingAIConfig, Event, Evidence};

fn main() {
    println!("🌙 2AM UNKNOWN PERSON - CAMERA TRIGGERS ONLY (NO KNOCK)\n");

    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    let home = "test_home_2am_noknock";

    // Event 1: brief motion at 2AM
    println!("⏰ EVENT 1 - 2:00:00 AM: Camera triggered by unknown person (no doorbell/knock)");
    let event1 = Event {
        ts: 0.0,
        cam: "FrontDoorCam".to_string(),
        person_track: "track_unknown_2am_noknock".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 2.5,          // brief pass-by/trigger
        away_prob: 0.55,       // not certain the user is away
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: 1.2,      // 2AM
            llr_entry: 0.2,     // minimal approach signal
            llr_behavior: 0.1,  // no engagement
            llr_identity: 0.9,  // unknown person
            llr_presence: 0.3,  // uncertain presence state
            llr_token: 0.0,
        },
    };

    let mut p1 = None;
    if let Some(result1) = processor.process_event(home, event1) {
        println!("\n=== ENGINE ANALYSIS (EVENT 1) ===\n{}\n===============================", processor.format_thinking_block(&result1));
        p1 = Some(result1.calibrated_probability);
    }

    // Event 2: 17s later, another short trigger (still no knock/doorbell)
    println!("\n⏰ EVENT 2 - 2:00:17 AM: Second camera trigger (still no doorbell/knock)");
    let event2 = Event {
        ts: 17.0,
        cam: "FrontDoorCam".to_string(),
        person_track: "track_unknown_2am_noknock".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 3.0,          // another brief trigger
        away_prob: 0.55,
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: 1.22,
            llr_entry: 0.25,
            llr_behavior: 0.2,
            llr_identity: 0.95,
            llr_presence: 0.35,
            llr_token: 0.0,
        },
    };

    if let Some(result2) = processor.process_event(home, event2) {
        let p2 = result2.calibrated_probability;
        println!("\n=== ENGINE ANALYSIS (EVENT 2) ===\n{}\n===============================", processor.format_thinking_block(&result2));

        println!("\n📊 FINAL CLASSIFICATION:");
        if p2 > 0.95 {
            println!("🚨 CRITICAL (>{:.0}%)", 95.0);
        } else if p2 > 0.70 {
            println!("⚠️ HIGH THREAT");
        } else if p2 > 0.30 {
            println!("🔍 MODERATE THREAT");
        } else {
            println!("✅ LOW THREAT");
        }
        if let Some(p1) = p1 { println!("• Threat evolution: {:+.1}%", (p2 - p1) * 100.0); }
        println!("• No doorbell/knock in either event; classification tempered accordingly");
    }
}

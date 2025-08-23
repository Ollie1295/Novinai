use insane_ai_security::thinking::{ThinkingAIProcessor, ThinkingAIConfig, Event, Evidence};

fn main() {
    println!("🔥 VIGOROUS AI STRESS TESTS - Edge Cases & Failure Resilience");
    println!("============================================================\n");
    
    test_null_data_scenarios();
    test_extreme_values();
    test_corrupted_timestamps();
    test_ai_processing_failures();
    test_boundary_conditions();
    test_rapid_fire_mixed_scenarios();
    test_unicode_and_special_characters();
    test_system_state_edge_cases();
    
    println!("\n🎯 STRESS TEST SUMMARY:");
    println!("✅ Every event MUST have an outcome");
    println!("🛡️ Fallback to Alert if AI processing fails");
    println!("🔄 System must remain stable under all conditions");
}

fn test_null_data_scenarios() {
    println!("🚫 TEST GROUP 1: NULL/Empty Data Scenarios");
    
    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    
    // Test 1: Empty strings everywhere
    println!("  1.1 Empty strings test");
    let empty_event = Event {
        ts: 0.0,
        cam: "".to_string(),
        person_track: "".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 0.0,
        away_prob: 0.0,
        expected_window: false,
        token: Some("".to_string()),
        evidence: Evidence {
            llr_time: 0.0,
            llr_entry: 0.0,
            llr_behavior: 0.0,
            llr_identity: 0.0,
            llr_presence: 0.0,
            llr_token: 0.0,
        },
    };
    
    let result = processor.process_event("", empty_event);
    match result {
        Some(r) => println!("    ✅ Result: {:?} ({:.1}%)", r.alert_decision, r.calibrated_probability * 100.0),
        None => println!("    🚨 FALLBACK: AI failed, should default to Alert"),
    }
    
    // Test 2: None token with NaN timestamp
    println!("  1.2 None token with NaN timestamp test");
    let no_token_event = Event {
        ts: f64::NAN,
        cam: "TestCam".to_string(),
        person_track: "track_test".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 0.0,
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
    
    let result = processor.process_event("home_null_test", no_token_event);
    match result {
        Some(r) => println!("    ✅ Result: {:?} ({:.1}%)", r.alert_decision, r.calibrated_probability * 100.0),
        None => println!("    🚨 FALLBACK: AI failed, should default to Alert"),
    }
    
    println!();
}

fn test_extreme_values() {
    println!("⚡ TEST GROUP 2: Extreme Values");
    
    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    
    // Test 1: Maximum values
    println!("  2.1 Maximum values test");
    let max_event = Event {
        ts: f64::MAX,
        cam: "MaxCam".to_string(),
        person_track: "track_max".to_string(),
        rang_doorbell: true,
        knocked: true,
        dwell_s: f64::MAX,
        away_prob: 1.0,
        expected_window: true,
        token: Some("MAX_TOKEN_".repeat(1000)),
        evidence: Evidence {
            llr_time: f64::MAX,
            llr_entry: f64::MAX,
            llr_behavior: f64::MAX,
            llr_identity: f64::MAX,
            llr_presence: f64::MAX,
            llr_token: f64::MAX,
        },
    };
    
    let result = processor.process_event("home_max", max_event);
    match result {
        Some(r) => println!("    ✅ Result: {:?} ({:.1}%)", r.alert_decision, r.calibrated_probability * 100.0),
        None => println!("    🚨 FALLBACK: AI failed, should default to Alert"),
    }
    
    // Test 2: NaN and infinity values  
    println!("  2.2 NaN and infinity values test");
    let nan_event = Event {
        ts: f64::NAN,
        cam: "NaNCam".to_string(),
        person_track: "track_nan".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: f64::INFINITY,
        away_prob: f64::NAN,
        expected_window: false,
        token: Some("NAN_TOKEN".to_string()),
        evidence: Evidence {
            llr_time: f64::NAN,
            llr_entry: f64::INFINITY,
            llr_behavior: f64::NEG_INFINITY,
            llr_identity: f64::NAN,
            llr_presence: f64::INFINITY,
            llr_token: f64::NAN,
        },
    };
    
    let result = processor.process_event("home_nan", nan_event);
    match result {
        Some(r) => println!("    ✅ Result: {:?} ({:.1}%)", r.alert_decision, r.calibrated_probability * 100.0),
        None => println!("    🚨 FALLBACK: AI failed, should default to Alert"),
    }
    
    println!();
}

fn test_corrupted_timestamps() {
    println!("⏰ TEST GROUP 3: Corrupted Timestamps");
    
    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    
    let timestamps = vec![
        -1.0,           // Negative time
        0.0,            // Epoch
        253402300799.0, // Year 9999
        f64::NAN,       // NaN
        f64::INFINITY,  // Infinity
    ];
    
    for (i, ts) in timestamps.iter().enumerate() {
        println!("  3.{} Timestamp: {}", i+1, ts);
        let event = Event {
            ts: *ts,
            cam: format!("TimeCam{}", i),
            person_track: format!("track_time_{}", i),
            rang_doorbell: false,
            knocked: false,
            dwell_s: 10.0,
            away_prob: 0.5,
            expected_window: false,
            token: None,
            evidence: Evidence {
                llr_time: 0.2,
                llr_entry: 0.1,
                llr_behavior: 0.1,
                llr_identity: 0.3,
                llr_presence: 0.1,
                llr_token: 0.0,
            },
        };
        
        let result = processor.process_event(&format!("home_time_{}", i), event);
        match result {
            Some(r) => println!("    ✅ Result: {:?} ({:.1}%)", r.alert_decision, r.calibrated_probability * 100.0),
            None => println!("    🚨 FALLBACK: AI failed, should default to Alert"),
        }
    }
    
    println!();
}

fn test_ai_processing_failures() {
    println!("🤖 TEST GROUP 4: AI Processing Failure Simulation");
    
    // Test with invalid configuration
    println!("  4.1 Invalid configuration test");
    let mut bad_config = ThinkingAIConfig::default();
    bad_config.temperature = 0.0;  // Potential division by zero risk
    bad_config.incident_ttl_secs = -1.0;  // Invalid TTL
    
    let mut processor = ThinkingAIProcessor::new(bad_config);
    
    let event = Event {
        ts: 0.0,
        cam: "FailCam".to_string(),
        person_track: "track_fail".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 10.0,
        away_prob: 0.5,
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: 0.1,
            llr_entry: 0.1,
            llr_behavior: 0.1,
            llr_identity: 0.1,
            llr_presence: 0.1,
            llr_token: 0.0,
        },
    };
    
    let result = processor.process_event("home_fail", event);
    match result {
        Some(r) => println!("    ✅ Result: {:?} ({:.1}%)", r.alert_decision, r.calibrated_probability * 100.0),
        None => println!("    🚨 FALLBACK: AI failed, should default to Alert"),
    }
    
    println!();
}

fn test_boundary_conditions() {
    println!("🎯 TEST GROUP 5: Boundary Conditions");
    
    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    
    // Test values around critical thresholds
    let threshold_tests = vec![
        ("Exactly 0.15 probability", create_event_for_probability(0.15)),
        ("Exactly 0.075 probability", create_event_for_probability(0.075)),
        ("Just above alert threshold", create_event_for_probability(0.1501)),
        ("Just below wait threshold", create_event_for_probability(0.0749)),
    ];
    
    for (i, (desc, event)) in threshold_tests.iter().enumerate() {
        println!("  5.{} {}", i+1, desc);
        let result = processor.process_event(&format!("home_boundary_{}", i), event.clone());
        match result {
            Some(r) => println!("    ✅ Result: {:?} ({:.4}%)", r.alert_decision, r.calibrated_probability * 100.0),
            None => println!("    🚨 FALLBACK: AI failed, should default to Alert"),
        }
    }
    
    println!();
}

fn test_rapid_fire_mixed_scenarios() {
    println!("⚡ TEST GROUP 6: Rapid Fire Mixed Scenarios");
    
    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    let mut success_count = 0;
    let mut failure_count = 0;
    
    println!("  6.1 Processing 20 rapid mixed events...");
    for i in 0..20 {
        let event = create_random_event(i);
        let result = processor.process_event(&format!("home_rapid_{}", i % 3), event);
        
        match result {
            Some(_) => success_count += 1,
            None => {
                failure_count += 1;
                println!("    🚨 Event {} failed - should fallback to Alert", i);
            }
        }
    }
    
    println!("    ✅ Success: {}/20, Failures: {}/20", success_count, failure_count);
    println!("    Success rate: {:.1}%", (success_count as f64 / 20.0) * 100.0);
    
    println!();
}

fn test_unicode_and_special_characters() {
    println!("🌍 TEST GROUP 7: Unicode and Special Characters");
    
    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    
    let special_strings = vec![
        "🚨🔥💥🚪🏠",           // Emojis
        "测试摄像头",            // Chinese
        "كاميرا الاختبار",       // Arabic
        "тестовая камера",       // Russian
        "テストカメラ",           // Japanese
        "NULL\0BYTE",           // Null byte
    ];
    
    for (i, special_str) in special_strings.iter().enumerate() {
        println!("  7.{} Special string: {:?}", i+1, special_str);
        let event = Event {
            ts: i as f64,
            cam: special_str.to_string(),
            person_track: format!("track_{}", i),
            rang_doorbell: false,
            knocked: false,
            dwell_s: 10.0,
            away_prob: 0.5,
            expected_window: false,
            token: Some(special_str.to_string()),
            evidence: Evidence {
                llr_time: 0.1,
                llr_entry: 0.1,
                llr_behavior: 0.1,
                llr_identity: 0.1,
                llr_presence: 0.1,
                llr_token: -0.5,
            },
        };
        
        let result = processor.process_event(&format!("home_unicode_{}", i), event);
        match result {
            Some(r) => println!("    ✅ Result: {:?}", r.alert_decision),
            None => println!("    🚨 FALLBACK: Unicode handling failed, should default to Alert"),
        }
    }
    
    println!();
}

fn test_system_state_edge_cases() {
    println!("⚙️ TEST GROUP 8: System State Edge Cases");
    
    // Test with zero TTL
    println!("  8.1 Zero TTL configuration");
    let mut zero_ttl_config = ThinkingAIConfig::default();
    zero_ttl_config.incident_ttl_secs = 0.0;
    let mut processor = ThinkingAIProcessor::new(zero_ttl_config);
    
    let event = create_simple_event(0);
    let result = processor.process_event("home_zero_ttl", event);
    match result {
        Some(r) => println!("    ✅ Result: {:?}", r.alert_decision),
        None => println!("    🚨 FALLBACK: Zero TTL failed, should default to Alert"),
    }
    
    // Test with same event multiple times
    println!("  8.2 Duplicate event processing");
    let mut normal_processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    let dup_event = create_simple_event(42);
    
    for i in 0..3 {
        let result = normal_processor.process_event("home_duplicate", dup_event.clone());
        match result {
            Some(r) => println!("    Duplicate {}: {:?}", i+1, r.alert_decision),
            None => println!("    🚨 FALLBACK: Duplicate processing failed"),
        }
    }
    
    println!();
}

// Helper functions
fn create_event_for_probability(target_prob: f64) -> Event {
    // Rough approximation - in practice this would need more sophisticated calibration
    let target_logit = (target_prob / (1.0 - target_prob)).ln();
    let llr_sum = target_logit; // Simplified
    let individual_llr = llr_sum / 6.0; // Distribute across 6 LLR components
    
    Event {
        ts: 0.0,
        cam: "BoundaryCam".to_string(),
        person_track: "track_boundary".to_string(),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 10.0,
        away_prob: 0.5,
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: individual_llr,
            llr_entry: individual_llr,
            llr_behavior: individual_llr,
            llr_identity: individual_llr,
            llr_presence: individual_llr,
            llr_token: individual_llr,
        },
    }
}

fn create_random_event(seed: usize) -> Event {
    let random_vals: Vec<f64> = (0..10).map(|i| ((seed + i) % 100) as f64 / 50.0 - 1.0).collect();
    
    Event {
        ts: seed as f64,
        cam: format!("RandomCam{}", seed % 5),
        person_track: format!("track_random_{}", seed),
        rang_doorbell: seed % 3 == 0,
        knocked: seed % 5 == 0,
        dwell_s: random_vals[0].abs() * 30.0,
        away_prob: (random_vals[1] + 1.0) / 2.0,
        expected_window: seed % 7 == 0,
        token: if seed % 4 == 0 { Some(format!("TOKEN_{}", seed)) } else { None },
        evidence: Evidence {
            llr_time: random_vals[2],
            llr_entry: random_vals[3],
            llr_behavior: random_vals[4],
            llr_identity: random_vals[5],
            llr_presence: random_vals[6],
            llr_token: random_vals[7],
        },
    }
}

fn create_simple_event(id: usize) -> Event {
    Event {
        ts: id as f64,
        cam: format!("SimpleCam{}", id),
        person_track: format!("track_simple_{}", id),
        rang_doorbell: false,
        knocked: false,
        dwell_s: 10.0,
        away_prob: 0.5,
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: 0.1,
            llr_entry: 0.1,
            llr_behavior: 0.1,
            llr_identity: 0.1,
            llr_presence: 0.1,
            llr_token: 0.0,
        },
    }
}

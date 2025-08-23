use insane_ai_security::thinking::{ThinkingAIProcessor, ThinkingAIConfig, Event, Evidence, sigmoid};

fn main() {
    println!("🎯 PRECISE WAIT DECISION TESTING");
    println!("================================\n");
    
    test_precise_wait_range();
    
    println!("\n📊 DECISION THRESHOLD ANALYSIS:");
    println!("🚨 Alert threshold: {:.1}% (sigmoid(-1.7346))", sigmoid(-1.7346) * 100.0);
    println!("⏳ Wait threshold:  {:.1}% (alert_threshold * 0.5)", sigmoid(-1.7346) * 0.5 * 100.0);
    println!("✅ Ignore: < {:.1}%", sigmoid(-1.7346) * 0.5 * 100.0);
    println!("⚖️ Wait range: {:.1}% - {:.1}%", sigmoid(-1.7346) * 0.5 * 100.0, sigmoid(-1.7346) * 100.0);
}

fn test_precise_wait_range() {
    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    
    println!("🔬 Testing events engineered to hit specific probability ranges...\n");
    
    // Target probabilities for testing
    let test_targets = vec![
        (0.050, "Far below wait threshold"),
        (0.070, "Just below wait threshold"),
        (0.075, "Exactly at wait threshold"),
        (0.080, "Just above wait threshold - WAIT expected"),
        (0.100, "Middle of wait range - WAIT expected"),
        (0.120, "Upper wait range - WAIT expected"),
        (0.145, "Just below alert threshold - WAIT expected"),
        (0.150, "Exactly at alert threshold"),
        (0.160, "Just above alert threshold - ALERT expected"),
        (0.200, "Well above alert threshold - ALERT expected"),
    ];
    
    for (i, (target_prob, description)) in test_targets.iter().enumerate() {
        let event = engineer_event_for_probability(*target_prob);
        
        match processor.process_event(&format!("test_home_{}", i), event) {
            Some(result) => {
                let actual_prob = result.calibrated_probability;
                let decision_str = match result.alert_decision {
                    insane_ai_security::thinking::AlertDecision::Alert => "🚨 ALERT",
                    insane_ai_security::thinking::AlertDecision::Wait => "⏳ WAIT",
                    insane_ai_security::thinking::AlertDecision::Ignore => "✅ IGNORE",
                };
                
                println!("Target: {:.1}% | Actual: {:.1}% | {} | {}", 
                    target_prob * 100.0, 
                    actual_prob * 100.0,
                    decision_str,
                    description
                );
                
                // Verify expected decision based on thresholds
                let expected_decision = if actual_prob >= 0.15 {
                    "ALERT"
                } else if actual_prob >= 0.075 {
                    "WAIT"
                } else {
                    "IGNORE"
                };
                
                let actual_decision = match result.alert_decision {
                    insane_ai_security::thinking::AlertDecision::Alert => "ALERT",
                    insane_ai_security::thinking::AlertDecision::Wait => "WAIT",
                    insane_ai_security::thinking::AlertDecision::Ignore => "IGNORE",
                };
                
                if expected_decision != actual_decision {
                    println!("  ⚠️ MISMATCH: Expected {} but got {}", expected_decision, actual_decision);
                }
            }
            None => println!("  🚨 FALLBACK: Processing failed - should default to Alert"),
        }
    }
}

/// Engineer an event to approximately hit a target probability
/// This is a rough approximation - exact calibration would require solving the inverse sigmoid
fn engineer_event_for_probability(target_prob: f64) -> Event {
    // Convert probability to logit
    let target_logit = if target_prob <= 0.001 {
        -6.0 // Very low logit for near-zero probabilities
    } else if target_prob >= 0.999 {
        6.0  // Very high logit for near-one probabilities
    } else {
        (target_prob / (1.0 - target_prob)).ln()
    };
    
    // The system uses: logit = prior_logit + sum_of_llrs
    // Default prior_logit is -2.0, so we need sum_of_llrs = target_logit - prior_logit
    let prior_logit = -2.0;
    let needed_llr_sum = target_logit - prior_logit;
    
    // Distribute the needed LLR sum across the 6 evidence components
    let base_llr = needed_llr_sum / 6.0;
    
    Event {
        ts: 0.0,
        cam: "TestCam".to_string(),
        person_track: format!("engineered_track_{:.0}", target_prob * 1000.0),
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

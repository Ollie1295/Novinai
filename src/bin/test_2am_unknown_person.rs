use insane_ai_security::thinking::{
    ThinkingAIProcessor, ThinkingAIConfig, Event, Evidence
};

fn main() {
    println!("🌙 2AM UNKNOWN PERSON - DUAL EVENT TEST");
    println!("========================================\n");
    println!("Scenario: Unknown person at front door at 2AM, two events 17 seconds apart");
    println!("Expected: High threat detection with escalating concern\n");

    let mut processor = ThinkingAIProcessor::new(ThinkingAIConfig::default());
    let home = "test_home_2am";
    
    // Event 1: Initial detection at 2:00:00 AM
    println!("⏰ EVENT 1 - 2:00:00 AM: Unknown person detected at front door");
    let event1 = Event {
        ts: 0.0, // Starting timestamp
        cam: "FrontDoorCam".to_string(),
        person_track: "track_unknown_2am".to_string(),
        rang_doorbell: false, // No doorbell - suspicious
        knocked: false,       // No knocking either
        dwell_s: 12.0,        // Loitering for 12 seconds
        away_prob: 0.85,      // High probability user is asleep/away
        expected_window: false, // No deliveries expected at 2AM
        token: None,          // No authentication token
        evidence: Evidence {
            llr_time: 1.2,      // 2AM is very suspicious time
            llr_entry: 0.8,     // Lurking without announcing
            llr_behavior: 1.0,  // Suspicious loitering behavior
            llr_identity: 1.4,  // Completely unknown person
            llr_presence: 0.7,  // High confidence user is asleep
            llr_token: 0.0,     // No token provided
        },
    };
    
    if let Some(result1) = processor.process_event(home, event1) {
        println!("\n🔍 ANALYSIS - EVENT 1:");
        println!("{}", processor.format_thinking_block(&result1));
        
        println!("\n📊 THREAT ASSESSMENT:");
        println!("• Calibrated probability: {:.1}%", result1.calibrated_probability * 100.0);
        println!("• Decision: {:?}", result1.alert_decision);
        
        if !result1.top_questions.is_empty() {
            println!("\n❓ Key Questions:");
            for (i, q) in result1.top_questions.iter().take(3).enumerate() {
                println!("  {}. {:?} (Value: {:.3})", i+1, q.q, q.expected_entropy_reduction);
            }
        }
        
        if !result1.counterfactuals.is_empty() {
            println!("\n🔄 What would change the decision:");
            for cf in result1.counterfactuals.iter().take(2) {
                println!("  • {} (ΔLLR: {:+.2})", cf.description, cf.delta_llr);
            }
        }
    }
    
    println!("\n============================================================");
    
    // Event 2: 17 seconds later - person still there, more concerning
    println!("\n⏰ EVENT 2 - 2:00:17 AM: Same person still at front door (17 seconds later)");
    let event2 = Event {
        ts: 17.0, // 17 seconds after first event
        cam: "FrontDoorCam".to_string(),
        person_track: "track_unknown_2am".to_string(), // Same person track
        rang_doorbell: false, // Still no doorbell
        knocked: true,        // NOW trying to knock - escalation!
        dwell_s: 25.0,        // Total dwell time now 25 seconds
        away_prob: 0.85,      // Still confident user is asleep
        expected_window: false,
        token: None,
        evidence: Evidence {
            llr_time: 1.3,      // Still 2AM, even more suspicious with time
            llr_entry: 1.2,     // More concerning - now knocking
            llr_behavior: 1.4,  // Escalating behavior - persistence + knocking
            llr_identity: 1.5,  // Still unknown, now more concerning
            llr_presence: 0.8,  // Even more confident user is sleeping
            llr_token: 0.0,
        },
    };
    
    if let Some(result2) = processor.process_event(home, event2) {
        println!("\n🔍 ANALYSIS - EVENT 2 (Updated incident):");
        println!("{}", processor.format_thinking_block(&result2));
        
        println!("\n📊 ESCALATED THREAT ASSESSMENT:");
        println!("• Calibrated probability: {:.1}%", result2.calibrated_probability * 100.0);
        println!("• Decision: {:?}", result2.alert_decision);
        
        if !result2.top_questions.is_empty() {
            println!("\n❓ Updated Strategic Questions:");
            for (i, q) in result2.top_questions.iter().take(3).enumerate() {
                println!("  {}. {:?} (Value: {:.3})", i+1, q.q, q.expected_entropy_reduction);
            }
        }
        
        if !result2.counterfactuals.is_empty() {
            println!("\n🔄 What would defuse the situation:");
            for cf in result2.counterfactuals.iter().take(3) {
                println!("  • {} (ΔLLR: {:+.2})", cf.description, cf.delta_llr);
            }
        }
        
        // Final assessment
        println!("\n🎯 FINAL ASSESSMENT:");
        if result2.calibrated_probability > 0.8 {
            println!("🚨 CRITICAL THREAT: Immediate security response recommended");
            println!("   → Unknown person persistent at 2AM with escalating behavior");
        } else if result2.calibrated_probability > 0.6 {
            println!("⚠️ HIGH THREAT: Security alert warranted");
            println!("   → Suspicious 2AM activity requires attention");
        } else if result2.calibrated_probability > 0.3 {
            println!("🔍 MODERATE THREAT: Monitor closely");
            println!("   → Unusual but not necessarily threatening");
        } else {
            println!("✅ LOW THREAT: Likely benign");
        }
        
        println!("\n🧠 THINKING AI REASONING:");
        println!("• Detected pattern escalation over 17-second window");
        println!("• Fused evidence from both events for comprehensive assessment");
        println!("• Generated actionable questions for human operators");
        println!("• Provided clear counterfactual explanations");
    }
    
    println!("\n============================================================");
    println!("✅ 2AM Unknown Person Scenario Complete");
}

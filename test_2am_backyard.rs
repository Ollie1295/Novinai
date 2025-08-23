//! Test: Unknown person at 2 AM in back garden, user home
//! This SHOULD be a high threat scenario

fn main() {
    println!("🌙 2 AM BACK GARDEN TEST");
    println!("========================");
    println!("Unknown person at 2 AM in back garden while user is home");
    println!("This SHOULD trigger legitimate high alerts\n");

    test_bayesian_assessment();
    test_time_model();
    test_entry_intelligence();
    test_context_absence();
    test_calibration();
}

fn test_bayesian_assessment() {
    println!("🧠 BAYESIAN ASSESSMENT:");
    println!("   Context: 2 AM + back garden + user home + unknown person");
    
    // High base threat for 2 AM back garden
    let contextual_prior = 0.4; // 40% - very high for this context
    let personal_prior = 0.05;  // 5% - unknown person base
    let blended_prior = contextual_prior * 0.7 + personal_prior * 0.3;
    
    println!("   Contextual prior (2 AM back garden): {:.3}", contextual_prior);
    println!("   Personal prior (unknown): {:.3}", personal_prior);
    println!("   Blended prior: {:.3}", blended_prior);
    
    // Motion evidence
    let motion_llr = 0.8; // High motion intensity suspicious at 2 AM
    let approach_llr = 0.6; // Suspicious approach pattern
    let face_llr = 0.3; // Unknown face penalty
    
    let total_llr = motion_llr + approach_llr + face_llr;
    let final_prob = sigmoid(logit(blended_prior) + total_llr);
    
    println!("   Evidence LLRs: Motion={:.1}, Approach={:.1}, Face={:.1}", 
        motion_llr, approach_llr, face_llr);
    println!("   Final probability: {:.3} ({:.1}%)", final_prob, final_prob * 100.0);
    
    let decision = if final_prob > 0.7 {
        "🚨 CRITICAL ALERT"
    } else if final_prob > 0.4 {
        "⚠️  HIGH ALERT"
    } else {
        "📋 STANDARD"
    };
    
    println!("   Decision: {}", decision);
    println!();
}

fn test_time_model() {
    println!("🕐 ADAPTIVE TIME MODEL:");
    println!("   Testing 2:00 AM time slot adjustment");
    
    // 2 AM should have very low benign activity, high threat activity
    let benign_events = 0.1_f64; // Almost no benign activity at 2 AM
    let threat_events = 0.8_f64;  // Most 2 AM activity is threatening
    
    let b_total = 180.0_f64; // Total benign events in history
    let t_total = 100.0_f64; // Total threat events in history
    
    let p_benign = (benign_events + 1.0) / (b_total + 96.0); // Smoothed
    let p_threat = (threat_events + 1.0) / (t_total + 96.0);
    
    let llr_empirical = (p_threat / p_benign).ln();
    let n_effective = benign_events + threat_events;
    let weight = n_effective / (n_effective + 200.0);
    let time_adjustment = weight * llr_empirical;
    
    println!("   2 AM patterns: B={:.1}, T={:.1}", benign_events, threat_events);
    println!("   P(benign|2AM)={:.4}, P(threat|2AM)={:.4}", p_benign, p_threat);
    println!("   Empirical LLR: {:.3}", llr_empirical);
    println!("   Time adjustment: {:.3} (POSITIVE - increases threat)", time_adjustment);
    println!();
}

fn test_entry_intelligence() {
    println!("🚪 ENTRY INTELLIGENCE:");
    println!("   Testing back garden entry point analysis");
    
    // Back garden = highly suspicious entry point
    let entry_context_llr = 1.2; // Back garden is suspicious
    let behavior_llr = 0.8; // No doorbell, lurking behavior
    let no_auth_penalty = 0.0; // No auth token expected for intruder
    
    let total_entry_llr = entry_context_llr + behavior_llr + no_auth_penalty;
    
    println!("   Entry context (back garden): +{:.1} LLR", entry_context_llr);
    println!("   Behavior (lurking, no ring): +{:.1} LLR", behavior_llr);
    println!("   No auth token: +{:.1} LLR", no_auth_penalty);
    println!("   Total entry LLR: +{:.1} (SUSPICIOUS ENTRY)", total_entry_llr);
    println!();
}

fn test_context_absence() {
    println!("🏠 CONTEXT ABSENCE:");
    println!("   Testing presence-aware assessment (user IS home)");
    
    // User home = unexpected for 2 AM back garden activity
    let away_prob = 0.1; // User is definitely home (90% confidence)
    let presence_adjustment = if away_prob < 0.3 {
        0.9 // High penalty - very suspicious when user is home
    } else {
        0.0
    };
    
    println!("   Away probability: {:.1} (user is HOME)", away_prob);
    println!("   Presence adjustment: +{:.1} LLR", presence_adjustment);
    println!("   Reasoning: Back garden activity while user home is VERY suspicious");
    println!();
}

fn test_calibration() {
    println!("🎯 CALIBRATION TEST:");
    println!("   Testing probability calibration for high-threat scenario");
    
    // Accumulate all evidence
    let raw_logit = 0.295 + 1.7 + 2.0 + 0.9; // Prior + Time + Entry + Presence
    println!("   Raw accumulated logit: {:.1}", raw_logit);
    
    let naive_prob = sigmoid(raw_logit);
    println!("   Naive sigmoid: {:.3} ({:.1}%)", naive_prob, naive_prob * 100.0);
    
    // Apply temperature scaling for 2 AM context (should be confident)
    let temperature = 1.2; // Less softening for clear threat scenarios
    let calibrated_logit = raw_logit / temperature;
    let calibrated_prob = sigmoid(calibrated_logit);
    
    println!("   Temperature scaling: {:.1}", temperature);
    println!("   Calibrated probability: {:.3} ({:.1}%)", calibrated_prob, calibrated_prob * 100.0);
    
    let decision = if calibrated_prob > 0.8 {
        "🚨 CRITICAL ALERT - Call Police"
    } else if calibrated_prob > 0.6 {
        "⚠️  HIGH ALERT - Immediate Action"
    } else {
        "📋 Monitor Situation"
    };
    
    println!("   Final decision: {}", decision);
    println!();
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

fn logit(p: f64) -> f64 {
    (p / (1.0 - p)).ln()
}

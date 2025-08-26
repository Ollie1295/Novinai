//! Debug AI Reasoning - Show Step-by-Step Calculations

use insane_ai_security::thinking::{ThinkingAIProcessor, ThinkingAIConfig, Event, Evidence, sigmoid, calibrate_logit};

fn main() {
    println!("🔍 DEBUG AI REASONING - STEP BY STEP ANALYSIS");
    println!("=============================================");
    
    let config = ThinkingAIConfig::default();
    println!("\n📊 CONFIG:");
    println!("   prior_logit: {}", config.prior_logit);
    println!("   alert_threshold_logit: {}", config.alert_threshold_logit);
    println!("   mean_logit: {}", config.mean_logit);
    println!("   temperature: {}", config.temperature);
    println!("   odds_cap: {}", config.odds_cap);
    println!("   pos_cap: {}", config.pos_cap);
    println!("   neg_cap: {}", config.neg_cap);
    
    // Test Case 1: Multiple Intruders (Should be HIGH threat)
    let intruder_evidence = Evidence {
        llr_time: 3.2,      // EXTREMELY suspicious time
        llr_entry: -3.5,    // Forced entry detected  
        llr_behavior: 3.8,  // HIGHLY aggressive behavior
        llr_identity: -2.8, // Multiple unknown people
        llr_presence: 2.5,  // Strong detection
        llr_token: -3.0,    // No authorization
    };
    
    println!("\n🔥 INTRUDER CASE ANALYSIS:");
    println!("========================");
    println!("Raw Evidence:");
    println!("   llr_time: {}", intruder_evidence.llr_time);
    println!("   llr_entry: {}", intruder_evidence.llr_entry);
    println!("   llr_behavior: {}", intruder_evidence.llr_behavior);
    println!("   llr_identity: {}", intruder_evidence.llr_identity);
    println!("   llr_presence: {}", intruder_evidence.llr_presence);
    println!("   llr_token: {}", intruder_evidence.llr_token);
    
    let intruder_sum = intruder_evidence.sum();
    println!("   Evidence Sum: {}", intruder_sum);
    
    let intruder_raw_logit = config.prior_logit + intruder_sum;
    println!("   Raw Logit: {} + {} = {}", config.prior_logit, intruder_sum, intruder_raw_logit);
    
    let intruder_calibrated = calibrate_logit(intruder_raw_logit, config.mean_logit, config.temperature, config.odds_cap);
    println!("   Calibrated Probability: {:.4} ({:.2}%)", intruder_calibrated, intruder_calibrated * 100.0);
    
    // Test Case 2: Family Member (Should be LOW threat)
    let family_evidence = Evidence {
        llr_time: -0.5,     // Normal time
        llr_entry: 1.2,     // Legitimate entry behavior
        llr_behavior: -1.8, // Normal behavior
        llr_identity: 2.1,  // Recognized family member
        llr_presence: 0.8,  // Normal presence
        llr_token: 1.5,     // Authorized access
    };
    
    println!("\n✅ FAMILY CASE ANALYSIS:");
    println!("=======================");
    println!("Raw Evidence:");
    println!("   llr_time: {}", family_evidence.llr_time);
    println!("   llr_entry: {}", family_evidence.llr_entry);
    println!("   llr_behavior: {}", family_evidence.llr_behavior);
    println!("   llr_identity: {}", family_evidence.llr_identity);
    println!("   llr_presence: {}", family_evidence.llr_presence);
    println!("   llr_token: {}", family_evidence.llr_token);
    
    let family_sum = family_evidence.sum();
    println!("   Evidence Sum: {}", family_sum);
    
    let family_raw_logit = config.prior_logit + family_sum;
    println!("   Raw Logit: {} + {} = {}", config.prior_logit, family_sum, family_raw_logit);
    
    let family_calibrated = calibrate_logit(family_raw_logit, config.mean_logit, config.temperature, config.odds_cap);
    println!("   Calibrated Probability: {:.4} ({:.2}%)", family_calibrated, family_calibrated * 100.0);
    
    // Check thresholds
    let alert_threshold = sigmoid(config.alert_threshold_logit);
    let wait_threshold = alert_threshold * 0.5;
    
    println!("\n🎯 THRESHOLD ANALYSIS:");
    println!("====================");
    println!("   Alert Threshold: {:.4} ({:.2}%)", alert_threshold, alert_threshold * 100.0);
    println!("   Wait Threshold: {:.4} ({:.2}%)", wait_threshold, wait_threshold * 100.0);
    println!("   Critical Threshold: 0.5000 (50.00%)");
    println!("   Elevated Threshold: 0.3000 (30.00%)");
    
    // Decision logic
    println!("\n⚡ DECISION ANALYSIS:");
    println!("===================");
    println!("Intruder Decision:");
    if intruder_calibrated >= 0.5 {
        println!("   🚨 CRITICAL (>= 50%)");
    } else if intruder_calibrated >= 0.3 {
        println!("   ⚠️ ELEVATED (>= 30%)");
    } else if intruder_calibrated >= alert_threshold {
        println!("   📢 STANDARD (>= {:.1}%)", alert_threshold * 100.0);
    } else if intruder_calibrated >= wait_threshold {
        println!("   ⏳ WAIT (>= {:.1}%)", wait_threshold * 100.0);
    } else {
        println!("   😴 IGNORE (< {:.1}%)", wait_threshold * 100.0);
    }
    
    println!("Family Decision:");
    if family_calibrated >= 0.5 {
        println!("   🚨 CRITICAL (>= 50%)");
    } else if family_calibrated >= 0.3 {
        println!("   ⚠️ ELEVATED (>= 30%)");
    } else if family_calibrated >= alert_threshold {
        println!("   📢 STANDARD (>= {:.1}%)", alert_threshold * 100.0);
    } else if family_calibrated >= wait_threshold {
        println!("   ⏳ WAIT (>= {:.1}%)", wait_threshold * 100.0);
    } else {
        println!("   😴 IGNORE (< {:.1}%)", wait_threshold * 100.0);
    }
}


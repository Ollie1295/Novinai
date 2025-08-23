//! AI Logic Problem Analysis
//! Identifying why the AI is overreacting to normal scenarios

use std::time::Instant;

fn main() {
    println!("🔍 AI LOGIC PROBLEM ANALYSIS");
    println!("============================");
    println!("Analyzing why AI assigned 0.873 threat level to routine scenario\n");

    // Recreate the exact calculation from the overreactive AI
    analyze_overreactive_logic();
    
    // Show what proper logic should look like
    println!("\n{}", "=".repeat(60));
    analyze_proper_logic();
}

fn analyze_overreactive_logic() {
    println!("❌ PROBLEM 1: EXCESSIVE BASE THREAT LEVEL");
    println!("==========================================");
    let base_threat = 0.3;
    println!("• Current AI starts with 0.3 (30%) base threat for ANY unknown person");
    println!("• PROBLEM: This assumes every unknown person is suspicious by default");
    println!("• REALITY: Most unknown people (delivery, neighbors, etc.) are benign");
    println!("• SHOULD BE: 0.05-0.1 base threat for unknown persons");
    println!("• IMPACT: Immediately biases system toward high alerts\n");

    println!("❌ PROBLEM 2: TIME-OF-DAY OVERWEIGHTING");
    println!("========================================");
    let time_penalty = 0.15;
    println!("• Current AI adds +0.15 (15%) threat for 8 PM");
    println!("• PROBLEM: 8 PM is normal evening hours, not suspicious");
    println!("• REALITY: People commonly arrive home, visit, or make deliveries until 9-10 PM");
    println!("• SHOULD BE: 0.0 penalty for 6 PM - 10 PM, minor penalty after 11 PM");
    println!("• IMPACT: Punishes normal social hours\n");

    println!("❌ PROBLEM 3: USER ABSENCE CATASTROPHIZING");
    println!("===========================================");
    let user_away_penalty = 0.25;
    println!("• Current AI adds +0.25 (25%) threat when user is away");
    println!("• PROBLEM: This is the largest single penalty, assuming absence = danger");
    println!("• REALITY: Many legitimate visits occur when owners are away (appointments, deliveries)");
    println!("• SHOULD BE: +0.05-0.1 for unknown persons, 0.0 for recognized delivery/service");
    println!("• IMPACT: Makes normal life activities seem threatening\n");

    println!("❌ PROBLEM 4: UNKNOWN IDENTITY DOUBLE-PENALIZATION"); 
    println!("===================================================");
    let unknown_penalty = 0.2;
    println!("• Current AI adds +0.2 (20%) for unknown identity");
    println!("• PROBLEM: This double-counts with the base threat already accounting for unknown status");
    println!("• REALITY: Being unknown doesn't automatically mean threatening");
    println!("• SHOULD BE: Integrated into base threat calculation, not added separately");
    println!("• IMPACT: Double-penalizes the same factor\n");

    println!("❌ PROBLEM 5: LOCATION BIAS");
    println!("============================");
    let location_penalty = 0.15;
    println!("• Current AI adds +0.15 (15%) for front door location");
    println!("• PROBLEM: Front door is the NORMAL place for legitimate visitors");
    println!("• REALITY: Legitimate visitors use front doors; intruders avoid them");
    println!("• SHOULD BE: 0.0 penalty for front door, +0.3 for windows/back entrances");
    println!("• IMPACT: Penalizes normal, expected behavior\n");

    println!("❌ PROBLEM 6: BEHAVIORAL INDICATOR STACKING");
    println!("============================================");
    let behavior_penalties = vec![
        ("approaching_door", 0.1),
        ("unknown_identity", 0.05), // Double counting again!
        ("after_hours", 0.1),
    ];
    let total_behavior = behavior_penalties.iter().map(|(_, v)| v).sum::<f64>();
    println!("• Current AI stacks behavioral penalties: +{:.2}", total_behavior);
    for (behavior, penalty) in &behavior_penalties {
        println!("  - '{}': +{:.2}", behavior, penalty);
    }
    println!("• PROBLEM: 'Unknown identity' counted again, 'after_hours' wrong for 8 PM");
    println!("• PROBLEM: 'Approaching door' is NORMAL visitor behavior");
    println!("• REALITY: These are signs of legitimate visitors, not threats");
    println!("• IMPACT: Punishes expected, normal behaviors\n");

    println!("❌ PROBLEM 7: AGGRESSIVE CALIBRATION");
    println!("=====================================");
    let raw_threat = base_threat + time_penalty + user_away_penalty + unknown_penalty + location_penalty + total_behavior;
    println!("• Raw threat calculation: {:.3}", raw_threat);
    let calibrated = 1.0 / (1.0 + (-2.5 * (raw_threat - 0.5)).exp());
    println!("• After calibration: {:.3}", calibrated);
    println!("• PROBLEM: Calibration function amplifies moderate scores into critical alerts");
    println!("• PROBLEM: Uses aggressive sigmoid that pushes scores toward extremes");
    println!("• IMPACT: Turns moderately suspicious (0.6-0.7) into critical alerts (0.87+)\n");

    println!("📊 CUMULATIVE IMPACT ANALYSIS");
    println!("==============================");
    println!("Starting threat: {:.3}", base_threat);
    let mut running_total = base_threat;
    
    running_total += time_penalty;
    println!("+ Time penalty: {:.3} → {:.3}", time_penalty, running_total);
    
    running_total += user_away_penalty;
    println!("+ User away: {:.3} → {:.3}", user_away_penalty, running_total);
    
    running_total += unknown_penalty;
    println!("+ Unknown person: {:.3} → {:.3}", unknown_penalty, running_total);
    
    running_total += location_penalty;
    println!("+ Front door: {:.3} → {:.3}", location_penalty, running_total);
    
    running_total += total_behavior;
    println!("+ Behaviors: {:.3} → {:.3}", total_behavior, running_total);
    
    // Apply camera confidence
    running_total *= 0.89;
    println!("× Camera confidence (0.89): → {:.3}", running_total);
    
    // Add motion intensity
    running_total += 0.65 * 0.1;
    println!("+ Motion intensity: {:.3} → {:.3}", 0.065, running_total);
    
    // Duration
    running_total += 0.05;
    println!("+ Duration >10s: 0.05 → {:.3}", running_total);
    
    println!("= FINAL RAW SCORE: {:.3}", running_total);
    
    let final_calibrated = 1.0 / (1.0 + (-2.5 * (running_total - 0.5)).exp());
    println!("= AFTER CALIBRATION: {:.3} (CRITICAL ALERT)", final_calibrated);
}

fn analyze_proper_logic() {
    println!("✅ CORRECTED AI LOGIC");
    println!("======================");
    
    // Proper base threat levels by context
    println!("🎯 PROPER BASE THREAT LEVELS:");
    println!("• Unknown person, daytime, user home: 0.05");
    println!("• Unknown person, evening (6-10 PM), user away: 0.10");  
    println!("• Unknown person, late night (11 PM-6 AM): 0.25");
    println!("• Recognized delivery person: 0.02");
    println!("• Recognized neighbor/friend: 0.01\n");

    println!("🕒 PROPER TIME WEIGHTING:");
    println!("• 6 AM - 10 PM: +0.0 (normal hours)");
    println!("• 10 PM - 11 PM: +0.05 (getting late)");  
    println!("• 11 PM - 6 AM: +0.15-0.3 (suspicious hours)\n");

    println!("🏠 PROPER LOCATION WEIGHTING:");
    println!("• Front door/main entrance: +0.0 (expected)");
    println!("• Side/service entrances: +0.05");
    println!("• Windows/back areas: +0.2-0.4 (suspicious)\n");

    println!("👤 PROPER USER STATUS IMPACT:");
    println!("• User home: +0.0");
    println!("• User away, expected visitor: +0.0"); 
    println!("• User away, unknown person: +0.05-0.1");
    println!("• User away, suspicious behavior: +0.2+\n");

    println!("🎭 PROPER BEHAVIORAL ANALYSIS:");
    println!("• Normal approach patterns: 0.0");
    println!("• Lingering/pacing: +0.1");
    println!("• Attempting entry: +0.3");
    println!("• Carrying tools/weapons: +0.5");
    println!("• Concealment attempts: +0.4\n");

    // Calculate what the threat SHOULD be
    let proper_base = 0.10; // Unknown person, 8 PM, user away
    let proper_time = 0.0;   // 8 PM is normal
    let proper_location = 0.0; // Front door is expected  
    let proper_user_away = 0.05; // Minor increase for unknown + away
    let proper_behavior = 0.0;   // Approaching door is normal
    
    let proper_total = proper_base + proper_time + proper_location + proper_user_away + proper_behavior;
    
    println!("✅ CORRECTED CALCULATION FOR THIS SCENARIO:");
    println!("Base (unknown, evening, away): {:.3}", proper_base);
    println!("+ Time penalty (8 PM): {:.3}", proper_time);
    println!("+ Location (front door): {:.3}", proper_location);  
    println!("+ User away unknown: {:.3}", proper_user_away);
    println!("+ Behavior (normal approach): {:.3}", proper_behavior);
    println!("= PROPER TOTAL: {:.3}", proper_total);
    
    let proper_alert = match proper_total {
        t if t >= 0.8 => "CRITICAL",
        t if t >= 0.6 => "HIGH",
        t if t >= 0.4 => "ELEVATED", 
        t if t >= 0.2 => "STANDARD",
        _ => "LOW",
    };
    
    println!("= PROPER ALERT LEVEL: {} ✅", proper_alert);
    println!("\nThis matches your assessment that it should be STANDARD/ELEVATED at most!");
}

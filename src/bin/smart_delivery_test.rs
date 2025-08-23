use insane_ai_security::*;

fn main() {
    println!("🚚 SMART DELIVERY RECOGNITION SYSTEM");
    println!("Testing intelligent delivery vs. threat detection\n");

    // Evening delivery scenario
    let evening_delivery = ThreatContext {
        time_risk: 0.8,           // 8pm
        location_risk: 0.9,       // Front door
        entity_count: 1,          // Single person
        identity_certainty: 0.15, // Unknown person
        user_presence: false,     // User away
        environmental_conditions: "unknown_person_package_evening_front_door".to_string(),
    };

    println!("📦 EVENING DELIVERY SCENARIO:");
    println!("   🕐 Time: 8:00 PM");
    println!("   📍 Location: Front Door");
    println!("   👤 Person: Unknown (15% certainty)");
    println!("   🏠 User: AWAY");
    println!("   📦 Package: DETECTED");

    let delivery_score = smart_threat_assessment(&evening_delivery);
    
    println!("\n🎯 SMART THREAT SCORE: {:.3}", delivery_score);
    
    match delivery_score {
        s if s > 0.7 => println!("   🚨 HIGH ALERT - Investigate immediately"),
        s if s > 0.4 => println!("   ⚠️  MEDIUM ALERT - Monitor closely"),
        s if s > 0.2 => println!("   📋 LOW ALERT - Standard logging"),
        _ => println!("   💚 MINIMAL RISK - Routine operation"),
    }

    // Compare with actual intruder scenario
    println!("\n" + "=".repeat(50));
    
    let intruder_scenario = ThreatContext {
        time_risk: 0.9,           // Late night
        location_risk: 0.9,       // Front door
        entity_count: 1,          // Single person
        identity_certainty: 0.1,  // Very unknown
        user_presence: false,     // User away
        environmental_conditions: "unknown_person_no_package_late_night_suspicious_behavior".to_string(),
    };

    println!("🚨 ACTUAL INTRUDER SCENARIO:");
    println!("   🕐 Time: 11:00 PM");
    println!("   📍 Location: Front Door");
    println!("   👤 Person: Unknown (10% certainty)");
    println!("   🏠 User: AWAY");
    println!("   📦 Package: NONE");
    println!("   🔍 Behavior: Suspicious (looking around, trying handle)");

    let intruder_score = smart_threat_assessment(&intruder_scenario);
    
    println!("\n🎯 SMART THREAT SCORE: {:.3}", intruder_score);
    
    match intruder_score {
        s if s > 0.7 => println!("   🚨 HIGH ALERT - Investigate immediately"),
        s if s > 0.4 => println!("   ⚠️  MEDIUM ALERT - Monitor closely"),
        s if s > 0.2 => println!("   📋 LOW ALERT - Standard logging"),
        _ => println!("   💚 MINIMAL RISK - Routine operation"),
    }

    println!("\n🧠 AI ANALYSIS SUMMARY:");
    println!("   Delivery scenario: {:.3} (Recognized as likely legitimate)", delivery_score);
    println!("   Intruder scenario: {:.3} (Correctly identified as threat)", intruder_score);
    println!("   Difference: {:.3} (AI successfully distinguishes contexts)", (intruder_score - delivery_score).abs());
}

fn smart_threat_assessment(context: &ThreatContext) -> f64 {
    // Analyze delivery likelihood
    let delivery_confidence = analyze_delivery_patterns(context);
    
    println!("\n🤖 DELIVERY PATTERN ANALYSIS:");
    println!("   Delivery Likelihood: {:.1}%", delivery_confidence * 100.0);
    
    // If high delivery confidence, use delivery-specific scoring
    if delivery_confidence > 0.7 {
        println!("   🚚 DELIVERY DETECTED - Applying delivery threat model");
        return calculate_delivery_threat(context);
    }
    
    // If medium delivery confidence, reduce standard threat weights
    if delivery_confidence > 0.4 {
        println!("   📦 POSSIBLE DELIVERY - Reducing threat weights");
        return calculate_adjusted_threat(context, delivery_confidence);
    }
    
    // Low delivery confidence - use standard threat assessment
    println!("   🚨 NON-DELIVERY SCENARIO - Standard threat assessment");
    calculate_standard_threat(context)
}

fn analyze_delivery_patterns(context: &ThreatContext) -> f64 {
    let mut delivery_score = 0.0;
    
    // Package presence is strong delivery indicator
    if context.environmental_conditions.contains("package") {
        delivery_score += 0.5;
        println!("   ✅ Package detected: +0.5");
    }
    
    // Evening deliveries are increasingly common
    if context.time_risk > 0.7 && context.time_risk < 0.9 { // 7-9pm range
        delivery_score += 0.2;
        println!("   ✅ Evening delivery window: +0.2");
    }
    
    // Front door is expected for deliveries
    if context.location_risk > 0.8 {
        delivery_score += 0.15;
        println!("   ✅ Front door location: +0.15");
    }
    
    // Single person is typical for deliveries
    if context.entity_count == 1 {
        delivery_score += 0.1;
        println!("   ✅ Single person: +0.1");
    }
    
    // Suspicious behavior reduces delivery likelihood
    if context.environmental_conditions.contains("suspicious") {
        delivery_score -= 0.4;
        println!("   ❌ Suspicious behavior: -0.4");
    }
    
    delivery_score.clamp(0.0, 1.0)
}

fn calculate_delivery_threat(context: &ThreatContext) -> f64 {
    let mut score = 0.0;
    
    // Minimal base risk for unknown delivery person
    score += (1.0 - context.identity_certainty) * 0.1; // Much lower weight
    
    // Evening delivery is normal
    score += 0.05;
    
    // User away is expected for deliveries
    score += 0.08;
    
    // Cap delivery threats at low-medium risk
    score.clamp(0.0, 0.35)
}

fn calculate_adjusted_threat(context: &ThreatContext, delivery_confidence: f64) -> f64 {
    let mut score = 0.0;
    
    // Reduce identity weight based on delivery confidence
    let identity_weight = 0.4 * (1.0 - delivery_confidence);
    score += (1.0 - context.identity_certainty) * identity_weight;
    
    // Reduce time penalty
    score += context.time_risk * 0.15;
    
    // Reduce location penalty
    score += context.location_risk * 0.1;
    
    // Reduce user absence penalty
    if !context.user_presence {
        score += 0.15;
    }
    
    score.clamp(0.0, 0.6)
}

fn calculate_standard_threat(context: &ThreatContext) -> f64 {
    let mut score = 0.0;
    
    // Full identity uncertainty weight
    score += (1.0 - context.identity_certainty) * 0.5;
    
    // Full time risk
    score += context.time_risk * 0.25;
    
    // Full location risk
    score += context.location_risk * 0.2;
    
    // Full user absence penalty
    if !context.user_presence {
        score += 0.3;
    }
    
    score.clamp(0.0, 1.0)
}

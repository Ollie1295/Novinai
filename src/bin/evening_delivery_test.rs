use insane_ai_security::*;

fn main() {
    println!("🌙 DYNAMIC EVENING DELIVERY THREAT ASSESSMENT");
    println!("📍 Real-time scenario analysis\n");

    let mut system = InsaneSecuritySystem::default();
    
    // Test the specific scenario you requested
    println!("🎯 SCENARIO: Unknown person front door 8pm package detected user presence away");
    let scenario = create_dynamic_threat_context(
        "unknown_person",
        "front_door", 
        "8pm",
        true,  // package detected
        false  // user away
    );
    
    analyze_dynamic_scenario("Evening Package Scenario", &scenario, &mut system);
    
    println!("\n{}", "=".repeat(70));
    
    // Compare with a different scenario for contrast
    println!("🔄 COMPARISON SCENARIO: Known person, daytime, no package, user home");
    let comparison = create_dynamic_threat_context(
        "family_member",
        "front_door",
        "2pm", 
        false, // no package
        true   // user home
    );
    
    analyze_dynamic_scenario("Daytime Family Visit", &comparison, &mut system);
    
    println!("\n🧠 AI SYSTEM ANALYSIS COMPLETE");
}

fn create_dynamic_threat_context(
    person_type: &str,
    location: &str, 
    time: &str,
    package_detected: bool,
    user_home: bool
) -> ThreatContext {
    // Calculate dynamic risk factors based on inputs
    let time_risk = calculate_time_risk(time);
    let location_risk = calculate_location_risk(location);
    let identity_certainty = calculate_identity_certainty(person_type);
    
    let mut environmental_conditions = format!("{}_{}", person_type, location);
    if package_detected {
        environmental_conditions.push_str("_package");
    }
    environmental_conditions.push_str(&format!("_{}", time));
    
    ThreatContext {
        time_risk,
        location_risk,
        entity_count: 1,
        identity_certainty,
        user_presence: user_home,
        environmental_conditions,
    }
}

fn calculate_time_risk(time: &str) -> f64 {
    match time {
        "8pm" | "9pm" | "10pm" => 0.8,  // Evening - high risk
        "11pm" | "12am" | "1am" | "2am" => 0.95, // Late night - very high risk
        "6am" | "7am" => 0.6,   // Early morning - medium risk
        "12pm" | "1pm" | "2pm" | "3pm" => 0.2,   // Afternoon - low risk
        "4pm" | "5pm" | "6pm" => 0.4,   // Evening approach - medium risk
        _ => 0.5 // Default medium risk
    }
}

fn calculate_location_risk(location: &str) -> f64 {
    match location {
        "front_door" => 0.9,    // High risk - main entry
        "back_door" => 0.85,    // High risk - less visible
        "window" => 0.95,       // Very high risk - unusual entry
        "driveway" => 0.6,      // Medium risk
        "yard" => 0.4,          // Lower risk
        _ => 0.7 // Default
    }
}

fn calculate_identity_certainty(person_type: &str) -> f64 {
    match person_type {
        "family_member" => 0.95,      // Very certain
        "neighbor" => 0.8,            // High certainty  
        "delivery_person" => 0.7,     // Medium-high certainty
        "maintenance_worker" => 0.6,  // Medium certainty
        "unknown_person" => 0.15,     // Very low certainty - major risk
        "suspicious_person" => 0.05,  // Extremely low - critical risk
        _ => 0.5 // Default medium
    }
}

fn analyze_dynamic_scenario(name: &str, context: &ThreatContext, _system: &mut InsaneSecuritySystem) {
    println!("\n📊 ANALYZING: {}", name);
    println!("   📋 Dynamic Context:");
    
    // Parse time from environmental conditions for display
    let time_str = extract_time_from_context(&context.environmental_conditions);
    println!("   🕐 Time: {} (Risk: {:.0}%)", time_str, context.time_risk * 100.0);
    
    let location_str = extract_location_from_context(&context.environmental_conditions);
    println!("   📍 Location: {} (Risk: {:.0}%)", location_str, context.location_risk * 100.0);
    
    let person_str = extract_person_from_context(&context.environmental_conditions);
    println!("   👤 Person: {} (Certainty: {:.0}%)", person_str, context.identity_certainty * 100.0);
    
    println!("   🏠 User: {}", if context.user_presence { "HOME" } else { "AWAY" });
    
    let package_detected = context.environmental_conditions.contains("package");
    println!("   📦 Package: {}", if package_detected { "DETECTED" } else { "NONE" });
    
    let threat_score = calculate_dynamic_threat_score(context);
    println!("\n   🎯 DYNAMIC THREAT SCORE: {:.3}", threat_score);
    
    // Dynamic threat classification
    let (alert_level, color) = match threat_score {
        s if s > 0.85 => ("🚨 CRITICAL ALERT", "IMMEDIATE RESPONSE REQUIRED"),
        s if s > 0.7  => ("⚠️  HIGH ALERT", "Enhanced monitoring needed"),
        s if s > 0.5  => ("📋 MEDIUM ALERT", "Standard monitoring"),
        s if s > 0.3  => ("✅ LOW ALERT", "Routine logging"),
        _ => ("💚 MINIMAL RISK", "Normal operation")
    };
    
    println!("   {} - {}", alert_level, color);
    
    // Show dynamic risk factors
    println!("\n   📈 Risk Factor Analysis:");
    let identity_risk = (1.0 - context.identity_certainty) * 0.4;
    println!("      • Identity Risk: +{:.3}", identity_risk);
    
    let time_factor = context.time_risk * 0.25;
    println!("      • Time Factor: +{:.3}", time_factor);
    
    let location_factor = context.location_risk * 0.2;
    println!("      • Location Factor: +{:.3}", location_factor);
    
    if !context.user_presence {
        println!("      • User Away: +0.300 (CRITICAL)");
    }
    
    if package_detected {
        let package_risk = if !context.user_presence && context.identity_certainty < 0.3 { 
            0.25 
        } else { 
            0.05 
        };
        println!("      • Package Risk: +{:.3}", package_risk);
    }
}

fn calculate_evening_threat_score(context: &ThreatContext) -> f64 {
    // Smart delivery pattern recognition
    let delivery_confidence = analyze_delivery_legitimacy(context);
    
    println!("   🤖 AI DELIVERY ANALYSIS:");
    println!("      Delivery Likelihood: {:.1}%", delivery_confidence * 100.0);
    
    // If high confidence this is a delivery, apply delivery-specific scoring
    if delivery_confidence > 0.7 {
        println!("      🚚 DELIVERY DETECTED - Applying delivery-specific threat model");
        return calculate_delivery_threat_score(context, delivery_confidence);
    }
    
    // Otherwise use standard threat assessment but with contextual adjustments
    let mut score = 0.0;
    
    // Identity uncertainty (but less weight for potential deliveries)
    let identity_weight = if delivery_confidence > 0.4 { 0.3 } else { 0.5 };
    score += (1.0 - context.identity_certainty) * identity_weight;
    
    // Time risk (evening is normal for deliveries)
    let time_weight = if delivery_confidence > 0.4 { 0.1 } else { 0.25 };
    score += context.time_risk * time_weight;
    
    // Location risk (front door is expected for deliveries)
    let location_weight = if delivery_confidence > 0.4 { 0.05 } else { 0.2 };
    score += context.location_risk * location_weight;
    
    // User absence (less critical for deliveries)
    if !context.user_presence {
        let absence_penalty = if delivery_confidence > 0.4 { 0.1 } else { 0.3 };
        score += absence_penalty;
    }
    
    score.clamp(0.0, 1.0)
}

fn calculate_delivery_threat_score(context: &ThreatContext, delivery_confidence: f64) -> f64 {
    let mut score = 0.0;
    
    // Base risk for any unknown person (but much lower for deliveries)
    score += (1.0 - context.identity_certainty) * 0.15;
    
    // Evening delivery is increasingly common
    score += context.time_risk * 0.05;
    
    // Front door is expected location
    score += 0.02;
    
    // User away is normal for deliveries
    if !context.user_presence {
        score += 0.08; // Minimal penalty
    }
    
    // Package presence actually reduces threat for deliveries
    if context.environmental_conditions.contains("package") {
        score -= 0.05; // Negative adjustment - packages support delivery theory
    }
    
    // Cap delivery scenarios at medium-low risk
    let final_score = score.clamp(0.0, 0.45);
    
    println!("      📊 Delivery-Specific Scoring:");
    println!("         Base Unknown Person: +{:.3}", (1.0 - context.identity_certainty) * 0.15);
    println!("         Evening Time: +{:.3}", context.time_risk * 0.05);
    println!("         Location: +0.020");
    println!("         User Away: +0.080");
    println!("         Package Bonus: -0.050 (supports delivery)");
    println!("         Final Capped Score: {:.3}", final_score);
    
    final_score
}

fn extract_time_from_context(conditions: &str) -> &str {
    if conditions.contains("8pm") { "8:00 PM" }
    else if conditions.contains("2pm") { "2:00 PM" }
    else if conditions.contains("9pm") { "9:00 PM" }
    else if conditions.contains("12pm") { "12:00 PM" }
    else if conditions.contains("6am") { "6:00 AM" }
    else { "Unknown Time" }
}

fn extract_location_from_context(conditions: &str) -> &str {
    if conditions.contains("front_door") { "Front Door" }
    else if conditions.contains("back_door") { "Back Door" }
    else if conditions.contains("window") { "Window" }
    else if conditions.contains("driveway") { "Driveway" }
    else { "Unknown Location" }
}

fn extract_person_from_context(conditions: &str) -> &str {
    if conditions.contains("unknown_person") { "Unknown Person" }
    else if conditions.contains("family_member") { "Family Member" }
    else if conditions.contains("delivery_person") { "Delivery Person" }
    else if conditions.contains("neighbor") { "Neighbor" }
    else { "Unknown Person Type" }
}

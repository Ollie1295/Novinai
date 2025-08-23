use insane_ai_security::*;

fn main() {
    println!("📦 AI Security System - Delivery Scenario Test\n");

    let system = InsaneSecuritySystem::default();
    
    // Test delivery scenario
    let delivery_context = create_delivery_context();
    test_delivery_scenario(&delivery_context, &system);
    
    println!("✅ Delivery scenario test completed!");
}

fn create_delivery_context() -> ThreatContext {
    ThreatContext {
        time_risk: 0.8,
        location_risk: 0.9,
        entity_count: 2,
        identity_certainty: 0.65,
        user_presence: false,
        environmental_conditions: "delivery_scenario".to_string(),
    }
}

fn test_delivery_scenario(context: &ThreatContext, _system: &InsaneSecuritySystem) {
    println!("🧪 Testing: Delivery Scenario");
    
    let threat_score = calculate_delivery_threat_score(context);
    
    println!("   Time Risk: {:.1}%", context.time_risk * 100.0);
    println!("   Location Risk: {:.1}%", context.location_risk * 100.0);
    println!("   Entity Count: {}", context.entity_count);
    println!("   Identity Certainty: {:.1}%", context.identity_certainty * 100.0);
    println!("   Environmental: {}", context.environmental_conditions);
    println!("   Threat Score: {:.2}", threat_score);
    
    if threat_score > 0.7 {
        println!("   🚨 HIGH ALERT - Possible threat");
    } else if threat_score > 0.4 {
        println!("   ⚠️  Medium Alert - Monitor situation");
    } else {
        println!("   ✅ Low Risk - Normal delivery");
    }
}

fn calculate_delivery_threat_score(context: &ThreatContext) -> f64 {
    let mut score: f64 = 0.4; // Base unknown person score
    
    // Adjust based on environmental conditions
    if context.environmental_conditions.contains("delivery") {
        score -= 0.15; // Lower threat for delivery context
    }
    
    // Time and location factors  
    score += context.time_risk * 0.2;
    score += context.location_risk * 0.2;
    
    // Identity certainty
    score += (1.0 - context.identity_certainty) * 0.3;
    
    // Multiple entities can increase risk slightly
    if context.entity_count > 1 {
        score += 0.1;
    }
    
    score.max(0.0).min(1.0)
}

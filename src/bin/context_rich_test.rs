use insane_ai_security::*;

fn main() {
    println!("🔍 AI Security System - Context-Rich Delivery Scenario Test\n");

    let mut system = InsaneSecuritySystem::default();
    
    // Test delivery scenario
    let delivery_context = create_delivery_context();
    test_delivery_scenario(&delivery_context, &mut system);
    
    println!("✅ Context-rich test completed!");
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

fn test_delivery_scenario(context: &ThreatContext, _system: &mut InsaneSecuritySystem) {
    println!("🧪 Testing: Delivery Context");
    
    let threat_score = calculate_delivery_threat_score(context);
    
    println!("   Time Risk: {:.1}%", context.time_risk * 100.0);
    println!("   Location Risk: {:.1}%", context.location_risk * 100.0);
    println!("   Entity Count: {}", context.entity_count);
    println!("   Identity Certainty: {:.1}%", context.identity_certainty * 100.0);
    println!("   Threat Score: {:.2}", threat_score);
    
    if threat_score > 0.7 {
        println!("   🚨 HIGH ALERT");
    } else if threat_score > 0.4 {
        println!("   ⚠️  Medium Alert");
    } else {
        println!("   ✅ Low Risk");
    }
}

fn calculate_delivery_threat_score(context: &ThreatContext) -> f64 {
    let mut score: f64 = 0.4; // Base unknown person score
    
    // Adjust based on delivery indicators
    if context.environmental_conditions.contains("delivery") {
        score -= 0.2; // Lower threat for delivery context
    }
    
    // Time and location factors
    score += context.time_risk * 0.3;
    score += context.location_risk * 0.3;
    
    // Identity certainty
    score += (1.0 - context.identity_certainty) * 0.4;
    
    score.max(0.0).min(1.0)
}

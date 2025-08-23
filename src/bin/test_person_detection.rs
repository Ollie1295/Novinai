use insane_ai_security::*;

fn main() {
    println!("🔍 AI Security System - Person Detection Tests\n");

    let mut system = InsaneSecuritySystem::default();
    
    // Test 1: Known family member
    let family_context = create_person_context("family_member", 0.95);
    test_scenario("Known Family Member", &family_context, &mut system);
    
    // Test 2: Unknown person
    let unknown_context = create_person_context("unknown_person", 0.85);
    test_scenario("Unknown Person", &unknown_context, &mut system);
    
    // Test 3: Delivery person
    let delivery_context = create_person_context("delivery_person", 0.75);
    test_scenario("Delivery Person", &delivery_context, &mut system);
    
    println!("✅ All person detection tests completed!");
}

fn create_person_context(person_type: &str, confidence: f64) -> ThreatContext {
    ThreatContext {
        time_risk: 0.7,
        location_risk: 0.6,
        entity_count: 1,
        identity_certainty: confidence,
        user_presence: false,
        environmental_conditions: person_type.to_string(),
    }
}

fn test_scenario(name: &str, context: &ThreatContext, _system: &mut InsaneSecuritySystem) {
    println!("🧪 Testing: {}", name);
    
    // Calculate threat score (simplified)
    let threat_score = calculate_threat_score(context);
    
    println!("   Identity Certainty: {:.1}%", context.identity_certainty * 100.0);
    println!("   Context: {}", context.environmental_conditions);
    println!("   Threat Score: {:.2}", threat_score);
    
    if threat_score > 0.7 {
        println!("   🚨 HIGH ALERT");
    } else if threat_score > 0.4 {
        println!("   ⚠️  Medium Alert");
    } else {
        println!("   ✅ Low Risk");
    }
    
    println!();
}

fn calculate_threat_score(context: &ThreatContext) -> f64 {
    let mut score = 0.0;
    
    // Base score from identity certainty
    score += (1.0 - context.identity_certainty) * 0.5;
    
    // Environmental factors
    if context.environmental_conditions.contains("unknown") {
        score += 0.3;
    }
    
    // Time and location risk
    score += context.time_risk * 0.2;
    score += context.location_risk * 0.2;
    
    score.min(1.0)
}

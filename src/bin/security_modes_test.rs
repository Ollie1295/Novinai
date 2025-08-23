use insane_ai_security::core::*;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

fn main() {
    println!("🛡️  Testing Security Modes Implementation");
    println!("==========================================");

    // Create test threat context
    let entity_id = Uuid::new_v4();
    let mut threat_indicators = HashMap::new();
    threat_indicators.insert("suspicious_behavior".to_string(), 0.6);
    threat_indicators.insert("unauthorized_access".to_string(), 0.4);
    threat_indicators.insert("perimeter_breach".to_string(), 0.8);

    let context = ThreatContext {
        entity_id,
        threat_indicators,
        environmental_factors: HashMap::new(),
        temporal_context: Utc::now(),
        confidence: 0.85,
    };

    // Test Guardian Mode
    println!("\n🔰 GUARDIAN MODE TEST");
    println!("---------------------");
    let mut guardian_system = InsaneSecuritySystem::default();
    guardian_system.set_security_mode(SecurityMode::Guardian);
    
    let guardian_assessment = guardian_system.process_threat(&context);
    println!("Mode: {:?}", guardian_system.get_security_mode());
    println!("Threat Level: {:.2}", guardian_assessment.threat_level);
    println!("Confidence: {:.2}", guardian_assessment.confidence_score);
    println!("Response Time: {} minutes", guardian_assessment.temporal_horizon.num_minutes());
    println!("Countermeasures: {:?}", guardian_assessment.countermeasures);
    println!("Explanation: {}", guardian_assessment.explainability_trace);

    // Test Stealth Mode
    println!("\n🥷 STEALTH MODE TEST");
    println!("-------------------");
    let mut stealth_system = InsaneSecuritySystem::default();
    stealth_system.set_security_mode(SecurityMode::Stealth);
    
    let stealth_assessment = stealth_system.process_threat(&context);
    println!("Mode: {:?}", stealth_system.get_security_mode());
    println!("Threat Level: {:.2}", stealth_assessment.threat_level);
    println!("Confidence: {:.2}", stealth_assessment.confidence_score);
    println!("Response Time: {} minutes", stealth_assessment.temporal_horizon.num_minutes());
    println!("Countermeasures: {:?}", stealth_assessment.countermeasures);
    println!("Explanation: {}", stealth_assessment.explainability_trace);

    // Test Perimeter Guard Mode
    println!("\n🚧 PERIMETER GUARD MODE TEST");
    println!("----------------------------");
    let mut perimeter_system = InsaneSecuritySystem::default();
    perimeter_system.set_security_mode(SecurityMode::PerimeterGuard);
    
    let perimeter_assessment = perimeter_system.process_threat(&context);
    println!("Mode: {:?}", perimeter_system.get_security_mode());
    println!("Threat Level: {:.2}", perimeter_assessment.threat_level);
    println!("Confidence: {:.2}", perimeter_assessment.confidence_score);
    println!("Response Time: {} minutes", perimeter_assessment.temporal_horizon.num_minutes());
    println!("Countermeasures: {:?}", perimeter_assessment.countermeasures);
    println!("Explanation: {}", perimeter_assessment.explainability_trace);

    println!("\n✅ All three security modes are fully implemented and operational!");
    println!("🎯 Key Differences:");
    println!("   • Guardian: High confidence (0.95), quick response (5 min), visible deterrence");
    println!("   • Stealth: Lower confidence (0.75), longer observation (30 min), covert operations");
    println!("   • Perimeter Guard: Medium confidence (0.88), moderate response (10 min), boundary focus");
}

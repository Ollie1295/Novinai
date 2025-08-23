use insane_ai_security::core::*;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

fn main() {
    println!("🛡️ Security Mode Event Output Analysis");
    println!("======================================\n");

    let scenarios = vec![
        ("Known family member enters front door at 2 PM", create_family_context()),
        ("Unknown person walks slowly past driveway at 1 PM", create_unknown_walker_context()),
        ("Delivery driver leaves package at 3 PM, rings bell", create_delivery_context()),
        ("Pet triggers motion inside at midnight", create_pet_context()),
        ("Glassbreak detected in kitchen at 11 PM", create_glassbreak_context()),
        ("Smoke detected in living room at 10 AM", create_smoke_context()),
        ("Unknown car stops outside gate for 4 minutes", create_car_loitering_context()),
        ("Family member enters while away mode is on", create_away_mode_context()),
        ("Suspicious person touches door handle at 2 AM", create_door_handle_context()),
        ("Multiple nuisance events within 10 minutes", create_nuisance_context()),
    ];

    for (i, (description, context)) in scenarios.iter().enumerate() {
        println!("EVENT {}: {}", i + 1, description);
        println!("{}", "=".repeat(60));
        
        // Guardian Mode
        let mut guardian_system = InsaneSecuritySystem::default();
        guardian_system.set_security_mode(SecurityMode::Guardian);
        let guardian_result = guardian_system.process_threat(context);
        
        println!("🔰 GUARDIAN MODE:");
        println!("  Threat Level: {:.3}", guardian_result.threat_level);
        println!("  Confidence: {:.3}", guardian_result.confidence_score);
        println!("  Response Time: {} minutes", guardian_result.temporal_horizon.num_minutes());
        println!("  Alert Level: {}", get_alert_level(guardian_result.threat_level));
        println!("  Countermeasures: {:?}", guardian_result.countermeasures);
        println!("  Explanation: {}", guardian_result.explainability_trace);
        
        // Stealth Mode
        let mut stealth_system = InsaneSecuritySystem::default();
        stealth_system.set_security_mode(SecurityMode::Stealth);
        let stealth_result = stealth_system.process_threat(context);
        
        println!("\n🥷 STEALTH MODE:");
        println!("  Threat Level: {:.3}", stealth_result.threat_level);
        println!("  Confidence: {:.3}", stealth_result.confidence_score);
        println!("  Response Time: {} minutes", stealth_result.temporal_horizon.num_minutes());
        println!("  Alert Level: {}", get_alert_level(stealth_result.threat_level));
        println!("  Countermeasures: {:?}", stealth_result.countermeasures);
        println!("  Explanation: {}", stealth_result.explainability_trace);
        
        // Perimeter Guard Mode
        let mut perimeter_system = InsaneSecuritySystem::default();
        perimeter_system.set_security_mode(SecurityMode::PerimeterGuard);
        let perimeter_result = perimeter_system.process_threat(context);
        
        println!("\n🚧 PERIMETER GUARD MODE:");
        println!("  Threat Level: {:.3}", perimeter_result.threat_level);
        println!("  Confidence: {:.3}", perimeter_result.confidence_score);
        println!("  Response Time: {} minutes", perimeter_result.temporal_horizon.num_minutes());
        println!("  Alert Level: {}", get_alert_level(perimeter_result.threat_level));
        println!("  Countermeasures: {:?}", perimeter_result.countermeasures);
        println!("  Explanation: {}", perimeter_result.explainability_trace);
        
        println!("\n💡 MODE COMPARISON:");
        println!("  Guardian vs Stealth vs Perimeter: {:.3} vs {:.3} vs {:.3}", 
                 guardian_result.threat_level, stealth_result.threat_level, perimeter_result.threat_level);
        println!("  Response Times: {}min vs {}min vs {}min\n", 
                 guardian_result.temporal_horizon.num_minutes(),
                 stealth_result.temporal_horizon.num_minutes(),
                 perimeter_result.temporal_horizon.num_minutes());
        println!("{}\n", "─".repeat(80));
    }
}

fn get_alert_level(threat_level: f64) -> &'static str {
    match threat_level {
        t if t >= 0.9 => "CRITICAL",
        t if t >= 0.7 => "HIGH", 
        t if t >= 0.5 => "ELEVATED",
        t if t >= 0.3 => "STANDARD",
        _ => "LOW",
    }
}

fn create_family_context() -> ThreatContext {
    let mut indicators = HashMap::new();
    indicators.insert("known_entity".to_string(), 0.1);
    indicators.insert("authorized_access".to_string(), 0.05);
    
    ThreatContext {
        entity_id: Uuid::new_v4(),
        threat_indicators: indicators,
        environmental_factors: HashMap::new(),
        temporal_context: Utc::now(),
        confidence: 0.95,
    }
}

fn create_unknown_walker_context() -> ThreatContext {
    let mut indicators = HashMap::new();
    indicators.insert("unknown_entity".to_string(), 0.3);
    indicators.insert("surveillance_behavior".to_string(), 0.4);
    
    ThreatContext {
        entity_id: Uuid::new_v4(),
        threat_indicators: indicators,
        environmental_factors: HashMap::new(),
        temporal_context: Utc::now(),
        confidence: 0.7,
    }
}

fn create_delivery_context() -> ThreatContext {
    let mut indicators = HashMap::new();
    indicators.insert("legitimate_purpose".to_string(), 0.1);
    indicators.insert("expected_behavior".to_string(), 0.05);
    
    ThreatContext {
        entity_id: Uuid::new_v4(),
        threat_indicators: indicators,
        environmental_factors: HashMap::new(),
        temporal_context: Utc::now(),
        confidence: 0.9,
    }
}

fn create_pet_context() -> ThreatContext {
    let mut indicators = HashMap::new();
    indicators.insert("internal_motion".to_string(), 0.2);
    indicators.insert("pet_signature".to_string(), 0.05);
    
    ThreatContext {
        entity_id: Uuid::new_v4(),
        threat_indicators: indicators,
        environmental_factors: HashMap::new(),
        temporal_context: Utc::now(),
        confidence: 0.85,
    }
}

fn create_glassbreak_context() -> ThreatContext {
    let mut indicators = HashMap::new();
    indicators.insert("forced_entry".to_string(), 0.8);
    indicators.insert("property_damage".to_string(), 0.7);
    indicators.insert("nighttime_breach".to_string(), 0.6);
    
    ThreatContext {
        entity_id: Uuid::new_v4(),
        threat_indicators: indicators,
        environmental_factors: HashMap::new(),
        temporal_context: Utc::now(),
        confidence: 0.95,
    }
}

fn create_smoke_context() -> ThreatContext {
    let mut indicators = HashMap::new();
    indicators.insert("fire_hazard".to_string(), 0.9);
    indicators.insert("life_safety".to_string(), 1.0);
    indicators.insert("emergency_response".to_string(), 0.95);
    
    ThreatContext {
        entity_id: Uuid::new_v4(),
        threat_indicators: indicators,
        environmental_factors: HashMap::new(),
        temporal_context: Utc::now(),
        confidence: 0.98,
    }
}

fn create_car_loitering_context() -> ThreatContext {
    let mut indicators = HashMap::new();
    indicators.insert("surveillance_activity".to_string(), 0.6);
    indicators.insert("loitering_behavior".to_string(), 0.5);
    indicators.insert("perimeter_proximity".to_string(), 0.4);
    
    ThreatContext {
        entity_id: Uuid::new_v4(),
        threat_indicators: indicators,
        environmental_factors: HashMap::new(),
        temporal_context: Utc::now(),
        confidence: 0.8,
    }
}

fn create_away_mode_context() -> ThreatContext {
    let mut indicators = HashMap::new();
    indicators.insert("unexpected_entry".to_string(), 0.4);
    indicators.insert("known_entity".to_string(), 0.1);
    indicators.insert("system_armed".to_string(), 0.3);
    
    ThreatContext {
        entity_id: Uuid::new_v4(),
        threat_indicators: indicators,
        environmental_factors: HashMap::new(),
        temporal_context: Utc::now(),
        confidence: 0.75,
    }
}

fn create_door_handle_context() -> ThreatContext {
    let mut indicators = HashMap::new();
    indicators.insert("unauthorized_contact".to_string(), 0.8);
    indicators.insert("entry_attempt".to_string(), 0.7);
    indicators.insert("perimeter_breach".to_string(), 0.9);
    
    ThreatContext {
        entity_id: Uuid::new_v4(),
        threat_indicators: indicators,
        environmental_factors: HashMap::new(),
        temporal_context: Utc::now(),
        confidence: 0.9,
    }
}

fn create_nuisance_context() -> ThreatContext {
    let mut indicators = HashMap::new();
    indicators.insert("false_positive_pattern".to_string(), 0.2);
    indicators.insert("environmental_noise".to_string(), 0.3);
    indicators.insert("multiple_triggers".to_string(), 0.4);
    
    ThreatContext {
        entity_id: Uuid::new_v4(),
        threat_indicators: indicators,
        environmental_factors: HashMap::new(),
        temporal_context: Utc::now(),
        confidence: 0.6,
    }
}

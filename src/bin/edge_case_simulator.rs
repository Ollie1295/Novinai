use insane_ai_security::core::*;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc, TimeZone};
use serde_json::json;

#[derive(Debug)]
struct Scenario {
    id: u8,
    description: String,
    threat_indicators: HashMap<String, f64>,
    environmental_factors: HashMap<String, f64>,
    time_context: String,
    expected_threat_level: f64,
}

fn create_scenarios() -> Vec<Scenario> {
    vec![
        Scenario {
            id: 1,
            description: "Known family member enters front door at 2 PM".to_string(),
            threat_indicators: {
                let mut map = HashMap::new();
                map.insert("known_entity".to_string(), 0.1);
                map.insert("authorized_access".to_string(), 0.05);
                map.insert("normal_entry_point".to_string(), 0.0);
                map.insert("daylight_activity".to_string(), 0.0);
                map
            },
            environmental_factors: {
                let mut map = HashMap::new();
                map.insert("visibility".to_string(), 1.0);
                map.insert("normal_hours".to_string(), 1.0);
                map
            },
            time_context: "afternoon".to_string(),
            expected_threat_level: 0.05,
        },
        Scenario {
            id: 2,
            description: "Unknown person walks slowly past the driveway at 1 PM, never touching property".to_string(),
            threat_indicators: {
                let mut map = HashMap::new();
                map.insert("unknown_entity".to_string(), 0.3);
                map.insert("surveillance_behavior".to_string(), 0.4);
                map.insert("no_trespassing".to_string(), 0.0);
                map.insert("public_area".to_string(), 0.0);
                map
            },
            environmental_factors: {
                let mut map = HashMap::new();
                map.insert("visibility".to_string(), 1.0);
                map.insert("public_space".to_string(), 0.8);
                map
            },
            time_context: "afternoon".to_string(),
            expected_threat_level: 0.18,
        },
        Scenario {
            id: 3,
            description: "Delivery driver leaves a package at 3 PM, rings bell".to_string(),
            threat_indicators: {
                let mut map = HashMap::new();
                map.insert("legitimate_purpose".to_string(), 0.1);
                map.insert("expected_behavior".to_string(), 0.05);
                map.insert("package_delivery".to_string(), 0.0);
                map.insert("doorbell_use".to_string(), 0.0);
                map
            },
            environmental_factors: {
                let mut map = HashMap::new();
                map.insert("delivery_vehicle".to_string(), 0.9);
                map.insert("uniform_visible".to_string(), 0.9);
                map
            },
            time_context: "afternoon".to_string(),
            expected_threat_level: 0.04,
        },
        Scenario {
            id: 4,
            description: "Pet triggers motion inside at midnight".to_string(),
            threat_indicators: {
                let mut map = HashMap::new();
                map.insert("internal_motion".to_string(), 0.2);
                map.insert("pet_signature".to_string(), 0.05);
                map.insert("known_occupant".to_string(), 0.0);
                map.insert("nighttime_activity".to_string(), 0.1);
                map
            },
            environmental_factors: {
                let mut map = HashMap::new();
                map.insert("darkness".to_string(), 0.3);
                map.insert("internal_space".to_string(), 0.1);
                map
            },
            time_context: "night".to_string(),
            expected_threat_level: 0.09,
        },
        Scenario {
            id: 5,
            description: "Glassbreak detected in kitchen at 11 PM".to_string(),
            threat_indicators: {
                let mut map = HashMap::new();
                map.insert("forced_entry".to_string(), 0.8);
                map.insert("property_damage".to_string(), 0.7);
                map.insert("nighttime_breach".to_string(), 0.6);
                map.insert("critical_alert".to_string(), 0.9);
                map
            },
            environmental_factors: {
                let mut map = HashMap::new();
                map.insert("darkness".to_string(), 0.8);
                map.insert("vulnerable_entry".to_string(), 0.9);
                map
            },
            time_context: "night".to_string(),
            expected_threat_level: 0.77,
        },
        Scenario {
            id: 6,
            description: "Smoke detected in living room at 10 AM while family home".to_string(),
            threat_indicators: {
                let mut map = HashMap::new();
                map.insert("fire_hazard".to_string(), 0.9);
                map.insert("life_safety".to_string(), 1.0);
                map.insert("emergency_response".to_string(), 0.95);
                map.insert("occupants_present".to_string(), 0.8);
                map
            },
            environmental_factors: {
                let mut map = HashMap::new();
                map.insert("daylight".to_string(), 0.9);
                map.insert("occupied_dwelling".to_string(), 1.0);
                map
            },
            time_context: "morning".to_string(),
            expected_threat_level: 0.91,
        },
        Scenario {
            id: 7,
            description: "Unknown car stops outside gate and idles for 4 minutes".to_string(),
            threat_indicators: {
                let mut map = HashMap::new();
                map.insert("surveillance_activity".to_string(), 0.6);
                map.insert("loitering_behavior".to_string(), 0.5);
                map.insert("perimeter_proximity".to_string(), 0.4);
                map.insert("extended_presence".to_string(), 0.7);
                map
            },
            environmental_factors: {
                let mut map = HashMap::new();
                map.insert("vehicle_identification".to_string(), 0.3);
                map.insert("public_road".to_string(), 0.6);
                map
            },
            time_context: "day".to_string(),
            expected_threat_level: 0.55,
        },
        Scenario {
            id: 8,
            description: "Family member enters while away mode is on".to_string(),
            threat_indicators: {
                let mut map = HashMap::new();
                map.insert("unexpected_entry".to_string(), 0.4);
                map.insert("known_entity".to_string(), 0.1);
                map.insert("system_armed".to_string(), 0.3);
                map.insert("authorized_access".to_string(), 0.2);
                map
            },
            environmental_factors: {
                let mut map = HashMap::new();
                map.insert("away_mode_active".to_string(), 0.7);
                map.insert("legitimate_occupant".to_string(), 0.9);
                map
            },
            time_context: "variable".to_string(),
            expected_threat_level: 0.25,
        },
        Scenario {
            id: 9,
            description: "Suspicious person touches door handle at 2 AM".to_string(),
            threat_indicators: {
                let mut map = HashMap::new();
                map.insert("unauthorized_contact".to_string(), 0.8);
                map.insert("entry_attempt".to_string(), 0.7);
                map.insert("nighttime_activity".to_string(), 0.6);
                map.insert("perimeter_breach".to_string(), 0.9);
                map
            },
            environmental_factors: {
                let mut map = HashMap::new();
                map.insert("darkness".to_string(), 0.9);
                map.insert("vulnerable_hours".to_string(), 0.8);
                map
            },
            time_context: "night".to_string(),
            expected_threat_level: 0.75,
        },
        Scenario {
            id: 10,
            description: "Multiple nuisance events (wind, leaves, neighbour's cat) within 10 minutes".to_string(),
            threat_indicators: {
                let mut map = HashMap::new();
                map.insert("false_positive_pattern".to_string(), 0.2);
                map.insert("environmental_noise".to_string(), 0.3);
                map.insert("multiple_triggers".to_string(), 0.4);
                map.insert("natural_causes".to_string(), 0.1);
                map
            },
            environmental_factors: {
                let mut map = HashMap::new();
                map.insert("weather_conditions".to_string(), 0.7);
                map.insert("animal_activity".to_string(), 0.6);
                map
            },
            time_context: "variable".to_string(),
            expected_threat_level: 0.25,
        },
    ]
}

fn simulate_security_response(scenario: &Scenario, mode: SecurityMode) -> serde_json::Value {
    let entity_id = Uuid::new_v4();
    
    let context = ThreatContext {
        entity_id,
        threat_indicators: scenario.threat_indicators.clone(),
        environmental_factors: scenario.environmental_factors.clone(),
        temporal_context: Utc::now(),
        confidence: 0.85,
    };

    let mut system = InsaneSecuritySystem::default();
    system.set_security_mode(mode);
    
    let assessment = system.process_threat(&context);
    
    json!({
        "mode": format!("{:?}", mode),
        "threat_level": format!("{:.3}", assessment.threat_level),
        "confidence": format!("{:.3}", assessment.confidence_score),
        "response_time_minutes": assessment.temporal_horizon.num_minutes(),
        "countermeasures": assessment.countermeasures,
        "explanation": assessment.explainability_trace,
        "alert_level": determine_alert_level(assessment.threat_level)
    })
}

fn determine_alert_level(threat_level: f64) -> String {
    match threat_level {
        t if t >= 0.9 => "CRITICAL".to_string(),
        t if t >= 0.7 => "HIGH".to_string(),
        t if t >= 0.5 => "ELEVATED".to_string(),
        t if t >= 0.3 => "STANDARD".to_string(),
        _ => "LOW".to_string(),
    }
}

fn explain_mode_differences(guardian: &serde_json::Value, stealth: &serde_json::Value, perimeter: &serde_json::Value) -> String {
    let g_threat: f64 = guardian["threat_level"].as_str().unwrap().parse().unwrap();
    let s_threat: f64 = stealth["threat_level"].as_str().unwrap().parse().unwrap();
    let p_threat: f64 = perimeter["threat_level"].as_str().unwrap().parse().unwrap();
    
    let g_time: i64 = guardian["response_time_minutes"].as_i64().unwrap();
    let s_time: i64 = stealth["response_time_minutes"].as_i64().unwrap();
    let p_time: i64 = perimeter["response_time_minutes"].as_i64().unwrap();

    format!(
        "Guardian shows {} threat assessment with {}-min response (aggressive protection). Stealth shows {} threat with {}-min response (covert monitoring). Perimeter shows {} threat with {}-min response (boundary focus).",
        if g_threat > s_threat && g_threat > p_threat { "highest" } 
        else if g_threat < s_threat && g_threat < p_threat { "lowest" } 
        else { "moderate" },
        g_time,
        if s_threat > g_threat && s_threat > p_threat { "highest" } 
        else if s_threat < g_threat && s_threat < p_threat { "lowest" } 
        else { "moderate" },
        s_time,
        if p_threat > g_threat && p_threat > s_threat { "highest" } 
        else if p_threat < g_threat && p_threat < s_threat { "lowest" } 
        else { "moderate" },
        p_time
    )
}

fn main() {
    println!("🛡️ Edge Case Security Mode Simulation");
    println!("=====================================\n");

    let scenarios = create_scenarios();
    let mut results = Vec::new();

    for scenario in &scenarios {
        println!("Testing Scenario {}: {}", scenario.id, scenario.description);
        
        let guardian_response = simulate_security_response(scenario, SecurityMode::Guardian);
        let stealth_response = simulate_security_response(scenario, SecurityMode::Stealth);
        let perimeter_response = simulate_security_response(scenario, SecurityMode::PerimeterGuard);
        
        let explanation = explain_mode_differences(&guardian_response, &stealth_response, &perimeter_response);
        
        let result = json!({
            "scenario": scenario.description,
            "guardian_response": guardian_response,
            "stealth_response": stealth_response,
            "perimeter_response": perimeter_response,
            "explanation": explanation
        });
        
        results.push(result);
    }

    let final_output = json!({
        "simulation_metadata": {
            "timestamp": Utc::now().to_rfc3339(),
            "total_scenarios": scenarios.len(),
            "security_modes_tested": ["Guardian", "Stealth", "PerimeterGuard"]
        },
        "scenarios": results
    });

    println!("\n📊 SIMULATION RESULTS (JSON):");
    println!("{}", serde_json::to_string_pretty(&final_output).unwrap());
}

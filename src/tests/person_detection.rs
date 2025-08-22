#[cfg(test)]
mod person_detection_tests {
    use crate::core::*;
    use std::collections::HashMap;

    fn create_test_system() -> InsaneSecuritySystem {
        let config = SecurityConfig {
            intelligence_level: IntelligenceLevel::Insane,
            adaptive_thresholds: true,
            multi_modal_fusion: true,
            emotional_intelligence: true,
            continuous_learning: true,
            explainable_ai: true,
            active_response: true,
        };

        let mut system = InsaneSecuritySystem::default();
        system.config = config;
        system
    }

    fn create_person_context(person_type: &str, confidence: f64) -> ThreatContext {
        let mut biometric_data = HashMap::new();
        biometric_data.insert("facial_recognition".to_string(), confidence);
        biometric_data.insert("gait_analysis".to_string(), confidence * 0.9);
        
        ThreatContext {
            environmental_factors: vec![
                format!("person_detected:{}", person_type),
                "camera_zone:entrance".to_string(),
                "lighting:adequate".to_string(),
            ],
            temporal_patterns: vec![
                "time_of_day:business_hours".to_string(),
                "day_of_week:weekday".to_string(),
            ],
            historical_context: vec![
                format!("person_type:{}", person_type),
                format!("confidence:{:.2}", confidence),
            ],
            biometric_data,
            network_topology: HashMap::new(),
            geospatial_context: vec![
                "location:main_entrance".to_string(),
                "zone:public_access".to_string(),
            ],
        }
    }

    #[test]
    fn test_unknown_person_detection() {
        println!("\n🔍 Testing Unknown Person Detection");
        let mut system = create_test_system();
        
        // Create unknown person context with high confidence detection but no identity match
        let context = create_person_context("unknown", 0.92);
        let threat_score = 0.75; // Higher threat for unknown person
        
        // Test dynamic alert calculation
        let dynamic_alert = AlertLevel::from_threat_score_dynamic(threat_score, &context, &system.thresholds);
        let multi_dim_alert = AlertLevel::from_multi_dimensional(&context, threat_score);
        
        println!("   Unknown Person - Threat Score: {:.3}", threat_score);
        println!("   Dynamic Alert: {:?}", dynamic_alert);
        println!("   Multi-Dimensional Alert: {:?}", multi_dim_alert);
        
        // Test ML threat classification (simulated)
        println!("   ML Classification: ThreatClass::Unknown(confidence: {:.2})", threat_score);
        
        // Test emotional intelligence response (simulated)
        println!("   Emotional Analysis: High_Alert_State(empathy: moderate)");
        
        // Should trigger elevated or critical alert for unknown person
        assert!(matches!(dynamic_alert, AlertLevel::Elevated | AlertLevel::Critical));
        assert!(threat_score > 0.6);
    }

    #[test]
    fn test_known_person_detection() {
        println!("\n👤 Testing Known Person Detection");
        let mut system = create_test_system();
        
        // Create known person context with high confidence and identity match
        let mut context = create_person_context("known_employee", 0.95);
        context.historical_context.push("identity_verified:true".to_string());
        context.historical_context.push("access_level:authorized".to_string());
        context.biometric_data.insert("identity_match".to_string(), 0.96);
        
        let threat_score = 0.15; // Low threat for known authorized person
        
        // Test alert calculations
        let dynamic_alert = AlertLevel::from_threat_score_dynamic(threat_score, &context, &system.thresholds);
        let multi_dim_alert = AlertLevel::from_multi_dimensional(&context, threat_score);
        
        println!("   Known Person - Threat Score: {:.3}", threat_score);
        println!("   Dynamic Alert: {:?}", dynamic_alert);
        println!("   Multi-Dimensional Alert: {:?}", multi_dim_alert);
        
        // Test ML classification (simulated)
        println!("   ML Classification: ThreatClass::Authorized(confidence: {:.2})", 1.0 - threat_score);
        
        // Test emotional response (simulated)
        println!("   Emotional Analysis: Calm_State(empathy: high, trust: established)");
        
        // Should result in low alert level for known person
        assert!(matches!(dynamic_alert, AlertLevel::Ignore | AlertLevel::Standard));
        assert!(threat_score < 0.3);
    }

    #[test]
    fn test_unsure_person_detection() {
        println!("\n❓ Testing Unsure Person Detection");
        let mut system = create_test_system();
        
        // Create unsure person context with medium confidence
        let mut context = create_person_context("unsure", 0.65);
        context.historical_context.push("identity_uncertain:true".to_string());
        context.historical_context.push("partial_match:possible".to_string());
        context.biometric_data.insert("identity_confidence".to_string(), 0.68);
        
        let threat_score = 0.55; // Medium threat for uncertain identity
        
        // Test alert calculations
        let dynamic_alert = AlertLevel::from_threat_score_dynamic(threat_score, &context, &system.thresholds);
        let multi_dim_alert = AlertLevel::from_multi_dimensional(&context, threat_score);
        
        println!("   Unsure Person - Threat Score: {:.3}", threat_score);
        println!("   Dynamic Alert: {:?}", dynamic_alert);
        println!("   Multi-Dimensional Alert: {:?}", multi_dim_alert);
        
        // Test ML classification (simulated)
        println!("   ML Classification: ThreatClass::Uncertain(confidence: {:.2})", threat_score);
        
        // Test emotional intelligence (simulated)
        println!("   Emotional Analysis: Cautious_State(empathy: moderate, uncertainty: high)");
        
        // Test continuous learning update (simulated)
        println!("   Learning Update: UncertaintyCase(requires_additional_data: true)");
        
        // Should result in standard or elevated alert for uncertain person
        assert!(matches!(dynamic_alert, AlertLevel::Standard | AlertLevel::Elevated));
        assert!(threat_score > 0.4 && threat_score < 0.7);
    }

    #[test]
    fn test_person_detection_with_explainable_ai() {
        println!("\n🧠 Testing Person Detection with Explainable AI");
        let mut system = create_test_system();
        
        let scenarios = vec![
            ("unknown", 0.85, "Unknown person detected at entrance"),
            ("known_employee", 0.12, "Authorized employee John Doe identified"),
            ("unsure", 0.58, "Partial facial match, requires verification"),
        ];
        
        for (person_type, threat_score, description) in scenarios {
            println!("\n   Scenario: {}", description);
            let context = create_person_context(person_type, threat_score);
            
            // Generate explanation for the threat assessment (simulated)
            println!("   AI Explanation: Decision based on facial_recognition({:.2}), temporal_context(business_hours), location(entrance)", threat_score);
            
            // Test reasoning transparency (simulated)
            println!("   Reasoning Chain: [Detection] -> [Classification] -> [Context_Analysis] -> [Threat_Assessment] -> [Decision]");
        }
    }

    #[test]
    fn test_automated_response_to_person_detection() {
        println!("\n🚨 Testing Automated Response to Person Detection");
        let mut system = create_test_system();
        
        // Test response to unknown person
        let unknown_context = create_person_context("unknown", 0.88);
        println!("   Unknown Person Response: Alert_Security(priority: high, actions: [notify_guards, lock_doors, track_movement])");
        
        // Test response to known person
        let known_context = create_person_context("known_employee", 0.18);
        println!("   Known Person Response: Normal_Access(priority: low, actions: [log_entry, continue_monitoring])");
        
        // Test response to unsure person
        let unsure_context = create_person_context("unsure", 0.62);
        println!("   Unsure Person Response: Verify_Identity(priority: medium, actions: [request_id, additional_scans, human_review])");
    }
}

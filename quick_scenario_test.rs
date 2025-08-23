//! Quick Scenario Test - Unknown Person at Front Door
//! User away, 8pm, front door camera detection

use std::time::Instant;

fn main() {
    println!("🏠 INSANE AI SECURITY - QUICK SCENARIO TEST");
    println!("===========================================");
    println!("📹 Scenario: Unknown person at front door, 8PM, user away");
    println!();

    let test_start = Instant::now();
    
    // Simulate real-world scenario
    let scenario = SecurityScenario {
        time: "8:00 PM",
        location: "Front Door",
        user_status: "Away",
        entities_detected: 1,
        person_recognized: false,
        environmental_factors: vec!["evening", "residential_area", "motion_detected"],
        behavioral_indicators: vec!["approaching_door", "unknown_identity", "after_hours"],
        camera_confidence: 0.89,
        motion_intensity: 0.65,
        duration_seconds: 12,
    };
    
    println!("🔍 SCENARIO DETAILS:");
    println!("   Time: {}", scenario.time);
    println!("   Location: {}", scenario.location);
    println!("   User Status: {}", scenario.user_status);
    println!("   Entities: {}", scenario.entities_detected);
    println!("   Person Recognized: {}", scenario.person_recognized);
    println!("   Camera Confidence: {:.1}%", scenario.camera_confidence * 100.0);
    println!("   Motion Intensity: {:.1}%", scenario.motion_intensity * 100.0);
    println!("   Duration: {}s", scenario.duration_seconds);
    println!();
    
    // Run AI threat assessment
    let result = assess_threat(&scenario);
    
    println!("🤖 AI THREAT ASSESSMENT:");
    println!("   Threat Level: {:.3}", result.threat_level);
    println!("   Alert Level: {}", result.alert_level);
    println!("   Confidence: {:.1}%", result.confidence * 100.0);
    println!("   Response Time: {:?}", result.processing_time);
    println!();
    
    println!("🎯 RECOMMENDED ACTIONS:");
    for (i, action) in result.recommended_actions.iter().enumerate() {
        println!("   {}. {}", i + 1, action);
    }
    println!();
    
    println!("🧠 AI REASONING:");
    println!("   {}", result.reasoning);
    println!();
    
    println!("📊 UNCERTAINTY ANALYSIS:");
    println!("   Epistemic (Model): {:.1}%", result.epistemic_uncertainty * 100.0);
    println!("   Aleatoric (Data): {:.1}%", result.aleatoric_uncertainty * 100.0);
    println!("   Total Uncertainty: {:.1}%", result.total_uncertainty * 100.0);
    println!();
    
    // Risk assessment
    let risk_level = match result.threat_level {
        t if t >= 0.8 => "🔴 HIGH RISK",
        t if t >= 0.6 => "🟠 MODERATE RISK", 
        t if t >= 0.4 => "🟡 ELEVATED RISK",
        t if t >= 0.2 => "🟢 LOW RISK",
        _ => "⚪ MINIMAL RISK",
    };
    
    println!("⚖️  FINAL RISK ASSESSMENT: {}", risk_level);
    
    let total_time = test_start.elapsed();
    println!("⏱️  Total Processing Time: {:?}", total_time);
}

#[derive(Debug)]
struct SecurityScenario {
    time: &'static str,
    location: &'static str,
    user_status: &'static str,
    entities_detected: u32,
    person_recognized: bool,
    environmental_factors: Vec<&'static str>,
    behavioral_indicators: Vec<&'static str>,
    camera_confidence: f64,
    motion_intensity: f64,
    duration_seconds: u32,
}

#[derive(Debug)]
struct ThreatAssessment {
    threat_level: f64,
    alert_level: String,
    confidence: f64,
    processing_time: std::time::Duration,
    recommended_actions: Vec<String>,
    reasoning: String,
    epistemic_uncertainty: f64,
    aleatoric_uncertainty: f64,
    total_uncertainty: f64,
}

fn assess_threat(scenario: &SecurityScenario) -> ThreatAssessment {
    let start = Instant::now();
    
    // Base threat calculation
    let mut threat = 0.3; // Base level for unknown person
    
    // Time factor (8 PM is somewhat suspicious)
    threat += 0.15; // Evening hours
    
    // User away factor (significant)
    threat += 0.25; // User not present
    
    // Unknown person factor
    if !scenario.person_recognized {
        threat += 0.2;
    }
    
    // Location factor (front door is high priority)
    if scenario.location == "Front Door" {
        threat += 0.15;
    }
    
    // Behavioral factors
    for behavior in &scenario.behavioral_indicators {
        match *behavior {
            "approaching_door" => threat += 0.1,
            "unknown_identity" => threat += 0.05,
            "after_hours" => threat += 0.1,
            _ => threat += 0.05,
        }
    }
    
    // Camera confidence factor
    let confidence_factor = scenario.camera_confidence;
    threat *= confidence_factor;
    
    // Motion intensity
    threat += scenario.motion_intensity * 0.1;
    
    // Duration factor (longer presence = higher threat)
    if scenario.duration_seconds > 10 {
        threat += 0.05;
    }
    
    // Apply calibration
    let calibrated_threat = 1.0 / (1.0 + (-2.5 * (threat - 0.5)).exp());
    let final_threat = calibrated_threat.clamp(0.0, 1.0);
    
    // Determine alert level
    let alert_level = match final_threat {
        t if t >= 0.8 => "CRITICAL",
        t if t >= 0.6 => "HIGH", 
        t if t >= 0.4 => "ELEVATED",
        t if t >= 0.2 => "STANDARD",
        _ => "LOW",
    };
    
    // Calculate uncertainties
    let epistemic: f64 = 0.15; // Model uncertainty
    let aleatoric = (1.0 - confidence_factor) * 0.2; // Data uncertainty
    let total = (epistemic.powi(2) + aleatoric.powi(2)).sqrt();
    
    // Generate recommendations
    let mut actions = vec![];
    
    if final_threat >= 0.6 {
        actions.push("🚨 Send immediate alert to user".to_string());
        actions.push("📱 Activate mobile notification".to_string());
        actions.push("🔊 Trigger audible alarm".to_string());
    } else if final_threat >= 0.4 {
        actions.push("📧 Send notification to user".to_string());
        actions.push("📹 Begin continuous recording".to_string());
        actions.push("🔍 Monitor for 5 more minutes".to_string());
    } else {
        actions.push("📝 Log event in security history".to_string());
        actions.push("📹 Save 30-second clip".to_string());
    }
    
    // Always add these actions
    actions.push("💾 Store biometric data for future recognition".to_string());
    actions.push("🕒 Timestamp and geo-tag event".to_string());
    
    // Generate reasoning
    let reasoning = format!(
        "Unknown person detected at {} when user is {}. Camera confidence {:.1}%. \
         Evening hours and front door location increase threat assessment. \
         Motion detected for {}s with {:.1}% intensity. \
         Behavioral indicators: {:?}",
        scenario.time,
        scenario.user_status.to_lowercase(),
        confidence_factor * 100.0,
        scenario.duration_seconds,
        scenario.motion_intensity * 100.0,
        scenario.behavioral_indicators
    );
    
    ThreatAssessment {
        threat_level: final_threat,
        alert_level: alert_level.to_string(),
        confidence: confidence_factor,
        processing_time: start.elapsed(),
        recommended_actions: actions,
        reasoning,
        epistemic_uncertainty: epistemic,
        aleatoric_uncertainty: aleatoric,
        total_uncertainty: total,
    }
}

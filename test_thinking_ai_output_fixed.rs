use std::collections::HashMap;

#[derive(Clone)]
struct Event {
    timestamp: f64,
    camera: &'static str,
    person_id: &'static str,
    motion_type: &'static str,
    face_confidence: f64,
    rang_doorbell: bool,
    knocked: bool,
    dwell_seconds: f64,
}

#[derive(Default)]
struct IncidentGraph {
    person_id: String,
    cameras_seen: Vec<String>,
    start_time: f64,
    events: Vec<Event>,
    total_dwell: f64,
    rang_doorbell: bool,
    knocked: bool,
    identity_confirmed: bool,
    threat_level: String,
    suppressed_count: u32,
}

fn main() {
    println!("🧠 THINKING AI OUTPUT");
    println!("=====================");
    println!("Scenario: Unknown person, 2 events in 30s, front door cam, user home\n");

    // Simulate the two events
    let events = vec![
        Event {
            timestamp: 0.0,
            camera: "FrontDoorCam",
            person_id: "unknown_person_47",
            motion_type: "approach",
            face_confidence: 0.23, // Low confidence (partial face/angle)
            rang_doorbell: false,
            knocked: false,
            dwell_seconds: 12.0,
        },
        Event {
            timestamp: 28.5,
            camera: "FrontDoorCam", 
            person_id: "unknown_person_47", // Same person ID
            motion_type: "lingering",
            face_confidence: 0.31, // Still low confidence
            rang_doorbell: false,
            knocked: false,
            dwell_seconds: 18.0,
        },
    ];

    // Process through thinking AI pipeline
    let mut incident = build_incident_graph(&events);
    let threat_assessment = assess_threat(&incident);
    incident.threat_level = threat_assessment.clone();
    
    // Generate natural language output
    let summary = generate_summary(&incident, &threat_assessment);
    let counterfactuals = generate_counterfactuals(&incident);
    
    println!("📱 USER NOTIFICATION:");
    println!("=====================");
    println!("{}\n", summary);
    
    println!("📊 DETAILED ANALYSIS:");
    println!("=====================");
    print_threat_reasoning(&incident, &threat_assessment);
    
    println!("\n🔍 COUNTERFACTUALS:");
    println!("==================");
    println!("{}\n", counterfactuals);
    
    println!("📈 INCIDENT TIMELINE:");
    println!("=====================");
    print_incident_timeline(&incident);
}

fn build_incident_graph(events: &[Event]) -> IncidentGraph {
    let mut incident = IncidentGraph::default();
    
    if let Some(first) = events.first() {
        incident.person_id = first.person_id.to_string();
        incident.start_time = first.timestamp;
    }
    
    for event in events {
        incident.events.push(event.clone());
        incident.total_dwell += event.dwell_seconds;
        incident.rang_doorbell = incident.rang_doorbell || event.rang_doorbell;
        incident.knocked = incident.knocked || event.knocked;
        
        if !incident.cameras_seen.contains(&event.camera.to_string()) {
            incident.cameras_seen.push(event.camera.to_string());
        }
    }
    
    incident.suppressed_count = (events.len() - 1) as u32; // First event alerts, rest suppressed
    incident
}

fn assess_threat(incident: &IncidentGraph) -> String {
    let mut threat_score = 0.0;
    
    // Base threat for unknown person at front door while user home
    threat_score += 0.2; // Moderate base
    
    // Time factors (assuming daytime/evening - not suspicious)
    threat_score += 0.0;
    
    // Behavior factors
    if !incident.rang_doorbell && !incident.knocked {
        threat_score += 0.4; // Suspicious - no normal visitor protocol
    }
    
    if incident.total_dwell > 25.0 {
        threat_score += 0.3; // Long lingering without ringing
    }
    
    // Front door is actually GOOD for legitimate visitors
    threat_score -= 0.1;
    
    // User home factor - unexpected visitor
    threat_score += 0.2;
    
    if threat_score >= 0.6 {
        "ELEVATED".to_string()
    } else if threat_score >= 0.3 {
        "STANDARD".to_string() 
    } else {
        "LOW".to_string()
    }
}

fn generate_summary(incident: &IncidentGraph, threat: &str) -> String {
    let duration = if let (Some(first), Some(last)) = (incident.events.first(), incident.events.last()) {
        last.timestamp - first.timestamp + last.dwell_seconds
    } else {
        0.0
    };
    
    let behavior = if incident.rang_doorbell {
        "rang doorbell"
    } else if incident.knocked {
        "knocked"  
    } else {
        "no doorbell/knock"
    };
    
    format!(
        "🔔 Front Door Activity - {} Alert\n\
        Unknown visitor lingered {:.0}s over {:.0}s window, {}. \
        Pattern suggests unfamiliar person hesitating at entrance. \
        {} duplicate motions suppressed into this summary.",
        threat.to_uppercase(),
        incident.total_dwell,
        duration,
        behavior,
        incident.suppressed_count
    )
}

fn generate_counterfactuals(incident: &IncidentGraph) -> String {
    let mut factors = Vec::new();
    
    if !incident.rang_doorbell && !incident.knocked {
        factors.push("rang doorbell or knocked");
    }
    
    if incident.total_dwell > 20.0 {
        factors.push("stayed <20 seconds");
    }
    
    factors.push("was recognized as family/expected guest");
    factors.push("delivery token was active");
    
    format!(
        "Would have resulted in LOW/IGNORE alert if visitor had: {}",
        factors.join(", ")
    )
}

fn print_threat_reasoning(incident: &IncidentGraph, threat: &str) {
    println!("Person: {} (unrecognized)", incident.person_id);
    println!("Location: {} (normal entry point ✓)", incident.cameras_seen.join(", "));
    println!("Duration: {:.0}s total dwell", incident.total_dwell);
    println!("Behavior: {} (visitor protocol {})", 
        if incident.rang_doorbell || incident.knocked { "announced" } else { "silent" },
        if incident.rang_doorbell || incident.knocked { "✓" } else { "⚠️" });
    println!("Context: User home, no expected window active");
    println!("Threat Level: {} (lingering + no announcement)", threat);
}

fn print_incident_timeline(incident: &IncidentGraph) {
    for (i, event) in incident.events.iter().enumerate() {
        let action = if i == 0 { "ALERT SENT" } else { "SUPPRESSED" };
        println!("T+{:>4.1}s: {} motion, {:.0}s dwell - {}", 
            event.timestamp, event.motion_type, event.dwell_seconds, action);
    }
    println!("Summary: 1 alert sent, {} events suppressed", incident.suppressed_count);
}

//! Demonstration of "First Awareness, Then Suppression" workflow
//! Shows how the system provides initial awareness but suppresses redundant notifications

use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

// Simplified types for demonstration
#[derive(Debug, Clone)]
pub struct SecurityEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub location: String,
    pub confidence: f64,
    pub alert_level: AlertLevel,
}

#[derive(Debug, Clone)]
pub enum EventType {
    VehicleApproach,
    PersonDetected,
    DoorApproach,
    PackageDelivery,
}

#[derive(Debug, Clone)]
pub enum AlertLevel {
    Ignore,
    Standard,
    Elevated,
    Critical,
}

#[derive(Debug, Clone)]
pub enum EventClassification {
    DeliverySequence,
    KnownPersonMovement,
    SuspiciousActivity,
}

#[derive(Debug, Clone)]
pub struct CorrelatedEvent {
    pub primary_event_id: String,
    pub event_chain: Vec<String>,
    pub event_type_sequence: Vec<EventType>,
    pub start_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub confidence_evolution: Vec<f64>,
    pub classification: EventClassification,
    pub suppression_count: u32,
}

#[derive(Debug, Clone)]
pub enum NotificationDecision {
    Notify {
        message: String,
        priority: String,
    },
    Suppress {
        reason: String,
        correlation_id: Option<String>,
    },
    Summary {
        message: String,
        event_count: u32,
        correlation_id: String,
    },
}

pub struct EventCorrelationEngine {
    pub active_events: HashMap<String, CorrelatedEvent>,
    pub correlation_window: Duration,
}

impl EventCorrelationEngine {
    pub fn new() -> Self {
        Self {
            active_events: HashMap::new(),
            correlation_window: Duration::minutes(10),
        }
    }

    pub fn correlate_event(&mut self, event: &SecurityEvent) -> Option<String> {
        // Look for existing correlated events
        if let Some(parent_id) = self.find_correlatable_event(event) {
            self.add_to_existing_correlation(&parent_id, event);
            Some(parent_id)
        } else {
            // Start new correlation chain
            if self.is_sequence_initiator(event) {
                self.start_new_correlation(event);
                Some(event.id.clone())
            } else {
                None
            }
        }
    }

    fn find_correlatable_event(&self, event: &SecurityEvent) -> Option<String> {
        for (id, corr_event) in &self.active_events {
            if self.fits_sequence_pattern(event, corr_event) {
                return Some(id.clone());
            }
        }
        None
    }

    fn fits_sequence_pattern(&self, event: &SecurityEvent, corr_event: &CorrelatedEvent) -> bool {
        use EventType::*;
        
        match (&corr_event.classification, &event.event_type) {
            (EventClassification::DeliverySequence, PersonDetected) => {
                matches!(corr_event.event_type_sequence.last(), Some(VehicleApproach))
            },
            (EventClassification::DeliverySequence, DoorApproach) => {
                matches!(corr_event.event_type_sequence.last(), Some(PersonDetected))
            },
            (EventClassification::DeliverySequence, PackageDelivery) => {
                matches!(corr_event.event_type_sequence.last(), Some(DoorApproach))
            },
            _ => false,
        }
    }

    fn is_sequence_initiator(&self, event: &SecurityEvent) -> bool {
        matches!(event.event_type, EventType::VehicleApproach) ||
        (matches!(event.event_type, EventType::PersonDetected) && event.confidence > 0.7)
    }

    fn start_new_correlation(&mut self, event: &SecurityEvent) {
        let classification = self.classify_initial_event(event);
        
        let corr_event = CorrelatedEvent {
            primary_event_id: event.id.clone(),
            event_chain: vec![event.id.clone()],
            event_type_sequence: vec![event.event_type.clone()],
            start_time: event.timestamp,
            last_update: event.timestamp,
            confidence_evolution: vec![event.confidence],
            classification,
            suppression_count: 0,
        };
        
        self.active_events.insert(event.id.clone(), corr_event);
    }

    fn add_to_existing_correlation(&mut self, parent_id: &str, event: &SecurityEvent) {
        if let Some(corr_event) = self.active_events.get_mut(parent_id) {
            corr_event.event_chain.push(event.id.clone());
            corr_event.event_type_sequence.push(event.event_type.clone());
            corr_event.last_update = event.timestamp;
            corr_event.confidence_evolution.push(event.confidence);
            corr_event.suppression_count += 1;
        }
    }

    fn classify_initial_event(&self, event: &SecurityEvent) -> EventClassification {
        match event.event_type {
            EventType::VehicleApproach => EventClassification::DeliverySequence,
            EventType::PersonDetected if event.confidence > 0.8 => EventClassification::KnownPersonMovement,
            _ => EventClassification::SuspiciousActivity,
        }
    }
}

pub struct NotificationStrategy {
    pub awareness_threshold: f64,
    pub suppression_enabled: bool,
    pub max_suppression_count: u32,
    pub summary_enabled: bool,
}

impl NotificationStrategy {
    pub fn new() -> Self {
        Self {
            awareness_threshold: 0.6,
            suppression_enabled: true,
            max_suppression_count: 5,
            summary_enabled: true,
        }
    }

    pub fn decide_notification(
        &self,
        event: &SecurityEvent,
        correlation_engine: &EventCorrelationEngine,
    ) -> NotificationDecision {
        // Check if this event is part of a correlated sequence
        if let Some(corr_event) = correlation_engine.active_events.get(&event.id) {
            return self.handle_correlated_event(event, corr_event);
        }
        
        // Check if this event correlates to an existing sequence
        for (parent_id, corr_event) in &correlation_engine.active_events {
            if corr_event.event_chain.contains(&event.id) {
                return self.handle_sequence_event(event, corr_event, parent_id);
            }
        }
        
        // Standalone event
        NotificationDecision::Notify {
            message: format!("{:?} Alert: {:?} at {} (Confidence: {:.0}%)", 
                           event.alert_level, event.event_type, event.location, event.confidence * 100.0),
            priority: "Medium".to_string(),
        }
    }

    fn handle_correlated_event(
        &self,
        event: &SecurityEvent,
        corr_event: &CorrelatedEvent,
    ) -> NotificationDecision {
        // First event in sequence - provide awareness
        if event.confidence >= self.awareness_threshold {
            NotificationDecision::Notify {
                message: self.format_awareness_message(event, &corr_event.classification),
                priority: "Low".to_string(),
            }
        } else {
            NotificationDecision::Suppress {
                reason: "Below awareness threshold".to_string(),
                correlation_id: Some(event.id.clone()),
            }
        }
    }

    fn handle_sequence_event(
        &self,
        event: &SecurityEvent,
        corr_event: &CorrelatedEvent,
        parent_id: &str,
    ) -> NotificationDecision {
        // Check if we should suppress based on sequence classification
        let should_suppress = match corr_event.classification {
            EventClassification::DeliverySequence => {
                self.suppression_enabled && corr_event.suppression_count < self.max_suppression_count
            },
            _ => false,
        };
        
        if should_suppress {
            // Check if this is the final event (for summary)
            if self.is_sequence_completion_event(event, corr_event) && self.summary_enabled {
                NotificationDecision::Summary {
                    message: self.format_summary_message(corr_event),
                    event_count: corr_event.event_chain.len() as u32,
                    correlation_id: parent_id.to_string(),
                }
            } else {
                NotificationDecision::Suppress {
                    reason: format!("Part of {:?} sequence", corr_event.classification),
                    correlation_id: Some(parent_id.to_string()),
                }
            }
        } else {
            // Don't suppress
            NotificationDecision::Notify {
                message: format!("{:?} at {} (Confidence: {:.0}%)", 
                               event.event_type, event.location, event.confidence * 100.0),
                priority: "Medium".to_string(),
            }
        }
    }

    fn is_sequence_completion_event(&self, event: &SecurityEvent, corr_event: &CorrelatedEvent) -> bool {
        match corr_event.classification {
            EventClassification::DeliverySequence => {
                matches!(event.event_type, EventType::PackageDelivery) ||
                (event.location.contains("street") && corr_event.event_chain.len() >= 3)
            },
            _ => false,
        }
    }

    fn format_awareness_message(&self, _event: &SecurityEvent, classification: &EventClassification) -> String {
        match classification {
            EventClassification::DeliverySequence => {
                "📦 Likely delivery activity detected. Monitoring...".to_string()
            },
            EventClassification::KnownPersonMovement => {
                "👤 Known person detected on property. Tracking movement...".to_string()
            },
            _ => {
                "🔍 Activity detected. Analyzing...".to_string()
            },
        }
    }

    fn format_summary_message(&self, corr_event: &CorrelatedEvent) -> String {
        let duration = Utc::now().signed_duration_since(corr_event.start_time).num_minutes();
        
        match corr_event.classification {
            EventClassification::DeliverySequence => {
                format!("✅ Delivery completed. Package delivered at front door. Duration: {}min", duration)
            },
            _ => {
                format!("📋 Activity sequence completed. {} events over {}min", 
                       corr_event.event_chain.len(), duration)
            },
        }
    }
}

fn main() {
    println!("🚀 First Awareness, Then Suppression - Delivery Scenario Demo\n");

    let mut correlation_engine = EventCorrelationEngine::new();
    let notification_strategy = NotificationStrategy::new();

    // Simulate delivery sequence events
    let base_time = Utc::now();
    
    let events = vec![
        // Event 1: Amazon van approaches (T+0s)
        SecurityEvent {
            id: "event_001".to_string(),
            timestamp: base_time,
            event_type: EventType::VehicleApproach,
            location: "street".to_string(),
            confidence: 0.8,
            alert_level: AlertLevel::Ignore,
        },
        
        // Event 2: Driver exits with package (T+45s)
        SecurityEvent {
            id: "event_002".to_string(),
            timestamp: base_time + Duration::seconds(45),
            event_type: EventType::PersonDetected,
            location: "driveway".to_string(),
            confidence: 0.75,
            alert_level: AlertLevel::Ignore,
        },
        
        // Event 3: Approaches front door (T+65s)
        SecurityEvent {
            id: "event_003".to_string(),
            timestamp: base_time + Duration::seconds(65),
            event_type: EventType::DoorApproach,
            location: "front_door".to_string(),
            confidence: 0.9,
            alert_level: AlertLevel::Ignore,
        },
        
        // Event 4: Package delivery completed (T+95s)
        SecurityEvent {
            id: "event_004".to_string(),
            timestamp: base_time + Duration::seconds(95),
            event_type: EventType::PackageDelivery,
            location: "front_door".to_string(),
            confidence: 0.95,
            alert_level: AlertLevel::Ignore,
        },
    ];

    println!("🎬 Processing delivery sequence events:\n");

    for (i, event) in events.iter().enumerate() {
        println!("⏰ T+{}s - Event {}: {:?} at {}", 
                 event.timestamp.signed_duration_since(base_time).num_seconds(),
                 i + 1,
                 event.event_type, 
                 event.location);

        // Correlate the event
        correlation_engine.correlate_event(event);

        // Decide on notification
        let decision = notification_strategy.decide_notification(event, &correlation_engine);

        // Display the decision
        match decision {
            NotificationDecision::Notify { message, priority } => {
                println!("   🔔 NOTIFICATION ({}): {}", priority, message);
            },
            NotificationDecision::Suppress { reason, correlation_id } => {
                println!("   🔇 SUPPRESSED: {} (Correlation: {:?})", reason, correlation_id);
            },
            NotificationDecision::Summary { message, event_count, correlation_id } => {
                println!("   📋 SUMMARY: {} ({} events, ID: {})", message, event_count, correlation_id);
            },
        }

        println!();
    }

    println!("📊 Final Correlation State:");
    for (id, corr_event) in &correlation_engine.active_events {
        println!("   Event Chain {}: {} events, Classification: {:?}", 
                 id, corr_event.event_chain.len(), corr_event.classification);
        println!("   Confidence Evolution: {:?}", corr_event.confidence_evolution);
        println!("   Suppression Count: {}", corr_event.suppression_count);
    }

    println!("\n✨ Key Features Demonstrated:");
    println!("   • Initial awareness notification for first event");
    println!("   • Intelligent suppression of redundant notifications");
    println!("   • Event correlation and sequence classification");
    println!("   • Summary notification upon completion");
    println!("   • User preference-driven behavior");
}

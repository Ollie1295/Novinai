#!/usr/bin/env python3
import re

# Read the original file
with open('src/core/mod.rs', 'r') as f:
    content = f.read()

# Fix 1: Add missing type definitions after MetaCognition struct
metacognition_pattern = r'(pub struct MetaCognition \{[^}]+\})'
replacement = r'''\1

/// Meta-cognitive monitoring for psychological profiling
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaCognitiveMonitoring {
    pub reasoning_confidence: f64,
    pub bias_detection: Vec<CognitiveBias>,
    pub uncertainty_sources: Vec<UncertaintySource>,
    pub reasoning_quality: f64,
    pub self_critique: Vec<SelfCritique>,
    pub cognitive_load: f64,
    pub attention_focus: f64,
}

/// Hierarchical attention mechanisms for threat analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HierarchicalAttention {
    pub spatial_attention: SpatialMap,
    pub temporal_attention: TemporalWeights,
    pub feature_attention: FeatureWeights,
    pub global_attention: f64,
    pub attention_hierarchy: Vec<String>,
    pub focus_intensity: f64,
}'''

content = re.sub(metacognition_pattern, replacement, content, flags=re.DOTALL)

# Fix 2: Fix the unused variable warning
content = content.replace(
    'pub fn calculate_multi_dimensional_alert(&self, context: &ThreatContext, threat_score: f64) -> AlertLevel {',
    'pub fn calculate_multi_dimensional_alert(&self, context: &ThreatContext, _threat_score: f64) -> AlertLevel {'
)

# Fix 3: Fix the borrow checker error by restructuring the function
old_function = '''    fn add_to_existing_correlation(&mut self, parent_id: &str, event: &SecurityEvent) {
        if let Some(corr_event) = self.active_events.get_mut(parent_id) {
            corr_event.event_chain.push(event.id.clone());
            corr_event.event_type_sequence.push(event.event_type.clone());
            corr_event.last_update = event.timestamp;
            corr_event.confidence_evolution.push(event.confidence);
            corr_event.suppression_count += 1;
            
            // Update classification based on sequence
            corr_event.classification = self.reclassify_sequence(corr_event);
        }
    }'''

new_function = '''    fn add_to_existing_correlation(&mut self, parent_id: &str, event: &SecurityEvent) {
        if let Some(corr_event) = self.active_events.get_mut(parent_id) {
            corr_event.event_chain.push(event.id.clone());
            corr_event.event_type_sequence.push(event.event_type.clone());
            corr_event.last_update = event.timestamp;
            corr_event.confidence_evolution.push(event.confidence);
            corr_event.suppression_count += 1;
            
            // Update classification based on sequence
            let new_classification = Self::reclassify_sequence_static(
                &corr_event.event_type_sequence,
                &corr_event.confidence_evolution,
                &corr_event.classification
            );
            corr_event.classification = new_classification;
        }
    }
    
    fn reclassify_sequence_static(
        sequence: &[EventType], 
        confidence_evolution: &[f64], 
        current_classification: &EventClassification
    ) -> EventClassification {
        use EventType::*;
        
        // Delivery pattern: Vehicle -> Person -> Door -> Package
        if sequence.len() >= 2 && 
           matches!(sequence[0], VehicleApproach) &&
           sequence.iter().any(|t| matches!(t, PersonDetected)) {
            return EventClassification::DeliverySequence;
        }
        
        // Known person movement pattern
        if sequence.iter().all(|t| matches!(t, PersonDetected)) &&
           confidence_evolution.iter().all(|&c| c > 0.7) {
            return EventClassification::KnownPersonMovement;
        }
        
        current_classification.clone()
    }'''

content = content.replace(old_function, new_function)

# Write the fixed content back
with open('src/core/mod.rs', 'w') as f:
    f.write(content)

print("All fixes applied!")

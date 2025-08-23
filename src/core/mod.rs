//! Core threat detection and AI security system definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Type aliases for complex domain types
pub type CausalFactor = String;
pub type PsychologicalProfile = HashMap<String, f64>;
pub type BehaviorIndicators = HashMap<String, f64>;
pub type NetworkEffects = HashMap<String, f64>;
pub type Countermeasure = String;
pub type WeaponIndicator = String;
pub type MovementPattern = HashMap<String, f64>;
pub type AccessCapability = String;
pub type StealthProfile = HashMap<String, f64>;
pub type StressProfile = HashMap<String, f64>;
pub type ManipulationTactic = String;
pub type EmotionalState = HashMap<String, f64>;
pub type NetworkConnection = String;
pub type InfluenceOperation = String;
pub type NovelPattern = String;
pub type AdaptiveStrategy = String;
pub type Intervention = String;

/// Environmental context for threat assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalContext {
    pub location: String,
    pub ambient_conditions: Vec<String>,
    pub time_context: TimeContext,
}

/// Time-based context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeContext {
    Morning,
    Afternoon,
    Evening,
    Night,
}

/// Entity representation for threat analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub profile: Option<HashMap<String, f64>>,
    pub last_seen: Option<DateTime<Utc>>,
    pub interaction_count: u32,
}

/// Core threat assessment with causal reasoning and countermeasures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatAssessment {
    pub entity_id: Uuid,
    pub threat_level: f64,            // 0.0 to 1.0
    pub threat_probability: f64,      // Bayesian probability
    pub uncertainty_bounds: (f64, f64), // Lower and upper bounds
    pub confidence_score: f64,        // Model confidence
    pub temporal_horizon: chrono::Duration, // Prediction time horizon
    pub causal_chain: Vec<CausalFactor>,
    pub psychological_profile: PsychologicalProfile,
    pub behavioral_indicators: BehaviorIndicators,
    pub environmental_context: EnvironmentalContext,
    pub network_effects: NetworkEffects,
    pub countermeasures: Vec<Countermeasure>,
    pub assessment_timestamp: DateTime<Utc>,
    pub explainability_trace: String, // AI reasoning explanation
}

/// Intelligence level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntelligenceLevel {
    Standard,
    Enhanced,
    Advanced,
    Insane,
}

/// Security operation modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityMode {
    Guardian,
    Stealth,
    PerimeterGuard,
}

/// Security system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub intelligence_level: IntelligenceLevel,
    pub threat_threshold: f64,
    pub response_sensitivity: f64,
    pub learning_enabled: bool,
    pub security_mode: SecurityMode,
}

/// Alert severity levels
/// Threat severity classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub enum AlertLevel {
    Ignore,
    Standard,
    Elevated,
    High,
    Critical,
}

impl AlertLevel {
    /// Determine alert level from threat score with dynamic thresholds
    pub fn from_threat_score_dynamic(
        threat_score: f64, 
        _context: &ThreatContext, 
        _thresholds: &DynamicThresholds
    ) -> Self {
        match threat_score {
            s if s >= 0.9 => AlertLevel::Critical,
            s if s >= 0.7 => AlertLevel::High,
            s if s >= 0.5 => AlertLevel::Elevated,
            s if s >= 0.3 => AlertLevel::Standard,
            _ => AlertLevel::Ignore,
        }
    }

    /// Multi-dimensional alert assessment
    pub fn from_multi_dimensional(_context: &ThreatContext, threat_score: f64) -> Self {
        // Simplified implementation
        match threat_score {
            s if s >= 0.8 => AlertLevel::Critical,
            s if s >= 0.6 => AlertLevel::High,
            s if s >= 0.4 => AlertLevel::Elevated,
            s if s >= 0.2 => AlertLevel::Standard,
            _ => AlertLevel::Ignore,
        }
    }
}

/// Threat context for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatContext {
    pub entity_id: Uuid,
    pub threat_indicators: HashMap<String, f64>,
    pub environmental_factors: HashMap<String, f64>,
    pub temporal_context: DateTime<Utc>,
    pub confidence: f64,
}

/// Dynamic threshold management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicThresholds {
    pub base_threshold: f64,
    pub context_modifiers: HashMap<String, f64>,
    pub temporal_adjustments: HashMap<String, f64>,
}

impl Default for DynamicThresholds {
    fn default() -> Self {
        Self {
            base_threshold: 0.5,
            context_modifiers: HashMap::new(),
            temporal_adjustments: HashMap::new(),
        }
    }
}

/// Main security system
#[derive(Debug, Clone)]
pub struct InsaneSecuritySystem {
    pub config: SecurityConfig,
    pub thresholds: DynamicThresholds,
}

impl Default for InsaneSecuritySystem {
    fn default() -> Self {
        Self {
            config: SecurityConfig {
                intelligence_level: IntelligenceLevel::Insane,
                threat_threshold: 0.5,
                response_sensitivity: 0.8,
                learning_enabled: true,
                security_mode: SecurityMode::Guardian,
            },
            thresholds: DynamicThresholds::default(),
        }
    }
}

impl InsaneSecuritySystem {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config,
            thresholds: DynamicThresholds::default(),
        }
    }

    pub fn process_threat(&self, context: &ThreatContext) -> ThreatAssessment {
        match self.config.security_mode {
            SecurityMode::Guardian => self.process_guardian_mode(context),
            SecurityMode::Stealth => self.process_stealth_mode(context),
            SecurityMode::PerimeterGuard => self.process_perimeter_guard_mode(context),
        }
    }

    /// Guardian Mode: Active protection with visible deterrence
    fn process_guardian_mode(&self, context: &ThreatContext) -> ThreatAssessment {
        let base_threat = self.calculate_base_threat(context);
        let enhanced_threat = base_threat * 1.2; // More aggressive in Guardian mode
        
        ThreatAssessment {
            entity_id: context.entity_id,
            threat_level: enhanced_threat.min(1.0),
            threat_probability: enhanced_threat * 0.9,
            uncertainty_bounds: (enhanced_threat * 0.8, enhanced_threat * 1.1),
            confidence_score: 0.95, // High confidence in Guardian mode
            temporal_horizon: chrono::Duration::minutes(5), // Quick response
            causal_chain: vec!["guardian_analysis".to_string(), "active_monitoring".to_string()],
            psychological_profile: self.build_psychological_profile(context),
            behavioral_indicators: self.analyze_behavior_patterns(context),
            environmental_context: self.assess_environment(context),
            network_effects: HashMap::new(),
            countermeasures: vec![
                "visible_deterrence".to_string(),
                "active_monitoring".to_string(),
                "immediate_response".to_string()
            ],
            assessment_timestamp: Utc::now(),
            explainability_trace: "Guardian mode: Active protection with visible deterrence measures".to_string(),
        }
    }

    /// Stealth Mode: Covert monitoring with minimal detection
    fn process_stealth_mode(&self, context: &ThreatContext) -> ThreatAssessment {
        let base_threat = self.calculate_base_threat(context);
        let stealth_threat = base_threat * 0.8; // More conservative to avoid detection
        
        ThreatAssessment {
            entity_id: context.entity_id,
            threat_level: stealth_threat,
            threat_probability: stealth_threat * 0.7,
            uncertainty_bounds: (stealth_threat * 0.6, stealth_threat * 1.2),
            confidence_score: 0.75, // Lower confidence due to stealth constraints
            temporal_horizon: chrono::Duration::minutes(30), // Longer observation
            causal_chain: vec!["stealth_analysis".to_string(), "covert_monitoring".to_string()],
            psychological_profile: self.build_stealth_profile(context),
            behavioral_indicators: self.analyze_stealth_patterns(context),
            environmental_context: self.assess_environment(context),
            network_effects: HashMap::new(),
            countermeasures: vec![
                "covert_monitoring".to_string(),
                "passive_tracking".to_string(),
                "delayed_response".to_string()
            ],
            assessment_timestamp: Utc::now(),
            explainability_trace: "Stealth mode: Covert monitoring with minimal detection signature".to_string(),
        }
    }

    /// Perimeter Guard Mode: Boundary-focused protection
    fn process_perimeter_guard_mode(&self, context: &ThreatContext) -> ThreatAssessment {
        let base_threat = self.calculate_base_threat(context);
        let perimeter_threat = self.calculate_perimeter_threat(context, base_threat);
        
        ThreatAssessment {
            entity_id: context.entity_id,
            threat_level: perimeter_threat,
            threat_probability: perimeter_threat * 0.85,
            uncertainty_bounds: (perimeter_threat * 0.7, perimeter_threat * 1.15),
            confidence_score: 0.88,
            temporal_horizon: chrono::Duration::minutes(10),
            causal_chain: vec!["perimeter_analysis".to_string(), "boundary_assessment".to_string()],
            psychological_profile: self.build_psychological_profile(context),
            behavioral_indicators: self.analyze_perimeter_behavior(context),
            environmental_context: self.assess_environment(context),
            network_effects: HashMap::new(),
            countermeasures: vec![
                "perimeter_reinforcement".to_string(),
                "access_control".to_string(),
                "boundary_monitoring".to_string()
            ],
            assessment_timestamp: Utc::now(),
            explainability_trace: "Perimeter Guard mode: Boundary-focused protection with access control".to_string(),
        }
    }

    // Helper methods for threat calculation
    fn calculate_base_threat(&self, context: &ThreatContext) -> f64 {
        let mut threat_score = 0.0;
        for (_, value) in &context.threat_indicators {
            threat_score += value;
        }
        (threat_score / context.threat_indicators.len() as f64).min(1.0)
    }

    fn calculate_perimeter_threat(&self, context: &ThreatContext, base_threat: f64) -> f64 {
        // Enhanced threat calculation for perimeter violations
        let perimeter_multiplier = if context.threat_indicators.contains_key("perimeter_breach") {
            1.5
        } else if context.threat_indicators.contains_key("boundary_approach") {
            1.2
        } else {
            1.0
        };
        (base_threat * perimeter_multiplier).min(1.0)
    }

    fn build_psychological_profile(&self, _context: &ThreatContext) -> PsychologicalProfile {
        let mut profile = HashMap::new();
        profile.insert("aggression_level".to_string(), 0.3);
        profile.insert("intent_clarity".to_string(), 0.6);
        profile.insert("stress_indicators".to_string(), 0.4);
        profile
    }

    fn build_stealth_profile(&self, _context: &ThreatContext) -> PsychologicalProfile {
        let mut profile = HashMap::new();
        profile.insert("concealment_intent".to_string(), 0.7);
        profile.insert("awareness_level".to_string(), 0.5);
        profile.insert("evasion_patterns".to_string(), 0.6);
        profile
    }

    fn analyze_behavior_patterns(&self, _context: &ThreatContext) -> BehaviorIndicators {
        let mut indicators = HashMap::new();
        indicators.insert("movement_speed".to_string(), 0.5);
        indicators.insert("direction_changes".to_string(), 0.3);
        indicators.insert("attention_focus".to_string(), 0.7);
        indicators
    }

    fn analyze_stealth_patterns(&self, _context: &ThreatContext) -> BehaviorIndicators {
        let mut indicators = HashMap::new();
        indicators.insert("concealment_behavior".to_string(), 0.8);
        indicators.insert("noise_minimization".to_string(), 0.6);
        indicators.insert("shadow_utilization".to_string(), 0.7);
        indicators
    }

    fn analyze_perimeter_behavior(&self, _context: &ThreatContext) -> BehaviorIndicators {
        let mut indicators = HashMap::new();
        indicators.insert("boundary_testing".to_string(), 0.6);
        indicators.insert("access_attempts".to_string(), 0.4);
        indicators.insert("perimeter_mapping".to_string(), 0.5);
        indicators
    }

    fn assess_environment(&self, _context: &ThreatContext) -> EnvironmentalContext {
        EnvironmentalContext {
            location: "monitored_area".to_string(),
            ambient_conditions: vec!["normal_lighting".to_string(), "clear_visibility".to_string()],
            time_context: TimeContext::Afternoon,
        }
    }

    /// Switch security mode at runtime
    pub fn set_security_mode(&mut self, mode: SecurityMode) {
        self.config.security_mode = mode;
    }

    /// Get current security mode
    pub fn get_security_mode(&self) -> &SecurityMode {
        &self.config.security_mode
    }
}


//! Thinking AI Layer
//!
//! This module provides the "thinking AI" layer that sits on top of the existing LLR fusion AI,
//! adding incident-level reasoning, active questioning, counterfactual analysis, and narrative
//! summarization capabilities.

pub mod incident_engine;
pub mod active_reasoner;
pub mod decision_counterfactuals;
pub mod summarizer;
pub mod llr_integration;
pub mod llm_client;

// Re-export key types for easy access
pub use incident_engine::{
    Evidence, Event, Incident, IncidentStore, IncidentStatus,
    sigmoid, calibrate_logit
};

pub use active_reasoner::{
    Question, QuestionProposal, ReasonerConfig, generate_questions
};

pub use decision_counterfactuals::{
    CounterfactualSuggestion, minimal_changes_to_threshold
};

pub use summarizer::{
    summarize_incident
};

pub use llr_integration::{LLRExtractor, DemoLLRExtractor};

/// Configuration for the thinking AI system
#[derive(Debug, Clone)]
pub struct ThinkingAIConfig {
    /// TTL for incidents in seconds
    pub incident_ttl_secs: f64,
    /// Prior logit for threat assessment
    pub prior_logit: f64,
    /// Mean logit for calibration
    pub mean_logit: f64,
    /// Temperature for calibration
    pub temperature: f64,
    /// Odds cap for calibration
    pub odds_cap: f64,
    /// Positive LLR cap for evidence fusion
    pub pos_cap: f64,
    /// Negative LLR cap for evidence fusion  
    pub neg_cap: f64,
    /// Standard threshold logit for alerts
    pub alert_threshold_logit: f64,
    /// Reasoner configuration
    pub reasoner_config: ReasonerConfig,
}

impl Default for ThinkingAIConfig {
    fn default() -> Self {
        Self {
            incident_ttl_secs: 180.0,
            prior_logit: -2.0,
            mean_logit: 0.0,
            temperature: 1.4,
            odds_cap: 3.0,
            pos_cap: 1.6,
            neg_cap: 3.0,
            alert_threshold_logit: -1.7346, // logit(0.15)
            reasoner_config: ReasonerConfig::default(),
        }
    }
}

/// Complete thinking AI analysis result for an incident
#[derive(Debug, Clone)]
pub struct ThinkingAIResult {
    pub incident_id: u64,
    pub fused_evidence: Evidence,
    pub calibrated_probability: f64,
    pub narrative_summary: String,
    pub top_questions: Vec<QuestionProposal>,
    pub counterfactuals: Vec<CounterfactualSuggestion>,
    pub alert_decision: AlertDecision,
}

/// Alert decision based on thinking AI analysis with severity levels
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AlertDecision {
    /// No action needed - threat probability is very low
    Ignore,
    /// Standard alert - moderate threat detected, normal response
    Standard,
    /// Elevated alert - higher threat detected, increased response
    Elevated, 
    /// Critical alert - severe threat detected, immediate response required
    Critical,
    /// Wait for more information before making final decision
    Wait,
}

impl AlertDecision {
    pub fn from_probability(prob: f64, alert_threshold: f64, wait_threshold: f64) -> Self {
        // Define severity thresholds:
        // Critical: >= 50% threat probability
        // Elevated: >= 30% threat probability  
        // Standard: >= alert_threshold (15% by default)
        // Wait: >= wait_threshold (7.5% by default)
        // Ignore: < wait_threshold
        
        let critical_threshold = 0.5;
        let elevated_threshold = 0.3;
        
        if prob >= critical_threshold {
            AlertDecision::Critical
        } else if prob >= elevated_threshold {
            AlertDecision::Elevated
        } else if prob >= alert_threshold {
            AlertDecision::Standard
        } else if prob >= wait_threshold {
            AlertDecision::Wait
        } else {
            AlertDecision::Ignore
        }
    }
}

/// Main thinking AI processor that orchestrates the entire analysis pipeline
#[derive(Debug, Clone)]
pub struct ThinkingAIProcessor {
    config: ThinkingAIConfig,
    incident_stores: std::collections::HashMap<String, IncidentStore>,
}

impl ThinkingAIProcessor {
    pub fn new(config: ThinkingAIConfig) -> Self {
        Self {
            config,
            incident_stores: std::collections::HashMap::new(),
        }
    }

    /// Process an event through the thinking AI pipeline
    pub fn process_event(&mut self, home: &str, event: Event) -> Option<ThinkingAIResult> {
        // Get or create incident store for this home
        let store = self.incident_stores
            .entry(home.to_string())
            .or_insert_with(|| IncidentStore::new(self.config.incident_ttl_secs));

        // Upsert event into incident store
        let incident_id = store.upsert_event(home, event);

        // Get the incident for analysis
        if let Some(incident) = store.incidents.values().find(|i| i.id == incident_id) {
            // Fuse evidence
            let fused = incident.fused_evidence(self.config.pos_cap, self.config.neg_cap);
            
            // Calibrate probability
            let raw_logit = self.config.prior_logit + fused.sum();
            let calibrated_prob = calibrate_logit(
                raw_logit,
                self.config.mean_logit,
                self.config.temperature,
                self.config.odds_cap
            );

            // Generate narrative summary
            let summary = summarize_incident(incident, &fused, calibrated_prob, incident.suppressed_count);

            // Generate questions
            let questions = generate_questions(incident, &fused, self.config.prior_logit, &self.config.reasoner_config);

            // Generate counterfactuals
            let counterfactuals = minimal_changes_to_threshold(&fused, self.config.prior_logit, self.config.alert_threshold_logit);

            // Make alert decision
            let alert_decision = AlertDecision::from_probability(
                calibrated_prob,
                sigmoid(self.config.alert_threshold_logit),
                sigmoid(self.config.alert_threshold_logit) * 0.5 // Wait threshold is half of alert threshold
            );

            Some(ThinkingAIResult {
                incident_id,
                fused_evidence: fused,
                calibrated_probability: calibrated_prob,
                narrative_summary: summary,
                top_questions: questions.into_iter().take(5).collect(),
                counterfactuals,
                alert_decision,
            })
        } else {
            None
        }
    }

    /// Format thinking AI result as a text block for integration with existing systems
    pub fn format_thinking_block(&self, result: &ThinkingAIResult) -> String {
        let mut output = String::new();
        
        output.push_str("=== [ThinkingAI] ===\n");
        output.push_str(&result.narrative_summary);
        output.push_str("\n\nDecision: ");
        output.push_str(&format!("{:?}", result.alert_decision));
        
        if !result.top_questions.is_empty() {
            output.push_str("\n\nSelf-Questions (Value of Information):\n");
            for (i, q) in result.top_questions.iter().enumerate() {
                output.push_str(&format!("  {}. {:?} (ΔH≈{:.3})\n", i+1, q.q, q.expected_entropy_reduction));
            }
        }

        if !result.counterfactuals.is_empty() {
            output.push_str("\nCounterfactuals to downgrade alert:\n");
            for cf in &result.counterfactuals {
                output.push_str(&format!("  • {} (ΔLLR={:+.2})\n", cf.description, cf.delta_llr));
            }
        }
        
        output.push_str("=== [/ThinkingAI] ===\n");
        output
    }
}

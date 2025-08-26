//! Insane AI Security System
//! 
//! A next-generation security system with predictive threat modeling,
//! psychological profiling, and emergent intelligence capabilities.

pub mod core;
pub mod api;
pub mod prediction;
pub mod core;
pub mod pipeline;
pub mod vps_client;
pub mod thinking;
pub mod overnight;
pub mod image_preloader;

// pub mod observability;
// pub mod config;

#[cfg(test)]
mod tests;

pub use core::*;

/// Core result type for the security system
pub type SecurityResult<T> = anyhow::Result<T>;

/// System-wide configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemConfig {
    pub intelligence_level: IntelligenceLevel,
    pub prediction_horizon: std::time::Duration,
    pub learning_rate: f64,
    pub safety_threshold: f64,
    pub adversarial_mode: bool,
    pub emergent_discovery: bool,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum IntelligenceLevel {
    Standard,
    Enhanced,
    Insane,
    Godlike,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            intelligence_level: IntelligenceLevel::Insane,
            prediction_horizon: std::time::Duration::from_secs(3600), // 1 hour
            learning_rate: 0.001,
            safety_threshold: 0.95,
            adversarial_mode: true,
            emergent_discovery: true,
        }
    }
}

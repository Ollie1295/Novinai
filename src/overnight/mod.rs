//! Overnight Review System
//!
//! This module implements an intelligent overnight monitoring system that:
//! - Collects events during user-defined overnight hours
//! - Suppresses immediate alerts during sleep periods
//! - Generates AI-powered morning summaries
//! - Provides customizable review periods per home

pub mod config;
pub mod storage;
pub mod summary;
pub mod scheduler;
pub mod manager;

use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::thinking::AlertDecision;

/// Overnight review configuration for a home
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OvernightConfig {
    pub home_id: String,
    pub review_start_time: NaiveTime, // e.g., 22:00
    pub review_end_time: NaiveTime,   // e.g., 06:00
    pub summary_delivery_time: NaiveTime, // e.g., 07:00
    pub timezone: String, // e.g., "America/New_York"
    pub enabled: bool,
    pub delivery_channels: Vec<DeliveryChannel>,
}

impl Default for OvernightConfig {
    fn default() -> Self {
        Self {
            home_id: String::new(),
            review_start_time: NaiveTime::from_hms_opt(22, 0, 0).unwrap(), // 10 PM
            review_end_time: NaiveTime::from_hms_opt(6, 0, 0).unwrap(),    // 6 AM
            summary_delivery_time: NaiveTime::from_hms_opt(7, 0, 0).unwrap(), // 7 AM
            timezone: "UTC".to_string(),
            enabled: true,
            delivery_channels: vec![DeliveryChannel::Push, DeliveryChannel::WebSocket],
        }
    }
}

/// Available delivery channels for morning summaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryChannel {
    Push,       // Mobile push notification
    Email,      // Email summary
    WebSocket,  // Real-time to connected clients
    SMS,        // Text message
    Dashboard,  // Web dashboard notification
}

/// Result type for overnight review operations
pub type OvernightResult<T> = anyhow::Result<T>;

/// Errors that can occur in the overnight review system
#[derive(thiserror::Error, Debug)]
pub enum OvernightError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Summary generation error: {0}")]
    Summary(String),
    
    #[error("Delivery error: {0}")]
    Delivery(String),
    
    #[error("Scheduler error: {0}")]
    Scheduler(String),
}

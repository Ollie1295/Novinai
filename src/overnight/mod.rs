//! Overnight Review System

pub mod config;
pub mod storage;
pub mod summary;
pub mod manager;

// Re-export key types
pub use manager::{OvernightReviewManager, OvernightEventAnalysis, MorningSummary};
pub use storage::{OvernightStorageFactory, OvernightStorage};
pub use summary::SummaryTone;

use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OvernightConfig {
    pub home_id: String,
    pub review_start_time: NaiveTime,
    pub review_end_time: NaiveTime,   
    pub summary_delivery_time: NaiveTime,
    pub timezone: String,
    pub enabled: bool,
    pub delivery_channels: Vec<DeliveryChannel>,
}

impl Default for OvernightConfig {
    fn default() -> Self {
        Self {
            home_id: String::new(),
            review_start_time: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            review_end_time: NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            summary_delivery_time: NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            timezone: "UTC".to_string(),
            enabled: true,
            delivery_channels: vec![DeliveryChannel::Push, DeliveryChannel::WebSocket],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryChannel {
    Push,
    Email,
    WebSocket,
    SMS,
    Dashboard,
}

pub type OvernightResult<T> = anyhow::Result<T>;

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

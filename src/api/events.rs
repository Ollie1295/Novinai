//! Event Ingestion API
use axum::{
    extract::{Json, Path},
    response::{Result, Json as ResponseJson},
    http::StatusCode,
};
use crate::pipeline::{RawEvent, ProcessedEvent, SubscriptionTier, EventPipeline, PipelineConfig};
use crate::vps_client::VpsApiClient;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Deserialize)]
pub struct EventSubmission {
    pub sensor_id: String,
    pub data: String,
    pub user_id: String,
    pub home_id: String,
    pub api_key: String,
    pub subscription_tier: SubscriptionTier,
}

#[derive(Debug, Serialize)]
pub struct EventResponse {
    pub event_id: Uuid,
    pub status: String,
    pub message: String,
    pub processed_at: i64,
}

/// Submit an event for processing through the full AI pipeline
pub async fn submit_event(Json(submission): Json<EventSubmission>) -> Result<ResponseJson<EventResponse>, StatusCode> {
    let event_id = Uuid::new_v4();
    
    // Create raw event
    let raw_event = RawEvent {
        event_id,
        sensor_id: submission.sensor_id,
        timestamp: Utc::now().timestamp(),
        data: submission.data,
        user_id: submission.user_id,
        home_id: submission.home_id,
    };

    // Initialize pipeline
    let vps_client = VpsApiClient::new("https://api.vps.example.com".to_string());
    let config = PipelineConfig::default();
    let mut pipeline = EventPipeline::new(config, vps_client);

    // Process through full AI pipeline
    match pipeline.process_event(raw_event, submission.subscription_tier, &submission.api_key).await {
        Ok(processed_event) => {
            println!("✅ Event {} processed successfully: {}", event_id, processed_event.result_summary);
            
            Ok(ResponseJson(EventResponse {
                event_id,
                status: "processed".to_string(),
                message: format!("Event processed through AI pipeline: {}", processed_event.result_summary),
                processed_at: Utc::now().timestamp(),
            }))
        }
        Err(e) => {
            eprintln!("❌ Event processing error: {}", e);
            Ok(ResponseJson(EventResponse {
                event_id,
                status: "error".to_string(),
                message: format!("Processing failed: {}", e),
                processed_at: Utc::now().timestamp(),
            }))
        }
    }
}

/// Get recent events for a home
pub async fn get_events(Path(_home_id): Path<String>) -> Result<ResponseJson<Vec<ProcessedEvent>>, StatusCode> {
    Ok(ResponseJson(vec![]))
}

/// Health check
pub async fn health() -> ResponseJson<serde_json::Value> {
    ResponseJson(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now().timestamp(),
        "service": "event-ingestion",
        "pipeline": "ready"
    }))
}

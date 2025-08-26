#!/usr/bin/env rust-script
//! HTTP to NATS Sidecar
//! 
//! A tiny service that receives HTTP POST requests and forwards them to NATS.
//! This allows existing systems to integrate with the Deep-K NATS event bus
//! without code changes.

use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::env;
use tokio::net::TcpListener;
use tracing::{info, warn, error};

#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    service: String,
    nats_connected: bool,
    uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ForwardResponse {
    success: bool,
    message: String,
    subject: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::init();

    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://127.0.0.1:4222".to_string());
    let http_port = env::var("HTTP_PORT").unwrap_or_else(|_| "8080".to_string());

    info!("🚀 Starting HTTP-to-NATS Sidecar");
    info!("📡 NATS URL: {}", nats_url);
    info!("🌐 HTTP Port: {}", http_port);

    // Connect to NATS
    let nc = async_nats::connect(&nats_url).await?;
    info!("✅ Connected to NATS");

    // Create shared state
    let app_state = AppState {
        nats_client: nc,
        start_time: std::time::Instant::now(),
    };

    // Build the router
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/forward/:subject", post(forward_handler))
        .route("/perception/:camera_id", post(perception_handler))
        .with_state(app_state);

    // Start the server
    let addr = format!("0.0.0.0:{}", http_port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("🌐 HTTP server listening on {}", addr);
    info!("📋 Available endpoints:");
    info!("  GET  /health                     - Health check");
    info!("  POST /forward/:subject           - Forward JSON to any NATS subject");
    info!("  POST /perception/:camera_id      - Forward perception event to events.perception.<camera_id>");

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    nats_client: async_nats::Client,
    start_time: std::time::Instant,
}

async fn health_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<HealthResponse> {
    let uptime = state.start_time.elapsed().as_secs();
    
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "http-to-nats-sidecar".to_string(),
        nats_connected: !state.nats_client.is_closed(),
        uptime_seconds: uptime,
    })
}

async fn forward_handler(
    Path(subject): Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
    body: String,
) -> Result<Json<ForwardResponse>, StatusCode> {
    info!("📨 Forwarding to subject: {}", subject);

    // Validate JSON
    if let Err(_) = serde_json::from_str::<serde_json::Value>(&body) {
        warn!("❌ Invalid JSON in request body");
        return Ok(Json(ForwardResponse {
            success: false,
            message: "Invalid JSON format".to_string(),
            subject: Some(subject),
        }));
    }

    // Forward to NATS
    match state.nats_client.publish(subject.clone(), body.into()).await {
        Ok(_) => {
            info!("✅ Successfully forwarded to {}", subject);
            Ok(Json(ForwardResponse {
                success: true,
                message: "Message forwarded successfully".to_string(),
                subject: Some(subject),
            }))
        }
        Err(e) => {
            error!("❌ Failed to publish to NATS: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn perception_handler(
    Path(camera_id): Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
    body: String,
) -> Result<Json<ForwardResponse>, StatusCode> {
    let subject = format!("events.perception.{}", camera_id);
    info!("🎯 Perception event for camera: {} -> {}", camera_id, subject);

    // Validate JSON structure for perception events
    match serde_json::from_str::<serde_json::Value>(&body) {
        Ok(json) => {
            // Basic validation for perception event structure
            let required_fields = ["person", "vehicle", "pet", "timestamp"];
            for field in &required_fields {
                if !json.get(field).is_some() {
                    warn!("⚠️  Missing required field: {}", field);
                }
            }
        }
        Err(_) => {
            warn!("❌ Invalid JSON in perception event");
            return Ok(Json(ForwardResponse {
                success: false,
                message: "Invalid JSON format for perception event".to_string(),
                subject: Some(subject),
            }));
        }
    }

    // Forward to NATS
    match state.nats_client.publish(subject.clone(), body.into()).await {
        Ok(_) => {
            info!("✅ Perception event forwarded to {}", subject);
            Ok(Json(ForwardResponse {
                success: true,
                message: format!("Perception event forwarded for camera: {}", camera_id),
                subject: Some(subject),
            }))
        }
        Err(e) => {
            error!("❌ Failed to publish perception event: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

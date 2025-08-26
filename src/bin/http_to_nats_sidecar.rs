//! HTTP to NATS Sidecar
//! 
//! A service that receives HTTP POST requests and forwards them to NATS.
//! This allows existing systems to integrate with the Deep-K NATS event bus
//! without code changes.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{env, time::Instant};
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

#[derive(Clone)]
struct AppState {
    nats_client: async_nats::Client,
    start_time: Instant,
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
    info!("⏳ Connecting to NATS...");
    let nc = async_nats::connect(&nats_url).await?;
    info!("✅ Connected to NATS successfully");

    // Create shared state
    let app_state = AppState {
        nats_client: nc,
        start_time: Instant::now(),
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
    info!("");
    info!("💡 Example usage:");
    info!("  curl -X POST http://{}:8080/perception/front_door -d '{{\"person\":0.8,\"timestamp\":\"2025-08-26T12:00:00Z\"}}'", 
          if addr.starts_with("0.0.0.0") { "localhost" } else { &addr });

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
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
    State(state): State<AppState>,
    body: String,
) -> Result<Json<ForwardResponse>, StatusCode> {
    info!("📨 Forwarding to subject: {}", subject);

    // Validate JSON
    if serde_json::from_str::<serde_json::Value>(&body).is_err() {
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
    State(state): State<AppState>,
    body: String,
) -> Result<Json<ForwardResponse>, StatusCode> {
    let subject = format!("events.perception.{}", camera_id);
    info!("🎯 Perception event for camera: {} -> {}", camera_id, subject);

    // Validate JSON structure for perception events
    match serde_json::from_str::<serde_json::Value>(&body) {
        Ok(json) => {
            // Basic validation for perception event structure
            let required_fields = ["person", "vehicle", "pet", "timestamp"];
            let mut missing_fields = Vec::new();
            
            for field in &required_fields {
                if json.get(field).is_none() {
                    missing_fields.push(*field);
                }
            }
            
            if !missing_fields.is_empty() {
                warn!("⚠️  Missing required fields: {:?}", missing_fields);
                info!("💡 Expected fields: person, vehicle, pet, timestamp");
            }

            // Log the event for debugging
            if let Some(person) = json.get("person").and_then(|v| v.as_f64()) {
                info!("🔍 Perception scores - Person: {:.2}", person);
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

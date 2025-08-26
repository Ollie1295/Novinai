//! Pipeline Test Server
//! 
//! Complete event ingestion pipeline with AI processing

use axum::{
    routing::{get, post},
    Router,
    serve,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use insane_ai_security::api::events;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Build the router
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(events::health))
        .route("/api/events", post(events::submit_event))
        .route("/api/events/:home_id", get(events::get_events))
        .layer(CorsLayer::permissive());

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(&addr).await?;
    
    println!("🚀 NOVINAI AI SECURITY PIPELINE RUNNING");
    println!("   📡 Event Ingestion: POST http://{}/api/events", addr);
    println!("   📊 Health Check:    GET  http://{}/health", addr);
    println!("   🧠 AI Processing:   ENABLED");
    println!("   🌙 Overnight Mode:  ENABLED");
    println!();
    println!("✅ READY TO RECEIVE AND PROCESS EVENTS!");

    serve(listener, app).await?;
    Ok(())
}

async fn root_handler() -> axum::response::Json<serde_json::Value> {
    axum::response::Json(serde_json::json!({
        "name": "NOVINAI AI Security Pipeline",
        "version": "1.0.0",
        "status": "operational",
        "ai_pipeline": "active",
        "overnight_review": "enabled",
        "endpoints": {
            "submit_event": "POST /api/events",
            "get_events": "GET /api/events/{home_id}",
            "health": "GET /health"
        }
    }))
}

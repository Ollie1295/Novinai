//! API Server with Event Ingestion
//! 
//! Complete API server that can receive and process events from cameras/sensors

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
async fn main() {
    // Initialize tracing
    tracing_subscriber::init();

    // Build the router with all endpoints
    let app = Router::new()
        // Health and status endpoints
        .route("/", get(root_handler))
        .route("/health", get(events::health))
        
        // Event ingestion endpoints  
        .route("/api/events", post(events::submit_event))
        .route("/api/events/:home_id", get(events::get_events))
        .route("/api/events/upload", post(events::upload_media))
        
        // CORS layer for web clients
        .layer(CorsLayer::permissive());

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(&addr).await.unwrap();
    
    println!("🚀 Novinai API Server starting on http://{}", addr);
    println!("   Event Ingestion: POST http://{}/api/events", addr);
    println!("   Event History:   GET  http://{}/api/events/{{home_id}}", addr);
    println!("   Media Upload:    POST http://{}/api/events/upload", addr);
    println!("   Health Check:    GET  http://{}/health", addr);
    println!();
    println!("✅ Ready to receive events from cameras and sensors!");

    serve(listener, app).await.unwrap();
}

async fn root_handler() -> axum::response::Json<serde_json::Value> {
    axum::response::Json(serde_json::json!({
        "name": "Novinai AI Security System",
        "version": "1.0.0",
        "status": "operational",
        "features": [
            "Event Ingestion",
            "AI Processing",
            "Overnight Review",
            "Morning Summaries",
            "Multi-Channel Delivery"
        ]
    }))
}

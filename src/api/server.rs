use axum::{
    routing::get,
    Router,
    response::Json,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use serde_json::json;

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub static_files_dir: Option<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            static_files_dir: None,
        }
    }
}

pub struct ApiServer {
    pub config: ApiConfig,
}

impl ApiServer {
    pub fn new(config: ApiConfig) -> Self {
        Self { config }
    }

    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        let app = Router::new()
            .route("/", get(root_handler))
            .route("/health", get(health_handler))
            .route("/api/status", get(status_handler))
            .layer(CorsLayer::permissive());

        let addr = SocketAddr::from(([127, 0, 0, 1], self.config.port));
        
        println!("🚀 API Server starting on http://{}", addr);
        println!("   REST API: http://{}/api", addr);
        println!("   Health: http://{}/health", addr);
        println!();

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn root_handler() -> Json<serde_json::Value> {
    Json(json!({
        "name": "Insane AI Security API",
        "version": "1.0.0",
        "status": "operational"
    }))
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    }))
}

async fn status_handler() -> Json<serde_json::Value> {
    Json(json!({
        "api_status": "running",
        "ai_system": "operational", 
        "alert_severity_levels": {
            "critical": ">= 50%",
            "elevated": ">= 30%", 
            "standard": ">= 15%",
            "wait": ">= 7.5%",
            "ignore": "< 7.5%"
        },
        "timestamp": chrono::Utc::now()
    }))
}

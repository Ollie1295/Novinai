use axum::Router;
use sqlx::SqlitePool;
use std::sync::Arc;
use super::websocket::WebSocketManager;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: SqlitePool,
    pub websocket_manager: Arc<WebSocketManager>,
}

impl AppState {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self { 
            db_pool, 
            websocket_manager: Arc::new(WebSocketManager::new()) 
        }
    }
}

pub fn create_routes(_state: AppState) -> Router {
    use axum::routing::get;
    Router::new()
        .route("/api/system/health", get(|| async { "OK" }))
}

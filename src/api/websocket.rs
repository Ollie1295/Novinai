pub struct WebSocketManager;

impl WebSocketManager {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn get_client_count(&self) -> usize {
        0
    }
}

#[derive(serde::Serialize)]
pub struct WebSocketStats {
    pub connected_clients: usize,
    pub active_subscriptions: usize,
    pub messages_sent_today: u64,
    pub uptime_seconds: u64,
}

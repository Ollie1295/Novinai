// src/bin/pipeline_daemon.rs

use insane_ai_security::pipeline::*;
use insane_ai_security::vps_client::*;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

// Mock VPS API Server
async fn mock_vps_server() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Mock VPS API server listening on 127.0.0.1:8080");

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            // This is a simplified mock, in a real scenario we would parse the request
            // and respond accordingly.
            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"job_id\":\"mock-job-123\",\"status\":\"processing\"}";
            use tokio::io::AsyncWriteExt;
            let mut stream = socket;
            stream.write_all(response.as_bytes()).await.unwrap();
        });
    }
}

#[tokio::main]
async fn main() {
    // Start the mock VPS server in the background
    tokio::spawn(mock_vps_server());

    // -- Setup for the Event Pipeline --
    let config = PipelineConfig::default();

    // -- Initialize the real VPS API client --
    let vps_api_client = VpsApiClient::new("http://127.0.0.1:8080".to_string());

    // -- Create the event pipeline with the real client --
    let mut pipeline = EventPipeline::new(config, vps_api_client);

    println!("🚀 Event Pipeline Daemon started.");
    println!("Listening for events...");

    // -- Simulate receiving events --
    let mut event_counter = 0;
    loop {
        sleep(Duration::from_secs(5)).await;
        event_counter += 1;

        let user_id = format!("user_{}", (event_counter % 3) + 1);
        let home_id = format!("home_{}", (event_counter % 2) + 1);
        let tier = match event_counter % 3 {
            0 => SubscriptionTier::Free,
            1 => SubscriptionTier::Standard,
            _ => SubscriptionTier::Premium,
        };

        let event = RawEvent {
            event_id: Uuid::new_v4(),
            sensor_id: format!("cam-{:02}", (event_counter % 4) + 1),
            timestamp: chrono::Utc::now().timestamp(),
            data: "[simulated_image_data]".to_string(),
            user_id,
            home_id,
        };

        println!("\n---\n📨 Received event {} for tier {:?}", event.event_id, tier);

        match pipeline.process_event(event, tier.clone(), "test-api-key").await {
            Ok(processed_event) => {
                println!("✅ Event processed successfully:");
                println!("   Job ID: {}", processed_event.vps_job_id);
                println!("   Status: {}", processed_event.status);
                println!("   Summary: {}", processed_event.result_summary);
                
                if let Some(thinking_analysis) = &processed_event.thinking_ai_analysis {
                    println!("\n{}", thinking_analysis);
                }
            }
            Err(e) => {
                eprintln!("🔥 Error processing event: {}", e);
            }
        }
    }
}

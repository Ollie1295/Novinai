use insane_ai_security::api::{ApiServer, ApiConfig};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create server configuration  
    let config = ApiConfig::default();

    println!("🚀 Starting Insane AI Security API Server");
    println!("📡 Configuration:");
    println!("   Host: {}", config.host);
    println!("   Port: {}", config.port);
    println!("   Static files: {:?}", config.static_files_dir);

    // Create and start the API server
    let server = ApiServer::new(config);
    
    // This will start the server and block until it's shut down
    server.serve().await?;

    Ok(())
}

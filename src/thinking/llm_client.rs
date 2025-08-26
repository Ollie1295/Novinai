use reqwest;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
pub struct LLMSummaryRequest {
    pub decision: String,
    pub location: String,
    pub dwell_time: f64,
    pub rang_doorbell: bool,
    pub knocked: bool,
    pub threat_probability: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LLMSummaryResponse {
    pub success: bool,
    pub summary: Option<String>,
    pub style: Option<String>,
    pub model: Option<String>,
    pub error: Option<String>,
    pub fallback_reason: Option<String>,
}

pub struct LLMClient {
    client: reqwest::Client,
    base_url: String,
}

impl LLMClient {
    pub fn new(base_url: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))  // Reasonable timeout for LLM calls
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            client,
            base_url: base_url.unwrap_or_else(|| "http://127.0.0.1:8765".to_string()),
        }
    }
    
    /// Attempt to get an LLM-generated summary
    pub async fn get_summary(&self, request: LLMSummaryRequest) -> Option<String> {
        match self.try_get_summary(request).await {
            Ok(response) if response.success => {
                if let Some(summary) = response.summary {
                    Some(format!("🤖 {}", summary))  // Prefix to indicate LLM generated
                } else {
                    None
                }
            }
            Ok(response) => {
                eprintln!("LLM summary failed: {:?}", response.error);
                None
            }
            Err(e) => {
                eprintln!("LLM service error: {}", e);
                None
            }
        }
    }
    
    async fn try_get_summary(&self, request: LLMSummaryRequest) -> Result<LLMSummaryResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/summary", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;
            
        let status = response.status();
            
        if status.is_success() {
            let summary_response: LLMSummaryResponse = response.json().await?;
            Ok(summary_response)
        } else {
            // Try to parse error response
            match response.json::<LLMSummaryResponse>().await {
                Ok(error_response) => Ok(error_response),
                Err(_) => Ok(LLMSummaryResponse {
                    success: false,
                    summary: None,
                    style: None,
                    model: None,
                    error: Some(format!("HTTP {}", status)),
                    fallback_reason: None,
                })
            }
        }
    }
    
    /// Check if the LLM service is healthy and responsive
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/health", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}

// src/vps_client.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use bytes::Bytes;
use std::error::Error;

// Represents the response from the VPS API for a processing request
#[derive(Serialize, Deserialize, Debug)]
pub struct VpsProcessingResponse {
    pub job_id: String,
    pub status: String,
    pub result_url: Option<String>,
    pub error_message: Option<String>,
}

// Represents the payload for a processing request
#[derive(Serialize, Deserialize, Debug)]
pub struct VpsProcessingRequest {
    pub event_id: String,
    pub sensor_data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_data: Option<Bytes>, // Pre-downloaded image data
    pub processing_level: String,
    pub user_context: String,
}

// A client for interacting with the real VPS API
#[derive(Debug)]
pub struct VpsApiClient {
    client: Client,
    api_base_url: String,
}

impl VpsApiClient {
    // Creates a new API client
    pub fn new(api_base_url: String) -> Self {
        VpsApiClient {
            client: Client::new(),
            api_base_url,
        }
    }

    // Submits an event for processing to the VPS
    pub async fn process_event(
        &self,
        request: VpsProcessingRequest,
    ) -> Result<VpsProcessingResponse, Box<dyn Error>> {
        let url = format!("{}/v1/process", self.api_base_url);
        
        let response = self.client.post(&url)
            .json(request)
            .send()
            .await?;

        if response.status().is_success() {
            let processing_response = response.json::<VpsProcessingResponse>().await?;
            Ok(processing_response)
        } else {
            let error_text = response.text().await?;
            Err(format!("API Error: {}", error_text).into())
        }
    }
}

// src/pipeline.rs

use crate::vps_client::{VpsApiClient, VpsProcessingRequest};
use crate::thinking::{ThinkingAIProcessor, ThinkingAIConfig, Event, Evidence, LLRExtractor, DemoLLRExtractor};
use crate::overnight::{OvernightReviewManager, OvernightStorageFactory};
use crate::image_preloader::{ImagePreloader, Priority, extract_image_url};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use bytes::Bytes;
use tracing::{info, warn, error};

// Represents the user's subscription tier
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SubscriptionTier {
    Free,
    Standard,
    Premium,
}

// Configuration for the event pipeline
#[derive(Debug)]
pub struct PipelineConfig {
    pub tier_routing: HashMap<SubscriptionTier, ProcessingLevel>,
    pub thinking_ai_config: ThinkingAIConfig,
    pub overnight_enabled: bool,
}

// Processing level for an event
#[derive(Debug, Clone, Copy)]
pub enum ProcessingLevel {
    Basic,    // Minimal processing
    Advanced, // Enhanced analysis
    Priority, // Real-time, high-priority processing
}

// A raw event from a sensor
#[derive(Serialize, Deserialize, Debug)]
pub struct RawEvent {
    pub event_id: Uuid,
    pub sensor_id: String,
    pub timestamp: i64,
    pub data: String, // e.g., base64 encoded image or sensor reading
    pub user_id: String,
    pub home_id: String, // Added home_id for thinking AI
    pub image_url: Option<String>, // Direct image URL for faster processing
    pub image_data: Option<Bytes>, // Pre-downloaded image data
}

// An event that has been processed by the pipeline
#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessedEvent {
    pub original_event_id: Uuid,
    pub processing_timestamp: i64,
    pub tier: SubscriptionTier,
    pub processing_level: String,
    pub vps_job_id: String,
    pub status: String, // e.g., "completed", "failed", "suppressed_for_overnight_review"
    pub result_summary: String,
    pub thinking_ai_analysis: Option<String>, // New field for thinking AI results
    pub overnight_suppressed: bool, // NEW: Indicates if event was suppressed for overnight review
}

// The main event pipeline
pub struct EventPipeline {
    config: PipelineConfig,
    vps_client: VpsApiClient,
    thinking_ai: ThinkingAIProcessor,
    llr_extractor: DemoLLRExtractor,
    overnight_manager: Option<Arc<OvernightReviewManager>>, // NEW: Overnight review manager
    image_preloader: Arc<ImagePreloader>, // NEW: Image preloader for faster processing
}

impl EventPipeline {
    pub fn new(config: PipelineConfig, vps_client: VpsApiClient) -> Self {
        let thinking_ai = ThinkingAIProcessor::new(config.thinking_ai_config.clone());
        let llr_extractor = DemoLLRExtractor::default();
        let image_preloader = Arc::new(ImagePreloader::new());
        
        // Initialize overnight system if enabled
        let overnight_manager = if config.overnight_enabled {
            let storage = OvernightStorageFactory::create_in_memory();
            let thinking_ai_arc = Arc::new(RwLock::new(thinking_ai.clone()));
            Some(Arc::new(OvernightReviewManager::new(storage, thinking_ai_arc)))
        } else {
            None
        };

        EventPipeline {
            config,
            vps_client,
            thinking_ai,
            llr_extractor,
            overnight_manager,
            image_preloader,
        }
    }

    // NEW: Constructor with custom overnight manager for testing
    pub fn with_overnight_manager(
        config: PipelineConfig, 
        vps_client: VpsApiClient,
        overnight_manager: Arc<OvernightReviewManager>
    ) -> Self {
        let thinking_ai = ThinkingAIProcessor::new(config.thinking_ai_config.clone());
        let llr_extractor = DemoLLRExtractor::default();
        let image_preloader = Arc::new(ImagePreloader::new());

        EventPipeline {
            config,
            vps_client,
            thinking_ai,
            llr_extractor,
            overnight_manager: Some(overnight_manager),
            image_preloader,
        }
    }

    // Determines processing level based on subscription tier
    fn get_processing_level(&self, tier: &SubscriptionTier) -> ProcessingLevel {
        *self.config.tier_routing.get(tier).unwrap_or(&ProcessingLevel::Basic)
    }

    // Convert RawEvent to thinking AI Event with placeholder LLR evidence
    fn create_thinking_event(&self, raw_event: &RawEvent) -> Event {
        // TODO: Replace with real LLR evidence extraction
        let evidence = self.llr_extractor.extract_evidence(raw_event);
        
        Event {
            ts: raw_event.timestamp as f64,
            cam: raw_event.sensor_id.clone(),
            person_track: format!("track_{}", raw_event.event_id.to_string()[..8].to_string()),
            rang_doorbell: false, // TODO: Extract from sensor data
            knocked: false,       // TODO: Extract from sensor data
            dwell_s: 15.0,       // TODO: Extract from sensor data
            away_prob: 0.1,      // TODO: Extract from context
            expected_window: false, // TODO: Extract from context
            token: None,         // TODO: Extract from context
            evidence,
        }
    }

    /// Process event with immediate image pre-loading
    pub async fn process_event_with_preload(&self, mut raw_event: RawEvent) -> Result<ProcessedEvent, PipelineError> {
        info!("Processing event {} with image preload", raw_event.event_id);
        
        // Step 1: Start image download immediately if URL present
        let image_download_task = if raw_event.image_data.is_none() {
            if let Some(image_url) = raw_event.image_url.as_ref().or_else(|| extract_image_url(&raw_event.data)) {
                info!("Starting async image download for: {}", image_url);
                Some(self.image_preloader.download_image_sync(
                    image_url.clone(), 
                    raw_event.event_id
                ))
            } else {
                None
            }
        } else {
            None
        };

        // Step 2: Continue with other processing while image downloads
        let tier = self.determine_tier(&raw_event.user_id).await?;
        let processing_level = self.get_processing_level(&tier);
        
        // Step 3: Wait for image download to complete
        if let Some(download_task) = image_download_task {
            match download_task.await {
                Ok(image_data) => {
                    info!("Image downloaded successfully ({} bytes)", image_data.len());
                    raw_event.image_data = Some(image_data);
                }
                Err(e) => {
                    warn!("Image download failed: {}, continuing without image", e);
                }
            }
        }

        // Step 4: Process with downloaded image data
        self.process_event_internal(raw_event, tier, processing_level).await
    }

    /// Preload image in background (fire and forget)
    pub fn preload_image_background(&self, url: String, event_id: Uuid) {
        self.image_preloader.preload_image(url, event_id, Priority::Normal);
    }

    /// Get image preloader statistics
    pub async fn get_image_cache_stats(&self) -> crate::image_preloader::CacheStats {
        self.image_preloader.get_cache_stats().await
    }

    // Placeholder method for extracting LLR evidence from raw event
    // TODO: Replace with actual implementation that connects to your LLR models
    fn extract_llr_evidence(&self, _raw_event: &RawEvent) -> Evidence {
        // Demo static values - replace with real LLR evidence extraction
        Evidence {
            llr_time: 0.0,
            llr_entry: -0.1,
            llr_behavior: 0.3,
            llr_identity: 0.2,
            llr_presence: 0.2,
            llr_token: 0.0,
        }
    }

    async fn determine_tier(&self, _user_id: &str) -> Result<SubscriptionTier, PipelineError> {
        // TODO: Implement actual tier lookup
        Ok(SubscriptionTier::Standard)
    }

    async fn process_event_internal(
        &self, 
        raw_event: RawEvent, 
        tier: SubscriptionTier, 
        processing_level: ProcessingLevel
    ) -> Result<ProcessedEvent, PipelineError> {
        // Create VPS processing request with image data
        let vps_request = VpsProcessingRequest {
            event_id: raw_event.event_id.to_string(),
            sensor_data: raw_event.data.clone(),
            image_data: raw_event.image_data.clone(),
            processing_level: format!("{:?}", processing_level),
            user_context: format!("user:{}, home:{}", raw_event.user_id, raw_event.home_id),
        };

        // Send to VPS for processing
        let vps_response = self.vps_client.process_event(vps_request).await
            .map_err(|e| PipelineError::VpsError(format!("VPS processing failed: {}", e)))?;

        // Create thinking AI event
        let thinking_event = self.create_thinking_event(&raw_event);
        
        // Process with thinking AI
        let thinking_result = self.thinking_ai.process_event(
            &raw_event.home_id,
            thinking_event
        ).await;

        let thinking_analysis = match thinking_result {
            Ok(analysis) => Some(analysis),
            Err(e) => {
                warn!("Thinking AI processing failed: {}", e);
                None
            }
        };

        Ok(ProcessedEvent {
            original_event_id: raw_event.event_id,
            processing_timestamp: chrono::Utc::now().timestamp(),
            tier,
            processing_level: format!("{:?}", processing_level),
            vps_job_id: vps_response.job_id,
            status: "completed".to_string(),
            result_summary: vps_response.summary,
            thinking_ai_analysis: thinking_analysis,
            overnight_suppressed: false,
        })
    }

    // UPDATED: Main event processing method with overnight integration
    pub async fn process_event(&mut self, event: RawEvent, tier: SubscriptionTier, api_key: &str) -> Result<ProcessedEvent, PipelineError> {
        // Check if event is during overnight review period
        if let Some(overnight_mgr) = &self.overnight_manager {
            let event_time = DateTime::from_timestamp(event.timestamp, 0).unwrap_or_else(|| Utc::now());
            
            if overnight_mgr.is_in_review_period(&event.home_id, event_time).await
                .map_err(|e| PipelineError::OvernightError(e.to_string()))? 
            {
                // Process for overnight review (analyze but don't alert)
                let analysis = overnight_mgr.process_for_overnight_review(&event).await
                    .map_err(|e| PipelineError::OvernightError(e.to_string()))?;
                
                // Store for morning summary
                overnight_mgr.store_overnight_event(analysis).await
                    .map_err(|e| PipelineError::OvernightError(e.to_string()))?;

                // Return suppressed event response
                return Ok(ProcessedEvent {
                    original_event_id: event.event_id,
                    processing_timestamp: Utc::now().timestamp(),
                    tier,
                    processing_level: "overnight_suppressed".to_string(),
                    vps_job_id: "overnight".to_string(),
                    status: "suppressed_for_overnight_review".to_string(),
                    result_summary: "Event processed and stored for morning review".to_string(),
                    thinking_ai_analysis: None,
                    overnight_suppressed: true,
                });
            }
        }

        // Continue with normal pipeline processing if not in overnight period
        let processing_level = self.get_processing_level(&tier);

        // Process with VPS API
        let request = VpsProcessingRequest {
            api_key,
            event_id: &event.event_id.to_string(),
            sensor_data: &event.data,
            processing_level: &format!("{:?}", processing_level).to_lowercase(),
        };

        let vps_response = self.vps_client.submit_event_for_processing(&request).await
            .map_err(|e| PipelineError::VpsSubmissionError(format!("{}", e).into()))?;

        // Process with Thinking AI for Premium tier
        let thinking_ai_analysis = if matches!(tier, SubscriptionTier::Premium) {
            let thinking_event = self.create_thinking_event(&event);
            
            if let Some(result) = self.thinking_ai.process_event(&event.home_id, thinking_event) {
                Some(self.thinking_ai.format_thinking_block(&result))
            } else {
                None
            }
        } else {
            None
        };

        let mut result_summary = "Processing initiated with VPS".to_string();
        if thinking_ai_analysis.is_some() {
            result_summary.push_str(" + ThinkingAI analysis");
        }

        Ok(ProcessedEvent {
            original_event_id: event.event_id,
            processing_timestamp: Utc::now().timestamp(),
            tier,
            processing_level: request.processing_level.to_string(),
            vps_job_id: vps_response.job_id,
            status: vps_response.status,
            result_summary,
            thinking_ai_analysis,
            overnight_suppressed: false,
        })
    }

    // NEW: Generate morning summary for a home
    pub async fn generate_morning_summary(&self, home_id: &str) -> Result<Option<crate::overnight::MorningSummary>, PipelineError> {
        if let Some(overnight_mgr) = &self.overnight_manager {
            overnight_mgr.generate_morning_summary(home_id).await
                .map(Some)
                .map_err(|e| PipelineError::OvernightError(e.to_string()))
        } else {
            Ok(None)
        }
    }

    // NEW: Update overnight configuration for a home
    pub async fn update_overnight_config(&self, config: crate::overnight::OvernightConfig) -> Result<(), PipelineError> {
        if let Some(overnight_mgr) = &self.overnight_manager {
            overnight_mgr.update_config(config).await
                .map_err(|e| PipelineError::OvernightError(e.to_string()))
        } else {
            Err(PipelineError::OvernightError("Overnight system not enabled".to_string()))
        }
    }

    // NEW: Get overnight configuration for a home
    pub async fn get_overnight_config(&self, home_id: &str) -> Option<crate::overnight::OvernightConfig> {
        if let Some(overnight_mgr) = &self.overnight_manager {
            overnight_mgr.get_config(home_id).await
        } else {
            None
        }
    }
}

// UPDATED: Custom errors for the pipeline
#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Failed to submit event to VPS: {0}")]
    VpsSubmissionError(Box<dyn std::error::Error + Send + Sync>),

    #[error("Overnight review system error: {0}")]
    OvernightError(String), // NEW: Overnight system errors

    #[error("An unknown pipeline error occurred")]
    Unknown,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        let mut tier_routing = HashMap::new();
        tier_routing.insert(SubscriptionTier::Free, ProcessingLevel::Basic);
        tier_routing.insert(SubscriptionTier::Standard, ProcessingLevel::Advanced);
        tier_routing.insert(SubscriptionTier::Premium, ProcessingLevel::Priority);

        Self {
            tier_routing,
            thinking_ai_config: ThinkingAIConfig::default(),
            overnight_enabled: true, // NEW: Default to enabled
        }
    }
}

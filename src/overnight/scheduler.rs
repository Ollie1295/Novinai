//! Overnight Summary Scheduler System
//!
//! Time-based scheduling system that triggers morning summary generation
//! and delivery at configured times for each home.

use super::*;
use crate::overnight::{OvernightReviewManager, OvernightConfig, DeliveryChannel};
use crate::overnight::summary::{OvernightSummaryGenerator, MorningSummaryWithDelivery};
use chrono::{DateTime, Utc, NaiveTime, Timelike, Local};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, Duration};
use anyhow::{Result, anyhow};
use uuid::Uuid;

/// Scheduler for overnight summary generation and delivery
#[derive(Debug)]
pub struct OvernightScheduler {
    /// Reference to the overnight manager
    overnight_manager: Arc<OvernightReviewManager>,
    /// Summary generator instance
    summary_generator: Arc<OvernightSummaryGenerator>,
    /// Delivery system
    delivery_system: Arc<DeliverySystem>,
    /// Scheduler state
    state: Arc<RwLock<SchedulerState>>,
    /// Channel for shutdown signaling
    shutdown_tx: Option<mpsc::Sender<()>>,
}

/// Internal state of the scheduler
#[derive(Debug, Default)]
struct SchedulerState {
    /// Currently scheduled summaries by home_id
    scheduled_summaries: HashMap<String, ScheduledSummary>,
    /// Last check time for delivery scheduling
    last_check: Option<DateTime<Utc>>,
    /// Statistics
    stats: SchedulerStats,
}

/// A scheduled summary for a specific home
#[derive(Debug, Clone)]
struct ScheduledSummary {
    pub home_id: String,
    pub delivery_time: NaiveTime,
    pub channels: Vec<DeliveryChannel>,
    pub timezone: String,
    pub next_delivery: DateTime<Utc>,
    pub last_delivered: Option<DateTime<Utc>>,
}

/// Statistics about scheduler operations
#[derive(Debug, Default, Clone)]
pub struct SchedulerStats {
    pub summaries_generated: u64,
    pub summaries_delivered: u64,
    pub delivery_failures: u64,
    pub active_schedules: usize,
    pub last_check_time: Option<DateTime<Utc>>,
}

impl OvernightScheduler {
    /// Create a new overnight scheduler
    pub fn new(
        overnight_manager: Arc<OvernightReviewManager>,
        summary_generator: Arc<OvernightSummaryGenerator>,
    ) -> Self {
        let delivery_system = Arc::new(DeliverySystem::new());
        
        Self {
            overnight_manager,
            summary_generator,
            delivery_system,
            state: Arc::new(RwLock::new(SchedulerState::default())),
            shutdown_tx: None,
        }
    }

    /// Start the scheduler with specified check interval
    pub async fn start(&mut self, check_interval_minutes: u64) -> Result<()> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let overnight_manager = Arc::clone(&self.overnight_manager);
        let summary_generator = Arc::clone(&self.summary_generator);
        let delivery_system = Arc::clone(&self.delivery_system);
        let state = Arc::clone(&self.state);

        // Spawn the main scheduler task
        tokio::spawn(async move {
            let mut check_interval = interval(Duration::from_secs(check_interval_minutes * 60));
            
            loop {
                tokio::select! {
                    _ = check_interval.tick() => {
                        if let Err(e) = Self::check_and_process_deliveries(
                            &overnight_manager,
                            &summary_generator,
                            &delivery_system,
                            &state,
                        ).await {
                            eprintln!("Scheduler error: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        println!("Overnight scheduler shutting down");
                        break;
                    }
                }
            }
        });

        println!("Overnight scheduler started with {}-minute check interval", check_interval_minutes);
        Ok(())
    }

    /// Stop the scheduler
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            tx.send(()).await.map_err(|_| anyhow!("Failed to send shutdown signal"))?;
        }
        Ok(())
    }

    /// Schedule summary delivery for a home
    pub async fn schedule_home(&self, config: &OvernightConfig) -> Result<()> {
        let mut state = self.state.write().await;
        
        // Calculate next delivery time
        let next_delivery = self.calculate_next_delivery_time(
            config.summary_delivery_time,
            &config.timezone
        )?;
        
        let scheduled_summary = ScheduledSummary {
            home_id: config.home_id.clone(),
            delivery_time: config.summary_delivery_time,
            channels: config.delivery_channels.clone(),
            timezone: config.timezone.clone(),
            next_delivery,
            last_delivered: None,
        };
        
        state.scheduled_summaries.insert(config.home_id.clone(), scheduled_summary);
        state.stats.active_schedules = state.scheduled_summaries.len();
        
        println!("Scheduled summary delivery for home {} at {}", 
                config.home_id, config.summary_delivery_time);
        
        Ok(())
    }

    /// Remove scheduled delivery for a home
    pub async fn unschedule_home(&self, home_id: &str) -> Result<()> {
        let mut state = self.state.write().await;
        
        if state.scheduled_summaries.remove(home_id).is_some() {
            state.stats.active_schedules = state.scheduled_summaries.len();
            println!("Removed scheduled delivery for home {}", home_id);
            Ok(())
        } else {
            Err(anyhow!("No scheduled delivery found for home {}", home_id))
        }
    }

    /// Update schedule for a home
    pub async fn update_schedule(&self, config: &OvernightConfig) -> Result<()> {
        // Remove existing schedule
        let _ = self.unschedule_home(&config.home_id).await;
        
        // Add new schedule
        self.schedule_home(config).await
    }

    /// Get current scheduler statistics
    pub async fn get_stats(&self) -> SchedulerStats {
        self.state.read().await.stats.clone()
    }

    /// Get all scheduled summaries
    pub async fn get_scheduled_summaries(&self) -> Vec<ScheduledSummary> {
        self.state.read().await.scheduled_summaries.values().cloned().collect()
    }

    /// Force delivery for a specific home (for testing)
    pub async fn force_delivery(&self, home_id: &str) -> Result<MorningSummaryWithDelivery> {
        let scheduled_summary = {
            let state = self.state.read().await;
            state.scheduled_summaries.get(home_id).cloned()
                .ok_or_else(|| anyhow!("No scheduled delivery for home {}", home_id))?
        };
        
        self.process_single_delivery(&scheduled_summary).await
    }

    /// Main check and process deliveries logic
    async fn check_and_process_deliveries(
        overnight_manager: &Arc<OvernightReviewManager>,
        summary_generator: &Arc<OvernightSummaryGenerator>,
        delivery_system: &Arc<DeliverySystem>,
        state: &Arc<RwLock<SchedulerState>>,
    ) -> Result<()> {
        let now = Utc::now();
        let deliveries_to_process = {
            let mut state_lock = state.write().await;
            state_lock.last_check = Some(now);
            state_lock.stats.last_check_time = Some(now);
            
            // Find summaries that need to be delivered
            state_lock.scheduled_summaries.values()
                .filter(|summary| now >= summary.next_delivery)
                .cloned()
                .collect::<Vec<_>>()
        };

        if deliveries_to_process.is_empty() {
            return Ok(());
        }

        println!("Processing {} scheduled deliveries", deliveries_to_process.len());

        // Process each delivery
        for scheduled_summary in deliveries_to_process {
            match Self::process_delivery_with_manager(
                &scheduled_summary,
                overnight_manager,
                summary_generator,
                delivery_system,
            ).await {
                Ok(summary) => {
                    // Update state with successful delivery
                    let mut state_lock = state.write().await;
                    if let Some(existing) = state_lock.scheduled_summaries.get_mut(&scheduled_summary.home_id) {
                        existing.last_delivered = Some(now);
                        existing.next_delivery = Self::calculate_next_delivery_static(
                            existing.delivery_time,
                            &existing.timezone,
                        ).unwrap_or_else(|_| now + chrono::Duration::days(1));
                    }
                    state_lock.stats.summaries_generated += 1;
                    state_lock.stats.summaries_delivered += 1;
                    
                    println!("Successfully delivered summary for home {}: {} events", 
                            scheduled_summary.home_id, summary.summary.event_count);
                }
                Err(e) => {
                    eprintln!("Failed to deliver summary for home {}: {}", 
                             scheduled_summary.home_id, e);
                    
                    let mut state_lock = state.write().await;
                    state_lock.stats.delivery_failures += 1;
                }
            }
        }

        Ok(())
    }

    /// Process delivery using the manager components
    async fn process_delivery_with_manager(
        scheduled_summary: &ScheduledSummary,
        overnight_manager: &Arc<OvernightReviewManager>,
        summary_generator: &Arc<OvernightSummaryGenerator>,
        delivery_system: &Arc<DeliverySystem>,
    ) -> Result<MorningSummaryWithDelivery> {
        // Generate summary using the manager
        let base_summary = overnight_manager.generate_morning_summary(&scheduled_summary.home_id).await?;
        
        // Generate delivery-specific content
        let events = vec![]; // Events are already processed by the manager
        let summary_with_delivery = summary_generator
            .generate_summary_with_delivery(events, &scheduled_summary.channels)
            .await?;
        
        // Deliver the summary
        delivery_system.deliver_summary(&summary_with_delivery).await?;
        
        Ok(summary_with_delivery)
    }

    /// Process a single delivery
    async fn process_single_delivery(&self, scheduled_summary: &ScheduledSummary) -> Result<MorningSummaryWithDelivery> {
        // Generate summary
        let base_summary = self.overnight_manager.generate_morning_summary(&scheduled_summary.home_id).await?;
        
        // Generate delivery content
        let events = vec![]; // Events already processed
        let summary_with_delivery = self.summary_generator
            .generate_summary_with_delivery(events, &scheduled_summary.channels)
            .await?;
        
        // Deliver
        self.delivery_system.deliver_summary(&summary_with_delivery).await?;
        
        // Update stats
        let mut state = self.state.write().await;
        state.stats.summaries_generated += 1;
        state.stats.summaries_delivered += 1;
        
        Ok(summary_with_delivery)
    }

    /// Calculate the next delivery time for a given schedule
    fn calculate_next_delivery_time(&self, delivery_time: NaiveTime, timezone: &str) -> Result<DateTime<Utc>> {
        Self::calculate_next_delivery_static(delivery_time, timezone)
    }

    /// Static version of next delivery calculation
    fn calculate_next_delivery_static(delivery_time: NaiveTime, timezone: &str) -> Result<DateTime<Utc>> {
        let tz = chrono_tz::Tz::from_str(timezone)
            .map_err(|_| anyhow!("Invalid timezone: {}", timezone))?;
        
        let now = Utc::now().with_timezone(&tz);
        let today = now.date_naive();
        let tomorrow = today + chrono::Duration::days(1);
        
        // Check if we should deliver today or tomorrow
        let delivery_date = if now.time() < delivery_time {
            today // Delivery time hasn't passed today
        } else {
            tomorrow // Delivery time has passed, schedule for tomorrow
        };
        
        let local_delivery = delivery_date.and_time(delivery_time)
            .and_local_timezone(tz)
            .single()
            .ok_or_else(|| anyhow!("Invalid delivery time in timezone"))?;
        
        Ok(local_delivery.with_timezone(&Utc))
    }
}

/// System for delivering morning summaries through various channels
#[derive(Debug)]
pub struct DeliverySystem {
    /// Delivery statistics
    stats: Arc<RwLock<DeliveryStats>>,
}

/// Statistics about delivery operations
#[derive(Debug, Default)]
struct DeliveryStats {
    push_notifications: u64,
    emails: u64,
    sms_messages: u64,
    websocket_messages: u64,
    dashboard_updates: u64,
    failures: u64,
}

impl DeliverySystem {
    /// Create a new delivery system
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(DeliveryStats::default())),
        }
    }

    /// Deliver a summary through configured channels
    pub async fn deliver_summary(&self, summary: &MorningSummaryWithDelivery) -> Result<()> {
        let mut delivery_results = Vec::new();

        // Deliver through each configured channel
        for channel in &summary.channels {
            let result = match channel {
                DeliveryChannel::Push => self.deliver_push_notification(summary).await,
                DeliveryChannel::Email => self.deliver_email(summary).await,
                DeliveryChannel::SMS => self.deliver_sms(summary).await,
                DeliveryChannel::WebSocket => self.deliver_websocket(summary).await,
                DeliveryChannel::Dashboard => self.deliver_dashboard(summary).await,
            };
            
            delivery_results.push((channel, result));
        }

        // Update statistics
        let mut stats = self.stats.write().await;
        for (channel, result) in &delivery_results {
            match result {
                Ok(_) => {
                    match channel {
                        DeliveryChannel::Push => stats.push_notifications += 1,
                        DeliveryChannel::Email => stats.emails += 1,
                        DeliveryChannel::SMS => stats.sms_messages += 1,
                        DeliveryChannel::WebSocket => stats.websocket_messages += 1,
                        DeliveryChannel::Dashboard => stats.dashboard_updates += 1,
                    }
                }
                Err(_) => stats.failures += 1,
            }
        }

        // Check if any deliveries succeeded
        let successful_deliveries = delivery_results.iter().filter(|(_, r)| r.is_ok()).count();
        if successful_deliveries == 0 {
            return Err(anyhow!("All delivery channels failed"));
        }

        println!("Successfully delivered summary via {}/{} channels", 
                successful_deliveries, delivery_results.len());
        Ok(())
    }

    /// Deliver push notification (stub implementation)
    async fn deliver_push_notification(&self, summary: &MorningSummaryWithDelivery) -> Result<()> {
        if let Some(push_content) = &summary.delivery_content.push_notification {
            // TODO: Integrate with real push notification service (FCM, APNS, etc.)
            println!("PUSH: {}", push_content);
            Ok(())
        } else {
            Err(anyhow!("No push notification content available"))
        }
    }

    /// Deliver email (stub implementation)
    async fn deliver_email(&self, summary: &MorningSummaryWithDelivery) -> Result<()> {
        if let Some(email_content) = &summary.delivery_content.email {
            // TODO: Integrate with email service (SendGrid, AWS SES, etc.)
            println!("EMAIL: {}", email_content);
            Ok(())
        } else {
            Err(anyhow!("No email content available"))
        }
    }

    /// Deliver SMS (stub implementation)
    async fn deliver_sms(&self, summary: &MorningSummaryWithDelivery) -> Result<()> {
        if let Some(sms_content) = &summary.delivery_content.sms {
            // TODO: Integrate with SMS service (Twilio, AWS SNS, etc.)
            println!("SMS: {}", sms_content);
            Ok(())
        } else {
            Err(anyhow!("No SMS content available"))
        }
    }

    /// Deliver WebSocket message (stub implementation)
    async fn deliver_websocket(&self, summary: &MorningSummaryWithDelivery) -> Result<()> {
        if let Some(ws_content) = &summary.delivery_content.websocket {
            // TODO: Integrate with WebSocket connections
            println!("WEBSOCKET: {}", ws_content);
            Ok(())
        } else {
            Err(anyhow!("No WebSocket content available"))
        }
    }

    /// Update dashboard (stub implementation)
    async fn deliver_dashboard(&self, summary: &MorningSummaryWithDelivery) -> Result<()> {
        if let Some(dashboard_content) = &summary.delivery_content.dashboard {
            // TODO: Update dashboard/database with summary
            println!("DASHBOARD: {}", dashboard_content);
            Ok(())
        } else {
            Err(anyhow!("No dashboard content available"))
        }
    }

    /// Get delivery statistics
    pub async fn get_stats(&self) -> DeliveryStats {
        self.stats.read().await.clone()
    }
}

impl Clone for DeliveryStats {
    fn clone(&self) -> Self {
        Self {
            push_notifications: self.push_notifications,
            emails: self.emails,
            sms_messages: self.sms_messages,
            websocket_messages: self.websocket_messages,
            dashboard_updates: self.dashboard_updates,
            failures: self.failures,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thinking::{ThinkingAIProcessor, ThinkingAIConfig};
    use crate::overnight::storage::OvernightStorageFactory;
    use chrono::NaiveTime;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let storage = OvernightStorageFactory::create_in_memory();
        let thinking_ai = Arc::new(RwLock::new(ThinkingAIProcessor::new(ThinkingAIConfig::default())));
        let overnight_manager = Arc::new(OvernightReviewManager::new(storage, thinking_ai.clone()));
        let summary_generator = Arc::new(OvernightSummaryGenerator::new(thinking_ai));
        
        let scheduler = OvernightScheduler::new(overnight_manager, summary_generator);
        
        let stats = scheduler.get_stats().await;
        assert_eq!(stats.active_schedules, 0);
        assert_eq!(stats.summaries_generated, 0);
    }

    #[tokio::test]
    async fn test_schedule_management() {
        let storage = OvernightStorageFactory::create_in_memory();
        let thinking_ai = Arc::new(RwLock::new(ThinkingAIProcessor::new(ThinkingAIConfig::default())));
        let overnight_manager = Arc::new(OvernightReviewManager::new(storage, thinking_ai.clone()));
        let summary_generator = Arc::new(OvernightSummaryGenerator::new(thinking_ai));
        
        let scheduler = OvernightScheduler::new(overnight_manager, summary_generator);
        
        let config = OvernightConfig {
            home_id: "test-home".to_string(),
            review_start_time: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            review_end_time: NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            summary_delivery_time: NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            timezone: "UTC".to_string(),
            enabled: true,
            delivery_channels: vec![DeliveryChannel::Push],
        };
        
        // Schedule home
        scheduler.schedule_home(&config).await.unwrap();
        
        let stats = scheduler.get_stats().await;
        assert_eq!(stats.active_schedules, 1);
        
        let schedules = scheduler.get_scheduled_summaries().await;
        assert_eq!(schedules.len(), 1);
        assert_eq!(schedules[0].home_id, "test-home");
        
        // Unschedule home
        scheduler.unschedule_home("test-home").await.unwrap();
        
        let stats = scheduler.get_stats().await;
        assert_eq!(stats.active_schedules, 0);
    }

    #[test]
    fn test_next_delivery_calculation() {
        let delivery_time = NaiveTime::from_hms_opt(7, 30, 0).unwrap();
        let timezone = "UTC";
        
        let next_delivery = OvernightScheduler::calculate_next_delivery_static(
            delivery_time, 
            timezone
        ).unwrap();
        
        // Should be either today or tomorrow at 7:30 AM
        let expected_time = next_delivery.time();
        assert_eq!(expected_time.hour(), 7);
        assert_eq!(expected_time.minute(), 30);
    }
}

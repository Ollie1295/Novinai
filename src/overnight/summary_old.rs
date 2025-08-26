//! Overnight Summary Generation System
//!
//! AI-powered system that processes overnight events and generates 
//! intelligent, contextual morning summaries using the ThinkingAI system.

use super::*;
use crate::overnight::manager::{OvernightEventAnalysis, MorningSummary};
use crate::thinking::{ThinkingAIProcessor, AlertDecision};
use chrono::{DateTime, Utc, Timelike};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, anyhow};

/// AI-powered summary generator for overnight events
#[derive(Debug)]
pub struct OvernightSummaryGenerator {
    thinking_ai: Arc<RwLock<ThinkingAIProcessor>>,
}

impl OvernightSummaryGenerator {
    /// Create a new summary generator
    pub fn new(thinking_ai: Arc<RwLock<ThinkingAIProcessor>>) -> Self {
        Self { thinking_ai }
    }

    /// Generate a comprehensive morning summary from overnight events
    pub async fn generate_summary(&self, events: Vec<OvernightEventAnalysis>) -> Result<MorningSummary> {
        if events.is_empty() {
            return Ok(self.create_quiet_night_summary());
        }

        // Analyze event patterns and generate narrative
        let analysis = self.analyze_event_patterns(&events).await?;
        let narrative = self.generate_narrative(&events, &analysis).await?;
        let high_priority_events = self.extract_high_priority_events(&events);
        
        Ok(MorningSummary {
            home_id: events[0].home_id.clone(),
            summary_date: Utc::now().date_naive(),
            event_count: events.len(),
            narrative,
            high_priority_events,
            requires_attention: !high_priority_events.is_empty(),
        })
    }

    /// Generate morning summary with delivery preferences
    pub async fn generate_summary_with_delivery(
        &self, 
        events: Vec<OvernightEventAnalysis>,
        delivery_channels: &[DeliveryChannel]
    ) -> Result<MorningSummaryWithDelivery> {
        let base_summary = self.generate_summary(events).await?;
        
        // Customize content based on delivery channels
        let customized_content = self.customize_for_channels(&base_summary, delivery_channels).await?;
        
        Ok(MorningSummaryWithDelivery {
            summary: base_summary,
            delivery_content: customized_content,
            channels: delivery_channels.to_vec(),
        })
    }

    /// Analyze patterns in overnight events to inform summary generation
    async fn analyze_event_patterns(&self, events: &[OvernightEventAnalysis]) -> Result<EventPatternAnalysis> {
        let mut pattern_analysis = EventPatternAnalysis::default();
        
        // Time-based analysis
        pattern_analysis.temporal_distribution = self.analyze_temporal_patterns(events);
        
        // Sensor-based analysis
        pattern_analysis.sensor_distribution = self.analyze_sensor_patterns(events);
        
        // Threat level analysis
        pattern_analysis.threat_analysis = self.analyze_threat_levels(events);
        
        // Activity clustering
        pattern_analysis.activity_clusters = self.cluster_activities(events);
        
        Ok(pattern_analysis)
    }

    /// Generate contextual narrative based on events and patterns
    async fn generate_narrative(&self, events: &[OvernightEventAnalysis], analysis: &EventPatternAnalysis) -> Result<String> {
        let high_priority_count = events.iter().filter(|e| e.requires_morning_attention).count();
        let total_count = events.len();
        
        // Determine overall tone based on event analysis
        let tone = self.determine_summary_tone(analysis, high_priority_count, total_count);
        
        // Generate contextual opening
        let opening = match tone {
            SummaryTone::Quiet => self.generate_quiet_opening(total_count),
            SummaryTone::Active => self.generate_active_opening(total_count),
            SummaryTone::Concerning => self.generate_concerning_opening(total_count, high_priority_count),
            SummaryTone::Busy => self.generate_busy_opening(total_count),
        };

        // Add temporal context if significant
        let temporal_context = self.generate_temporal_context(&analysis.temporal_distribution);
        
        // Add sensor context if relevant
        let sensor_context = self.generate_sensor_context(&analysis.sensor_distribution);
        
        // Add specific event highlights
        let event_highlights = self.generate_event_highlights(events, high_priority_count);
        
        // Combine into cohesive narrative
        let narrative = format!(
            "{}{}{}{}",
            opening,
            temporal_context,
            sensor_context,
            event_highlights
        );
        
        Ok(narrative.trim().to_string())
    }

    /// Extract and rank high priority events that require attention
    fn extract_high_priority_events(&self, events: &[OvernightEventAnalysis]) -> Vec<OvernightEventAnalysis> {
        let mut high_priority: Vec<_> = events.iter()
            .filter(|e| e.requires_morning_attention)
            .cloned()
            .collect();
        
        // Sort by timestamp (most recent first)
        high_priority.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        // Limit to top 5 most important events
        high_priority.into_iter().take(5).collect()
    }

    /// Analyze temporal distribution of events
    fn analyze_temporal_patterns(&self, events: &[OvernightEventAnalysis]) -> TemporalDistribution {
        let mut hourly_counts = vec![0usize; 24];
        let mut peak_hour = 0;
        let mut peak_count = 0;
        
        for event in events {
            let hour = event.timestamp.hour() as usize;
            hourly_counts[hour] += 1;
            
            if hourly_counts[hour] > peak_count {
                peak_count = hourly_counts[hour];
                peak_hour = hour;
            }
        }
        
        TemporalDistribution {
            hourly_distribution: hourly_counts,
            peak_hour,
            peak_activity_count: peak_count,
            is_clustered: peak_count > events.len() / 3, // More than 1/3 of events in peak hour
        }
    }

    /// Analyze sensor distribution patterns
    fn analyze_sensor_patterns(&self, events: &[OvernightEventAnalysis]) -> SensorDistribution {
        let mut sensor_counts = std::collections::HashMap::new();
        
        for event in events {
            *sensor_counts.entry(event.sensor_id.clone()).or_insert(0) += 1;
        }
        
        let most_active_sensor = sensor_counts.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(sensor, count)| (sensor.clone(), *count));
        
        SensorDistribution {
            sensor_counts,
            most_active_sensor,
            unique_sensors: sensor_counts.len(),
        }
    }

    /// Analyze threat levels across events
    fn analyze_threat_levels(&self, events: &[OvernightEventAnalysis]) -> ThreatAnalysis {
        let mut alert_count = 0;
        let mut suppress_count = 0;
        let mut ignore_count = 0;
        
        for event in events {
            match event.threat_level {
                AlertDecision::Alert(_) => alert_count += 1,
                AlertDecision::Suppress => suppress_count += 1,
                AlertDecision::Ignore => ignore_count += 1,
            }
        }
        
        ThreatAnalysis {
            alert_events: alert_count,
            suppressed_events: suppress_count,
            ignored_events: ignore_count,
            max_threat_level: if alert_count > 0 { ThreatLevel::High }
                           else if suppress_count > 0 { ThreatLevel::Medium }
                           else { ThreatLevel::Low },
        }
    }

    /// Cluster activities by proximity in time
    fn cluster_activities(&self, events: &[OvernightEventAnalysis]) -> Vec<ActivityCluster> {
        if events.is_empty() {
            return vec![];
        }
        
        let mut sorted_events = events.to_vec();
        sorted_events.sort_by_key(|e| e.timestamp);
        
        let mut clusters = vec![];
        let mut current_cluster = vec![sorted_events[0].clone()];
        
        for event in sorted_events.iter().skip(1) {
            let last_event_time = current_cluster.last().unwrap().timestamp;
            let time_diff = event.timestamp - last_event_time;
            
            if time_diff.num_minutes() <= 15 { // Events within 15 minutes are clustered
                current_cluster.push(event.clone());
            } else {
                // Finish current cluster and start new one
                if current_cluster.len() > 1 {
                    clusters.push(ActivityCluster {
                        start_time: current_cluster.first().unwrap().timestamp,
                        end_time: current_cluster.last().unwrap().timestamp,
                        event_count: current_cluster.len(),
                        sensors_involved: current_cluster.iter()
                            .map(|e| e.sensor_id.clone())
                            .collect::<std::collections::HashSet<_>>()
                            .into_iter().collect(),
                    });
                }
                current_cluster = vec![event.clone()];
            }
        }
        
        // Add final cluster if it has multiple events
        if current_cluster.len() > 1 {
            clusters.push(ActivityCluster {
                start_time: current_cluster.first().unwrap().timestamp,
                end_time: current_cluster.last().unwrap().timestamp,
                event_count: current_cluster.len(),
                sensors_involved: current_cluster.iter()
                    .map(|e| e.sensor_id.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter().collect(),
            });
        }
        
        clusters
    }

    /// Determine appropriate tone based on event analysis
    fn determine_summary_tone(&self, analysis: &EventPatternAnalysis, high_priority: usize, total: usize) -> SummaryTone {
        if high_priority > 0 && high_priority as f32 / total as f32 > 0.3 {
            SummaryTone::Concerning
        } else if total > 10 || analysis.activity_clusters.len() > 2 {
            SummaryTone::Busy
        } else if total > 3 || high_priority > 0 {
            SummaryTone::Active
        } else {
            SummaryTone::Quiet
        }
    }

    /// Generate narrative openings for different tones
    fn generate_quiet_opening(&self, total: usize) -> String {
        if total == 0 {
            "Good morning! It was a quiet night with no security events detected.".to_string()
        } else {
            format!("Good morning! It was a peaceful night with only {} minor event{} detected.", 
                   total, if total == 1 { "" } else { "s" })
        }
    }

    fn generate_active_opening(&self, total: usize) -> String {
        format!("Good morning! There was some activity overnight with {} event{} recorded.", 
               total, if total == 1 { "" } else { "s" })
    }

    fn generate_concerning_opening(&self, total: usize, high_priority: usize) -> String {
        format!("Good morning! {} event{} occurred overnight, including {} that may require your attention.", 
               total, if total == 1 { "" } else { "s" }, high_priority)
    }

    fn generate_busy_opening(&self, total: usize) -> String {
        format!("Good morning! It was an active night with {} events detected across your security system.", total)
    }

    /// Generate temporal context narrative
    fn generate_temporal_context(&self, temporal: &TemporalDistribution) -> String {
        if temporal.is_clustered {
            let hour_12 = if temporal.peak_hour == 0 { 12 }
                         else if temporal.peak_hour > 12 { temporal.peak_hour - 12 }
                         else { temporal.peak_hour };
            let am_pm = if temporal.peak_hour < 12 { "AM" } else { "PM" };
            
            format!(" Most activity occurred around {}:00 {}.", hour_12, am_pm)
        } else {
            " Activity was distributed throughout the night.".to_string()
        }
    }

    /// Generate sensor context narrative
    fn generate_sensor_context(&self, sensors: &SensorDistribution) -> String {
        if let Some((sensor_name, count)) = &sensors.most_active_sensor {
            if sensors.unique_sensors == 1 {
                format!(" All events were detected by {}.", self.friendly_sensor_name(sensor_name))
            } else if *count as f32 / sensors.sensor_counts.values().sum::<usize>() as f32 > 0.6 {
                format!(" Most activity was concentrated around {}.", self.friendly_sensor_name(sensor_name))
            } else {
                format!(" Events were detected across {} different sensors.", sensors.unique_sensors)
            }
        } else {
            String::new()
        }
    }

    /// Generate event highlights
    fn generate_event_highlights(&self, _events: &[OvernightEventAnalysis], high_priority: usize) -> String {
        if high_priority > 0 {
            format!(" {} event{} have been flagged for your review.", 
                   high_priority, if high_priority == 1 { "" } else { "s" })
        } else {
            " All events appear routine with no immediate concerns.".to_string()
        }
    }

    /// Convert technical sensor names to friendly names
    fn friendly_sensor_name(&self, sensor_id: &str) -> String {
        if sensor_id.contains("camera") {
            format!("the {} camera", sensor_id.replace("camera", "").replace("_", " ").trim())
        } else if sensor_id.contains("motion") {
            format!("the {} motion detector", sensor_id.replace("motion", "").replace("_", " ").trim())
        } else if sensor_id.contains("door") {
            format!("the {} door sensor", sensor_id.replace("door", "").replace("_", " ").trim())
        } else {
            sensor_id.replace("_", " ")
        }
    }

    /// Customize summary content for different delivery channels
    async fn customize_for_channels(&self, summary: &MorningSummary, channels: &[DeliveryChannel]) -> Result<DeliveryContent> {
        let mut content = DeliveryContent::default();
        
        for channel in channels {
            match channel {
                DeliveryChannel::Push => {
                    content.push_notification = Some(self.generate_push_content(summary));
                }
                DeliveryChannel::Email => {
                    content.email = Some(self.generate_email_content(summary));
                }
                DeliveryChannel::SMS => {
                    content.sms = Some(self.generate_sms_content(summary));
                }
                DeliveryChannel::WebSocket => {
                    content.websocket = Some(self.generate_websocket_content(summary));
                }
                DeliveryChannel::Dashboard => {
                    content.dashboard = Some(self.generate_dashboard_content(summary));
                }
            }
        }
        
        Ok(content)
    }

    /// Generate content optimized for push notifications (short)
    fn generate_push_content(&self, summary: &MorningSummary) -> String {
        if summary.requires_attention {
            format!("Security Summary: {} events overnight, {} need attention", 
                   summary.event_count, summary.high_priority_events.len())
        } else if summary.event_count > 0 {
            format!("Security Summary: {} routine events overnight", summary.event_count)
        } else {
            "Security Summary: Quiet night, no events".to_string()
        }
    }

    /// Generate content for email delivery (detailed)
    fn generate_email_content(&self, summary: &MorningSummary) -> String {
        let mut content = format!("Subject: Overnight Security Summary - {}\n\n", 
                                 summary.summary_date.format("%B %d, %Y"));
        content.push_str(&summary.narrative);
        
        if !summary.high_priority_events.is_empty() {
            content.push_str("\n\nEvents Requiring Attention:\n");
            for (i, event) in summary.high_priority_events.iter().enumerate() {
                content.push_str(&format!("{}. {} at {} - {}\n", 
                                         i + 1,
                                         event.sensor_id,
                                         event.timestamp.format("%I:%M %p"),
                                         event.analysis.lines().next().unwrap_or("Analysis pending")));
            }
        }
        
        content
    }

    /// Generate content for SMS delivery (very short)
    fn generate_sms_content(&self, summary: &MorningSummary) -> String {
        if summary.requires_attention {
            format!("Security: {} events, {} priority. Check app for details.", 
                   summary.event_count, summary.high_priority_events.len())
        } else {
            format!("Security: {} routine events overnight. All clear.", summary.event_count)
        }
    }

    /// Generate content for WebSocket delivery (structured)
    fn generate_websocket_content(&self, summary: &MorningSummary) -> String {
        serde_json::to_string(summary).unwrap_or_else(|_| summary.narrative.clone())
    }

    /// Generate content for dashboard display (rich)
    fn generate_dashboard_content(&self, summary: &MorningSummary) -> String {
        format!("Overnight Summary for {}\n\n{}\n\nTotal Events: {}\nPriority Events: {}",
               summary.summary_date.format("%B %d, %Y"),
               summary.narrative,
               summary.event_count,
               summary.high_priority_events.len())
    }

    /// Create a quiet night summary for when no events occurred
    fn create_quiet_night_summary(&self) -> MorningSummary {
        MorningSummary {
            home_id: "unknown".to_string(), // Will be set by caller
            summary_date: Utc::now().date_naive(),
            event_count: 0,
            narrative: "Good morning! It was a quiet night with no security events detected. Your home security system remained active and monitored throughout the night.".to_string(),
            high_priority_events: Vec::new(),
            requires_attention: false,
        }
    }
}

/// Analysis of patterns in overnight events
#[derive(Debug, Default)]
struct EventPatternAnalysis {
    temporal_distribution: TemporalDistribution,
    sensor_distribution: SensorDistribution,
    threat_analysis: ThreatAnalysis,
    activity_clusters: Vec<ActivityCluster>,
}

/// Analysis of when events occurred throughout the night
#[derive(Debug, Default)]
struct TemporalDistribution {
    hourly_distribution: Vec<usize>,
    peak_hour: usize,
    peak_activity_count: usize,
    is_clustered: bool,
}

/// Analysis of which sensors detected events
#[derive(Debug, Default)]
struct SensorDistribution {
    sensor_counts: std::collections::HashMap<String, usize>,
    most_active_sensor: Option<(String, usize)>,
    unique_sensors: usize,
}

/// Analysis of threat levels in events
#[derive(Debug, Default)]
struct ThreatAnalysis {
    alert_events: usize,
    suppressed_events: usize,
    ignored_events: usize,
    max_threat_level: ThreatLevel,
}

/// Cluster of events that occurred close together in time
#[derive(Debug, Clone)]
struct ActivityCluster {
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    event_count: usize,
    sensors_involved: Vec<String>,
}

/// Threat level classification
#[derive(Debug, Default)]
enum ThreatLevel {
    #[default]
    Low,
    Medium,
    High,
}

/// Summary with delivery-specific content
#[derive(Debug)]
pub struct MorningSummaryWithDelivery {
    pub summary: MorningSummary,
    pub delivery_content: DeliveryContent,
    pub channels: Vec<DeliveryChannel>,
}

/// Content customized for different delivery channels
#[derive(Debug, Default)]
pub struct DeliveryContent {
    pub push_notification: Option<String>,
    pub email: Option<String>,
    pub sms: Option<String>,
    pub websocket: Option<String>,
    pub dashboard: Option<String>,
}

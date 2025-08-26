use super::incident_engine::{Incident, Evidence};
use super::llm_client::{LLMClient, LLMSummaryRequest};
use std::sync::OnceLock;

// Global LLM client for reuse across calls
static LLM_CLIENT: OnceLock<LLMClient> = OnceLock::new();

fn get_llm_client() -> &'static LLMClient {
    LLM_CLIENT.get_or_init(|| LLMClient::new(None))
}

/// Generate incident summary, trying LLM first with rule-based fallback
pub fn summarize_incident(inc: &Incident, fused: &Evidence, calibrated_p: f64, suppressed: u32) -> String {
    // Try LLM summary first (async in sync context)
    if let Ok(runtime) = tokio::runtime::Runtime::new() {
        if let Some(llm_summary) = runtime.block_on(try_llm_summary(inc, fused, calibrated_p)) {
            return format!("{}\n\n📊 Technical Details: threat={:.1}%, LLR={:+.2}, suppressed={}", 
                llm_summary, calibrated_p * 100.0, fused.sum(), suppressed);
        }
    }
    
    // Fallback to rule-based summary
    rule_based_summary(inc, fused, calibrated_p, suppressed)
}

async fn try_llm_summary(inc: &Incident, fused: &Evidence, calibrated_p: f64) -> Option<String> {
    let client = get_llm_client();
    
    // Extract key information from incident
    let rang_doorbell = inc.events.iter().any(|e| e.rang_doorbell);
    let knocked = inc.events.iter().any(|e| e.knocked);
    let total_dwell = inc.total_dwell();
    
    // Determine decision based on probability
    let decision = if calibrated_p >= 0.5 {
        "Critical"
    } else if calibrated_p >= 0.3 {
        "Elevated"
    } else if calibrated_p >= 0.15 {
        "Standard"
    } else {
        "Normal"
    };
    
    // Get first camera/location
    let location = inc.events.first()
        .map(|e| e.cam.clone())
        .unwrap_or_else(|| "front_door".to_string());
    
    let request = LLMSummaryRequest {
        decision: decision.to_string(),
        location,
        dwell_time: total_dwell,
        rang_doorbell,
        knocked,
        threat_probability: calibrated_p,
    };
    
    client.get_summary(request).await
}

/// Rule-based fallback summary (original implementation)
fn rule_based_summary(inc: &Incident, fused: &Evidence, calibrated_p: f64, suppressed: u32) -> String {
    let duration = if let (Some(first), Some(last)) = (inc.events.first(), inc.events.last()) {
        last.ts - first.ts + last.dwell_s
    } else { 0.0 };
    let doors = if inc.events.iter().any(|e| e.rang_doorbell) { "rang doorbell".to_string() }
        else if inc.events.iter().any(|e| e.knocked) { "knocked".to_string() }
        else { "no doorbell/knock".to_string() };
    format!(
        "🔔 Front Door Activity\nTotal dwell {:.0}s over {:.0}s window, {}.\nFused LLR: time={:+.2}, entry={:+.2}, behavior={:+.2}, identity={:+.2}, presence={:+.2}, token={:+.2}.\nCalibrated threat: {:.1}%\nSuppressed duplicates: {}",
        inc.total_dwell(), duration, doors,
        fused.llr_time, fused.llr_entry, fused.llr_behavior, fused.llr_identity, fused.llr_presence, fused.llr_token,
        calibrated_p*100.0, suppressed
    )
}

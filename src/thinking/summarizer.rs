use super::incident_engine::{Incident, Evidence};

pub fn summarize_incident(inc: &Incident, fused: &Evidence, calibrated_p: f64, suppressed: u32) -> String {
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

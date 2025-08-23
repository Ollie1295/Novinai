use std::collections::HashMap;

#[derive(Clone)]
struct Event {
    t_secs: i32,
    device: &'static str,
    person: &'static str,
    calibrated_prob: f64, // already calibrated probability
}

#[derive(Default, Debug)]
struct KeyState {
    last_alert_ts: Option<i32>,
    suppressed_count: u32,
}

fn decide(p: f64) -> &'static str {
    if p >= 0.85 { "CRITICAL" }
    else if p >= 0.60 { "HIGH" }
    else if p >= 0.30 { "ELEVATED" }
    else if p >= 0.15 { "STANDARD" }
    else { "IGNORE" }
}

fn main() {
    println!("📨 ALERT SUPPRESSION TEST (same device, same person, 5 events in 60s)");
    println!("====================================================================\n");

    // Scenario setup: same camera + same recognized person, all within 1 minute
    let device = "FrontDoorCam"; // same device
    let person = "person_123";   // same track/identity

    // First event is STANDARD; subsequent events hover around STANDARD as well
    // Times: 0s, 12s, 24s, 36s, 48s (all within a 60s window)
    let events = vec![
        Event { t_secs: 0,  device, person, calibrated_prob: 0.22 }, // STANDARD
        Event { t_secs: 12, device, person, calibrated_prob: 0.21 }, // STANDARD (would be), but should be suppressed
        Event { t_secs: 24, device, person, calibrated_prob: 0.23 }, // STANDARD (would be), suppressed
        Event { t_secs: 36, device, person, calibrated_prob: 0.20 }, // STANDARD (would be), suppressed
        Event { t_secs: 48, device, person, calibrated_prob: 0.22 }, // STANDARD (would be), suppressed
    ];

    let suppression_window_secs: i32 = 60; // per device+person cooldown
    let mut state: HashMap<(String, String), KeyState> = HashMap::new();

    let mut alerts_sent = 0u32;
    let mut suppressed_total = 0u32;

    for (idx, e) in events.iter().enumerate() {
        let key = (e.device.to_string(), e.person.to_string());
        let st = state.entry(key.clone()).or_insert_with(KeyState::default);

        let decision = decide(e.calibrated_prob);
        let mut suppressed = false;
        let reason: String;

        if let Some(last_ts) = st.last_alert_ts {
            if e.t_secs - last_ts < suppression_window_secs {
                suppressed = true;
                st.suppressed_count += 1;
                suppressed_total += 1;
                reason = format!(
                    "Same device+person within {}s (last alert at {}s)",
                    suppression_window_secs, last_ts
                );
            } else {
                // Window elapsed — allow alert
                st.last_alert_ts = Some(e.t_secs);
                alerts_sent += 1;
                reason = "Window elapsed; sending alert".to_string();
            }
        } else {
            // First alert for this device+person
            st.last_alert_ts = Some(e.t_secs);
            alerts_sent += 1;
            reason = "First alert for device+person".to_string();
        }

        println!(
            "Event {} at t={:>2}s | device={} | person={} | p={:.1}% | would_be={}",
            idx + 1, e.t_secs, e.device, e.person, e.calibrated_prob * 100.0, decision
        );

        if suppressed {
            println!("  → SUPPRESSED: {}", reason);
            println!("    Rollup: suppressed_since_last={}\n", st.suppressed_count);
        } else {
            println!("  → ALERT SENT: {}", decision);
            println!("    Reason: {}\n", reason);
        }
    }

    println!("\n📊 SUMMARY");
    println!("=========");
    println!("Alerts sent: {}", alerts_sent);
    println!("Suppressed: {}", suppressed_total);
    println!("Policy: per device+person cooldown {}s prevents alert spam for duplicates.", suppression_window_secs);
}

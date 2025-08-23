use insane_ai_security::thinking::{
    IncidentStore, Evidence, Event, calibrate_logit,
    generate_questions, ReasonerConfig,
    minimal_changes_to_threshold,
    summarize_incident
};

fn main() {
    let odds_cap = 3.0; let temperature = 1.4; let mean_logit = 0.0; let prior_logit = -2.0;
    let mut store = IncidentStore::new(180.0); let home = "home_001";

    let ev1 = Event {
        ts: 0.0, cam: "FrontDoorCam".to_string(), person_track: "track_abc".to_string(),
        rang_doorbell:false, knocked:false, dwell_s:12.0, away_prob:0.1, expected_window:false, token: None,
        evidence: Evidence{ llr_time:0.0, llr_entry:-0.1, llr_behavior:0.3, llr_identity:0.2, llr_presence:0.2, llr_token:0.0 },
    };
    let ev2 = Event {
        ts: 28.0, cam: "FrontDoorCam".to_string(), person_track: "track_abc".to_string(),
        rang_doorbell:false, knocked:false, dwell_s:18.0, away_prob:0.1, expected_window:false, token: None,
        evidence: Evidence{ llr_time:0.0, llr_entry:-0.1, llr_behavior:0.3, llr_identity:0.2, llr_presence:0.2, llr_token:0.0 },
    };

    let _ = store.upsert_event(home, ev1);
    let _ = store.upsert_event(home, ev2);

    let inc = store.get_incident(home, "track_abc").unwrap().clone();
    let fused = inc.fused_evidence(1.6, 3.0);
    let calibrated = calibrate_logit(prior_logit + fused.sum(), mean_logit, temperature, odds_cap);

    let cfg = ReasonerConfig::default();
    let questions = generate_questions(&inc, &fused, prior_logit, &cfg);

    let standard_threshold_logit = -1.7346; // logit(0.15)
    let cf = minimal_changes_to_threshold(&fused, prior_logit, standard_threshold_logit);

    let summary = summarize_incident(&inc, &fused, calibrated, inc.suppressed_count);

    println!("🧠 THINKING AI DEMO\n====================\n{}", summary);
    println!("\nTop self-questions (VOI):");
    for q in questions.iter().take(3) { println!("  - {:?} (ΔH≈{:.3})", q.q, q.expected_entropy_reduction); }
    println!("\nCounterfactuals to downgrade alert:");
    for s in cf.iter() { println!("  - {} (ΔLLR={:+.2})", s.description, s.delta_llr); }
}

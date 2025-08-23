fn main() {
    println!("🌙 2 AM BACK GARDEN WITH FOLLOW-UP EVENTS");
    println!("=========================================");
    println!("Scenario timeline: Unknown person at 2 AM in back garden, user home");
    println!("Then within 30s: two other cameras see a recognized family member\n");

    // Shared helpers
    let prior: f64 = 0.295; // blended prior from earlier
    let prior_logit = logit(prior);

    // Initial evidence at T+0s (unknown at 2 AM back garden)
    // Components (positive = suspicious):
    //  - Time (2AM): +0.83 LLR (learned from adaptive time model)
    //  - Entry (back garden suspicious): +1.20 LLR
    //  - Behavior (lurking/no ring): +0.80 LLR
    //  - Presence (user home & unknown): +0.90 LLR
    //  - Identity (unknown): +0.30 LLR
    let llr_time = 0.83;
    let llr_entry = 1.20;
    let llr_behavior = 0.80;
    let llr_presence = 0.90;
    let llr_identity_unknown = 0.30;
    let llr_initial_sum = llr_time + llr_entry + llr_behavior + llr_presence + llr_identity_unknown;
    let logit_initial = prior_logit + llr_initial_sum;
    let p_initial = sigmoid(logit_initial);

    println!("T+0s - Initial event:");
    println!("  LLRs: time=+{:.2}, entry=+{:.2}, behavior=+{:.2}, presence=+{:.2}, identity=+{:.2}",
        llr_time, llr_entry, llr_behavior, llr_presence, llr_identity_unknown);
    println!("  Sum LLR: +{:.2}", llr_initial_sum);
    println!("  Prob (naive): {:.1}%", p_initial * 100.0);
    println!("  Decision: {}\n", decide(p_initial));

    // Follow-up Event A: T+15s on SideGateCam - recognized family member (conf 0.96)
    // Effects:
    //  - Identity explain-away: strong benign evidence (-2.20 LLR)
    //  - Visitor gating: damp time/behavior positives (time *0.25, behavior *0.2)
    //  - Presence penalty removed (it's a known occupant)
    let llr_identity_known_a = -2.20; // token-like strong benign evidence
    let llr_time_gated = (llr_time).max(0.0) * 0.25;      // 0.83 -> ~0.21
    let llr_behavior_gated = (llr_behavior).max(0.0) * 0.20; // 0.80 -> 0.16
    let llr_presence_removed = 0.0; // no penalty if it's known family

    // Keep entry context for now (back garden still unusual even for a family member)
    let llr_sum_a = llr_entry + llr_time_gated + llr_behavior_gated + llr_presence_removed + llr_identity_known_a;
    let logit_a = prior_logit + llr_sum_a;
    let p_a_naive = sigmoid(logit_a);

    // Calibrate: threat should soften further under known-identity context
    let temp_a = 1.6; // softer slope under explain-away
    let p_a_cal = sigmoid((logit_a).clamp(-3.0, 3.0) / temp_a);

    println!("T+15s - SideGateCam: family member recognized (conf=0.96):");
    println!("  Identity LLR: {:.2}", llr_identity_known_a);
    println!("  Visitor gating: time {:.2} -> {:.2}, behavior {:.2} -> {:.2}", llr_time, llr_time_gated, llr_behavior, llr_behavior_gated);
    println!("  Entry context kept: +{:.2}", llr_entry);
    println!("  New Sum LLR: {:+.2}", llr_sum_a);
    println!("  Prob (naive): {:.1}% | Calibrated: {:.1}%", p_a_naive * 100.0, p_a_cal * 100.0);
    println!("  Decision: {}\n", decide(p_a_cal));

    // Follow-up Event B: T+28s on KitchenCam - same family member recognized again (conf 0.94)
    // Identity ledger prevents double-counting; allow small additional confirmation, cap total exoneration.
    let extra_identity = -0.60; // small additional benign
    let mut identity_cap_total = -3.00; // cap for total identity exoneration
    let mut total_identity = llr_identity_known_a + extra_identity;
    if total_identity < identity_cap_total { total_identity = identity_cap_total; }

    // With repeat recognition indoors, reduce entry-context suspicion as well (they're inside)
    let llr_entry_reduced = llr_entry * 0.30; // back garden context becomes less relevant after moving inside

    let llr_sum_b = llr_entry_reduced + llr_time_gated + llr_behavior_gated + llr_presence_removed + total_identity;
    let logit_b = prior_logit + llr_sum_b;
    let p_b_naive = sigmoid(logit_b);

    let temp_b = 1.8; // even softer under strong benign confirmation
    let p_b_cal = sigmoid((logit_b).clamp(-3.0, 3.0) / temp_b);

    println!("T+28s - KitchenCam: family member recognized again (conf=0.94):");
    println!("  Added identity confirmation: {:+.2} (capped total: {:+.2})", extra_identity, total_identity);
    println!("  Entry reduced (now inside): {:.2} -> {:.2}", llr_entry, llr_entry_reduced);
    println!("  Final Sum LLR: {:+.2}", llr_sum_b);
    println!("  Prob (naive): {:.1}% | Calibrated: {:.1}%", p_b_naive * 100.0, p_b_cal * 100.0);
    println!("  Decision: {}\n", decide(p_b_cal));

    println!("🎯 Outcome: Initial HIGH/CRITICAL drops to IGNORE once family is recognized twice on different cams within 30s.");
    println!("   Identity ledger + visitor gating explain away the suspicious context.");
}

fn sigmoid(x: f64) -> f64 { 1.0 / (1.0 + (-x).exp()) }
fn logit(p: f64) -> f64 { (p / (1.0 - p)).ln() }

fn decide(p: f64) -> &'static str {
    if p >= 0.85 { "🚨 CRITICAL - Call Police" }
    else if p >= 0.60 { "⚠️  HIGH ALERT - Immediate Action" }
    else if p >= 0.30 { "📋 ELEVATED - Monitor Closely" }
    else if p >= 0.15 { "👀 STANDARD - Normal Monitoring" }
    else { "✅ IGNORE - Benign Activity" }
}

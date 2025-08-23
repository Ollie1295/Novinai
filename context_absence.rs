//! Context-Aware Absence Model - Intelligent Away-Time Risk Assessment

use std::collections::HashMap;

fn main() {
    println!("🏠 CONTEXT-AWARE ABSENCE MODEL");
    println!("===============================");
    println!("Intelligent away-time risk assessment\n");

    let mut absence_model = AbsenceModel::new();
    simulate_away_patterns(&mut absence_model);
    test_absence_scenarios(&mut absence_model);
    demonstrate_absence_intelligence(&mut absence_model);
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ContextKey {
    zone_id: String,
    day_of_week: u8,
    is_away_bucket: u8,
    expected_flag: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AuthTokenType {
    None,
    DeliveryCode,
    GuestPass,
    ServiceCode,
}

struct AbsenceModel {
    benign: HashMap<ContextKey, Vec<f64>>,
    threat: HashMap<ContextKey, Vec<f64>>,
    alpha: f64,
    kappa: f64,
    kernel: Vec<f64>,
    token_bonuses: HashMap<AuthTokenType, f64>,
    unexpected_penalty_cap: f64,
}

impl AbsenceModel {
    fn new() -> Self {
        let mut token_bonuses = HashMap::new();
        token_bonuses.insert(AuthTokenType::DeliveryCode, -2.2);
        token_bonuses.insert(AuthTokenType::GuestPass, -1.6);
        token_bonuses.insert(AuthTokenType::ServiceCode, -2.8);
        token_bonuses.insert(AuthTokenType::None, 0.0);

        Self {
            benign: HashMap::new(),
            threat: HashMap::new(),
            alpha: 1.0,
            kappa: 200.0,
            kernel: vec![0.25, 0.5, 0.25],
            token_bonuses,
            unexpected_penalty_cap: 1.8,
        }
    }

    fn update_history(&mut self, key: ContextKey, time_slot: usize, is_threat: bool) {
        self.benign.entry(key.clone()).or_insert_with(|| vec![0.0; 96]);
        self.threat.entry(key.clone()).or_insert_with(|| vec![0.0; 96]);

        if is_threat {
            self.threat.get_mut(&key).unwrap()[time_slot] += 1.0;
        } else {
            self.benign.get_mut(&key).unwrap()[time_slot] += 1.0;
        }
    }

    fn smoothed_bin(&self, hist: &[f64], slot: usize) -> f64 {
        let prev = (slot + 95) % 96;
        let next = (slot + 1) % 96;
        self.kernel[0] * hist[prev] + self.kernel[1] * hist[slot] + self.kernel[2] * hist[next]
    }

    fn absence_llr_adjustment(&self,
                             away_prob: f64,
                             occ_conf: f64,
                             expected_window_active: bool,
                             auth_token_type: AuthTokenType,
                             zone_id: &str,
                             time_slot: usize,
                             day_of_week: u8) -> f64 {

        let is_away_bucket = if away_prob >= 0.7 { 1 } else { 0 };
        let expected_flag = if expected_window_active { 1 } else { 0 };
        let key = ContextKey {
            zone_id: zone_id.to_string(),
            day_of_week,
            is_away_bucket,
            expected_flag,
        };

        println!("   Absence Analysis:");
        println!("     Away prob: {:.2}, Occ confidence: {:.2}", away_prob, occ_conf);
        println!("     Expected window: {}, Auth token: {:?}", expected_window_active, auth_token_type);
        println!("     Key: away_bucket={}, expected_flag={}", is_away_bucket, expected_flag);

        let empty = vec![0.0; 96];
        let b_hist = self.benign.get(&key).map(|v| v.as_slice()).unwrap_or(&empty);
        let t_hist = self.threat.get(&key).map(|v| v.as_slice()).unwrap_or(&empty);

        let b_s = self.smoothed_bin(b_hist, time_slot);
        let t_s = self.smoothed_bin(t_hist, time_slot);

        let b_tot: f64 = b_hist.iter().sum::<f64>() + self.alpha * 96.0;
        let t_tot: f64 = t_hist.iter().sum::<f64>() + self.alpha * 96.0;
        let p_b = (b_s + self.alpha) / b_tot;
        let p_t = (t_s + self.alpha) / t_tot;

        let llr_emp = (p_t / p_b).ln();
        println!("     Historical: B={:.2}, T={:.2}, LLR_empirical={:.4}", b_s, t_s, llr_emp);

        let n_eff = b_s + t_s;
        let w_data = n_eff / (n_eff + self.kappa);

        let w_presence = if is_away_bucket == 1 {
            (occ_conf * (away_prob - 0.5).abs() * 2.0).clamp(0.0, 1.0)
        } else {
            0.0
        };

        println!("     Data weight: {:.3}, Presence weight: {:.3}", w_data, w_presence);

        let llr_expect = if expected_window_active {
            -2.0  // Strong benign evidence
        } else if is_away_bucket == 1 {
            0.7   // Small penalty for unexpected + away
        } else {
            0.0   // No penalty when home
        };

        let llr_token = *self.token_bonuses.get(&auth_token_type).unwrap_or(&0.0);
        println!("     LLR_expect: {:.3}, LLR_token: {:.3}", llr_expect, llr_token);

        let llr_context = (w_data * w_presence * llr_emp).clamp(-3.0, self.unexpected_penalty_cap);
        let llr_total = llr_context + llr_expect + llr_token;

        println!("     LLR_context: {:.4}, LLR_total: {:.4}", llr_context, llr_total);

        llr_total
    }
}

fn simulate_away_patterns(model: &mut AbsenceModel) {
    println!("📊 SIMULATING AWAY-TIME PATTERNS");
    println!("=================================");

    // Expected deliveries while away
    let delivery_key = ContextKey {
        zone_id: "front_entrance".to_string(),
        day_of_week: 2,
        is_away_bucket: 1,
        expected_flag: 1,
    };

    println!("   Pattern 1: Expected deliveries while away");
    for hour in 10..16 {
        for i in 0..5 {
            let slot = hour * 4 + (i % 4);
            model.update_history(delivery_key.clone(), slot, false);
        }
        if hour == 12 {
            model.update_history(delivery_key.clone(), 12 * 4 + 1, true);
        }
    }

    // Unexpected activity while away  
    let unexpected_key = ContextKey {
        zone_id: "front_entrance".to_string(),
        day_of_week: 2,
        is_away_bucket: 1,
        expected_flag: 0,
    };

    println!("   Pattern 2: Unexpected activity while away");
    for hour in [14, 16, 19, 21] {
        for _ in 0..2 {
            let slot = hour * 4 + (hour % 4);
            model.update_history(unexpected_key.clone(), slot, false);
        }
        if hour == 19 || hour == 21 {
            let slot = hour * 4 + 2;
            model.update_history(unexpected_key.clone(), slot, true);
        }
    }

    // Activity while home
    let home_key = ContextKey {
        zone_id: "front_entrance".to_string(),
        day_of_week: 2,
        is_away_bucket: 0,
        expected_flag: 0,
    };

    println!("   Pattern 3: Activity while home (baseline)");
    for hour in 7..22 {
        for _ in 0..3 {
            let slot = hour * 4 + (hour % 4);
            model.update_history(home_key.clone(), slot, false);
        }
        if hour == 10 {  // One threat when home
            model.update_history(home_key.clone(), 10 * 4, true);
        }
    }

    println!("   Simulated comprehensive away/home patterns\n");
}

fn test_absence_scenarios(model: &mut AbsenceModel) {
    println!("🧪 TESTING ABSENCE SCENARIOS");
    println!("=============================");

    println!("Scenario 1: Unknown person, 8 PM, away, NO expected activity, NO auth");
    let adj1 = model.absence_llr_adjustment(
        0.85, 0.90, false, AuthTokenType::None, "front_entrance", 20 * 4, 2);
    println!("     Result: {:.4} LLR\n", adj1);

    println!("Scenario 2: Same situation but WITH delivery window active");
    let adj2 = model.absence_llr_adjustment(
        0.85, 0.90, true, AuthTokenType::None, "front_entrance", 20 * 4, 2);
    println!("     Result: {:.4} LLR\n", adj2);

    println!("Scenario 3: Unknown person WITH delivery code");
    let adj3 = model.absence_llr_adjustment(
        0.85, 0.90, false, AuthTokenType::DeliveryCode, "front_entrance", 20 * 4, 2);
    println!("     Result: {:.4} LLR\n", adj3);

    println!("Scenario 4: Same unknown person but user is HOME");
    let adj4 = model.absence_llr_adjustment(
        0.15, 0.90, false, AuthTokenType::None, "front_entrance", 20 * 4, 2);
    println!("     Result: {:.4} LLR\n", adj4);

    println!("📊 COMPARISON WITH OLD BROKEN MODEL:");
    println!("     Old model: +0.223 log-odds (+25% paranoid penalty)");
    println!("     New model scenarios:");
    println!("       Away + unexpected: {:.3} LLR", adj1);
    println!("       Away + expected:   {:.3} LLR", adj2);
    println!("       Away + auth code:  {:.3} LLR", adj3);
    println!("       Home + unexpected: {:.3} LLR", adj4);
    println!();
}

fn demonstrate_absence_intelligence(model: &mut AbsenceModel) {
    println!("🧠 ABSENCE INTELLIGENCE DEMONSTRATION");
    println!("======================================");

    let scenarios = vec![
        ("Morning delivery (expected)", 10 * 4, true, AuthTokenType::None, 0.9),
        ("Afternoon service (auth)", 14 * 4, false, AuthTokenType::ServiceCode, 0.8),
        ("Evening visitor (unexpected)", 19 * 4, false, AuthTokenType::None, 0.85),
        ("Late night activity", 23 * 4, false, AuthTokenType::None, 0.75),
        ("Weekend guest (pass)", 11 * 4, false, AuthTokenType::GuestPass, 0.8),
    ];

    for (desc, time_slot, expected, auth, away_prob) in scenarios {
        let adjustment = model.absence_llr_adjustment(
            away_prob, 0.9, expected, auth, "front_entrance", time_slot, 
            if desc.contains("Weekend") { 0 } else { 2 });

        let interpretation = if adjustment < -1.0 {
            "STRONGLY reduces threat (expected/authorized)"
        } else if adjustment < -0.1 {
            "Reduces threat (context supports benign)"
        } else if adjustment < 0.1 {
            "Neutral (no clear absence effect)"
        } else if adjustment < 0.5 {
            "Slight increase (unexpected while away)"
        } else {
            "Significant increase (suspicious pattern)"
        };

        println!("{}: {:.3} LLR - {}", desc, adjustment, interpretation);
    }

    println!("\n🎯 KEY INSIGHTS:");
    println!("• Being away is NOT inherently threatening");
    println!("• Expected activities (deliveries/guests) REDUCE threat while away");
    println!("• Authentication tokens provide strong benign evidence");
    println!("• Historical patterns shape expectations per home/zone/time");
    println!("• Presence confidence prevents false absence penalties");
}

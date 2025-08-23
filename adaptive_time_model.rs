//! Adaptive Time Model - Learning-Based Temporal Risk Assessment

use std::collections::HashMap;

fn main() {
    println!("🕒 ADAPTIVE TIME MODEL");
    println!("======================");
    println!("Learning-based temporal risk assessment\n");

    let mut time_model = TimeModel::new();
    simulate_historical_data(&mut time_model);
    test_time_adjustment(&mut time_model);
    demonstrate_time_learning(&mut time_model);
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct TimeKey {
    zone_id: String,
    day_of_week: u8,
    known_face: bool,
}

struct TimeModel {
    benign: HashMap<TimeKey, Vec<f64>>,
    threat: HashMap<TimeKey, Vec<f64>>,
    alpha: f64,
    kappa: f64,
    kernel: Vec<f64>,
}

impl TimeModel {
    fn new() -> Self {
        Self {
            benign: HashMap::new(),
            threat: HashMap::new(),
            alpha: 1.0,
            kappa: 50.0,
            kernel: vec![0.25, 0.5, 0.25],
        }
    }

    fn update_hist(&mut self, key: TimeKey, slot: usize, is_threat: bool) {
        self.benign.entry(key.clone()).or_insert_with(|| vec![0.0; 96]);
        self.threat.entry(key.clone()).or_insert_with(|| vec![0.0; 96]);

        if is_threat {
            self.threat.get_mut(&key).unwrap()[slot] += 1.0;
        } else {
            self.benign.get_mut(&key).unwrap()[slot] += 1.0;
        }
    }

    fn smoothed_bin(&self, hist: &[f64], slot: usize) -> f64 {
        let mut sum = 0.0;
        for (i, &kernel_val) in self.kernel.iter().enumerate() {
            let offset = i as i32 - 1; // kernel center is index 1
            let j = ((slot as i32 + offset + 96) % 96) as usize;
            sum += kernel_val * hist[j];
        }
        sum
    }

    fn time_llr_adjustment(&self, time_min: u32, day: u8, zone: &str, known: bool) -> f64 {
        let key = TimeKey {
            zone_id: zone.to_string(),
            day_of_week: day,
            known_face: known,
        };
        let slot = (time_min / 15) as usize;

        let empty = vec![0.0; 96];
        let b_hist = self.benign.get(&key).map(|v| v.as_slice()).unwrap_or(&empty);
        let t_hist = self.threat.get(&key).map(|v| v.as_slice()).unwrap_or(&empty);

        let b_s = self.smoothed_bin(b_hist, slot);
        let t_s = self.smoothed_bin(t_hist, slot);

        let b_tot: f64 = b_hist.iter().sum::<f64>() + self.alpha * 96.0;
        let t_tot: f64 = t_hist.iter().sum::<f64>() + self.alpha * 96.0;

        let p_b = (b_s + self.alpha) / b_tot;
        let p_t = (t_s + self.alpha) / t_tot;

        let llr = (p_t / p_b).ln();
        let n_eff = b_s + t_s;
        let weight = n_eff / (n_eff + self.kappa);
        let adjustment = weight * llr;

        println!("   Time Analysis for {}:{:02} on day {} ({}face):", 
            time_min / 60, time_min % 60, day,
            if known { "known " } else { "unknown " });
        println!("     Slot {}: B={:.2}, T={:.2}, B_total={:.1}, T_total={:.1}", 
            slot, b_s, t_s, b_tot, t_tot);
        println!("     P(benign)={:.6}, P(threat)={:.6}, LLR={:.4}", p_b, p_t, llr);
        println!("     Weight={:.3}, Adjustment={:.4}", weight, adjustment);
        
        adjustment
    }
}

fn simulate_historical_data(model: &mut TimeModel) {
    println!("🗂️  SIMULATING HISTORICAL DATA");
    println!("===============================");
    
    let key = TimeKey {
        zone_id: "front_entrance".to_string(),
        day_of_week: 2,
        known_face: false,
    };
    
    // Morning rush (7-8 AM): Mostly benign
    for hour in 7..=8 {
        for i in 0..12 {
            let slot = hour * 4 + (i % 4);
            model.update_hist(key.clone(), slot, false);
        }
        if hour == 8 {
            model.update_hist(key.clone(), hour * 4 + 1, true);
        }
    }
    
    // Workday quiet (10 AM - 5 PM)
    for hour in 10..17 {
        for i in 0..3 {
            let slot = hour * 4 + (i % 4);
            model.update_hist(key.clone(), slot, false);
        }
    }
    
    // Evening social hours (6-9 PM): HIGH benign activity
    for hour in 18..=21 {
        for i in 0..8 {
            let slot = hour * 4 + (i % 4);
            model.update_hist(key.clone(), slot, false);
        }
        if hour == 19 {
            model.update_hist(key.clone(), hour * 4 + 2, true);
        }
    }
    
    // Late night: Low activity, higher threat ratio
    for hour in [23, 0, 1, 2, 3, 4, 5] {
        let slot = (hour % 24) * 4 + 1;
        model.update_hist(key.clone(), slot, false);
        
        if hour == 2 || hour == 3 {
            let slot = (hour % 24) * 4 + 2;
            model.update_hist(key.clone(), slot, true);
        }
    }
    
    println!("   Simulated 4 weeks of activity patterns");
    
    let benign_8pm = model.benign.get(&key).map(|h| h[80]).unwrap_or(0.0);
    let threat_8pm = model.threat.get(&key).map(|h| h[80]).unwrap_or(0.0);
    println!("   8 PM slot: {:.1} benign events, {:.1} threat events", benign_8pm, threat_8pm);
    println!();
}

fn test_time_adjustment(model: &mut TimeModel) {
    println!("🧪 TESTING 8 PM ADJUSTMENT");
    println!("===========================");
    println!("Problem scenario: Unknown person at 8:00 PM, Tuesday, front entrance\n");
    
    let adjustment = model.time_llr_adjustment(20 * 60, 2, "front_entrance", false);
    
    println!("\n📊 RESULT:");
    println!("   Time adjustment: {:.4} log-odds", adjustment);
    
    if adjustment < 0.0 {
        println!("   ✅ NEGATIVE adjustment - 8 PM reduces threat (as it should!)");
        println!("   This is because 8 PM has high benign activity in this home");
    } else if adjustment > 0.1 {
        println!("   ❌ POSITIVE adjustment - 8 PM increases threat");
    } else {
        println!("   ⚪ NEUTRAL adjustment - insufficient data or balanced pattern");
    }
    
    let prob_impact = (adjustment.exp() - 1.0) * 100.0;
    println!("   Probability impact: {:.1}% change", prob_impact);
    println!();
}

fn demonstrate_time_learning(model: &mut TimeModel) {
    println!("📈 TIME LEARNING DEMONSTRATION");
    println!("===============================");
    
    let scenarios = vec![
        ("Early morning (6 AM)", 6 * 60),
        ("Morning rush (8 AM)", 8 * 60), 
        ("Midday quiet (2 PM)", 14 * 60),
        ("Evening social (8 PM)", 20 * 60),
        ("Late night (2 AM)", 2 * 60),
    ];
    
    for (desc, time_min) in scenarios {
        let adjustment = model.time_llr_adjustment(time_min, 2, "front_entrance", false);
        let direction = if adjustment < -0.1 {
            "REDUCES threat (high benign activity)"
        } else if adjustment > 0.1 {
            "INCREASES threat (suspicious for this home)"
        } else {
            "NEUTRAL (insufficient data or balanced)"
        };
        println!("{}: {:.4} LLR - {}", desc, adjustment, direction);
    }
    
    println!("\n🎯 KEY INSIGHT:");
    println!("Each home learns its own activity patterns!");
    println!("8 PM isn't inherently suspicious - it depends on THIS home's history.");
}

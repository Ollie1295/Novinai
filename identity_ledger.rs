//! Identity Evidence Ledger - Single-Source Truth for Identity Assessment

use std::collections::{HashMap, HashSet};

fn main() {
    println!("🆔 IDENTITY EVIDENCE LEDGER");
    println!("===========================");
    println!("Single-source truth for identity assessment\n");

    let mut identity_system = IdentitySystem::new();
    simulate_identity_patterns(&mut identity_system);
    test_identity_scenarios(&mut identity_system);
    demonstrate_evidence_protection(&mut identity_system);
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct IdentityKey {
    zone_id: String,
    day_of_week: u8,
    time_slot: u8,
    presence_bucket: u8,
}

struct EvidenceLedger {
    used: HashSet<String>,
    log_odds: f64,
}

impl EvidenceLedger {
    fn new(base_log_odds: f64) -> Self {
        Self {
            used: HashSet::new(),
            log_odds: base_log_odds,
        }
    }
    
    fn claim(&mut self, tag: &str) -> bool {
        if self.used.contains(tag) {
            false
        } else {
            self.used.insert(tag.to_string());
            true
        }
    }
    
    fn is_claimed(&self, tag: &str) -> bool {
        self.used.contains(tag)
    }
}

struct IdentitySystem {
    counts_known_benign: HashMap<IdentityKey, Vec<f64>>,
    counts_known_threat: HashMap<IdentityKey, Vec<f64>>,
    counts_unknown_benign: HashMap<IdentityKey, Vec<f64>>,
    counts_unknown_threat: HashMap<IdentityKey, Vec<f64>>,
    fnr_unknown: f64,
    fpr_known: f64,
    alpha: f64,
    kappa: f64,
    kernel: Vec<f64>,
    llr_cap_pos: f64,
    llr_cap_neg: f64,
}

impl IdentitySystem {
    fn new() -> Self {
        Self {
            counts_known_benign: HashMap::new(),
            counts_known_threat: HashMap::new(),
            counts_unknown_benign: HashMap::new(),
            counts_unknown_threat: HashMap::new(),
            fnr_unknown: 0.15,
            fpr_known: 0.05,
            alpha: 1.0,
            kappa: 200.0,
            kernel: vec![0.25, 0.5, 0.25],
            llr_cap_pos: 1.5,
            llr_cap_neg: 2.5,
        }
    }
    
    fn update_counts(&mut self, key: IdentityKey, time_slot: usize, is_known: bool, is_threat: bool) {
        self.counts_known_benign.entry(key.clone()).or_insert_with(|| vec![0.0; 24]);
        self.counts_known_threat.entry(key.clone()).or_insert_with(|| vec![0.0; 24]);
        self.counts_unknown_benign.entry(key.clone()).or_insert_with(|| vec![0.0; 24]);
        self.counts_unknown_threat.entry(key.clone()).or_insert_with(|| vec![0.0; 24]);
        
        if is_known {
            if is_threat {
                self.counts_known_threat.get_mut(&key).unwrap()[time_slot] += 1.0;
            } else {
                self.counts_known_benign.get_mut(&key).unwrap()[time_slot] += 1.0;
            }
        } else {
            if is_threat {
                self.counts_unknown_threat.get_mut(&key).unwrap()[time_slot] += 1.0;
            } else {
                self.counts_unknown_benign.get_mut(&key).unwrap()[time_slot] += 1.0;
            }
        }
    }
    
    fn smoothed_bin(&self, hist: &[f64], slot: usize) -> f64 {
        let prev = if slot == 0 { 23 } else { slot - 1 };
        let next = (slot + 1) % 24;
        self.kernel[0] * hist[prev] + self.kernel[1] * hist[slot] + self.kernel[2] * hist[next]
    }
    
    fn identity_llr_adjustment(&self,
                               ledger: &mut EvidenceLedger,
                               is_known_face: bool,
                               face_conf: f64,
                               occluded: bool,
                               zone_id: &str,
                               day_of_week: u8,
                               time_slot: usize,
                               away_prob: f64) -> f64 {
        
        if !ledger.claim("identity") {
            println!("     ⚠️  Identity evidence already claimed - returning 0.0");
            return 0.0;
        }
        
        println!("     🆔 Processing identity evidence (claimed successfully)");
        
        let unknown_flag = !is_known_face;
        let presence_bucket = if away_prob >= 0.7 { 1 } else { 0 };
        let key = IdentityKey {
            zone_id: zone_id.to_string(),
            day_of_week,
            time_slot: (time_slot / 4) as u8,
            presence_bucket,
        };
        
        println!("     Context: zone={}, day={}, hour={}, away_bucket={}", 
            zone_id, day_of_week, time_slot / 4, presence_bucket);
        
        let empty = vec![0.0; 24];
        let ku_b = self.counts_known_benign.get(&key).unwrap_or(&empty);
        let ku_t = self.counts_known_threat.get(&key).unwrap_or(&empty);
        let uu_b = self.counts_unknown_benign.get(&key).unwrap_or(&empty);
        let uu_t = self.counts_unknown_threat.get(&key).unwrap_or(&empty);
        
        let hour = time_slot / 4;
        let ku_b_smooth = self.smoothed_bin(ku_b, hour);
        let ku_t_smooth = self.smoothed_bin(ku_t, hour);
        let uu_b_smooth = self.smoothed_bin(uu_b, hour);
        let uu_t_smooth = self.smoothed_bin(uu_t, hour);
        
        println!("     Historical counts - Known: B={:.2}, T={:.2} | Unknown: B={:.2}, T={:.2}", 
            ku_b_smooth, ku_t_smooth, uu_b_smooth, uu_t_smooth);
        
        let b_tot = ku_b_smooth + uu_b_smooth + self.alpha * 2.0;
        let t_tot = ku_t_smooth + uu_t_smooth + self.alpha * 2.0;
        
        let p_unknown_given_benign = (uu_b_smooth + self.alpha) / b_tot;
        let p_unknown_given_threat = (uu_t_smooth + self.alpha) / t_tot;
        
        let llr_emp = (p_unknown_given_threat / p_unknown_given_benign).ln();
        
        println!("     P(unknown|benign)={:.4}, P(unknown|threat)={:.4}, LLR_emp={:.4}", 
            p_unknown_given_benign, p_unknown_given_threat, llr_emp);
        
        let n_eff = ku_b_smooth + ku_t_smooth + uu_b_smooth + uu_t_smooth;
        let w_data = n_eff / (n_eff + self.kappa);
        
        let vis_penalty = if occluded { 0.3 } else { 1.0 };
        let recog_quality = face_conf * (1.0 - self.fnr_unknown) * (1.0 - self.fpr_known);
        let w_recog = vis_penalty * recog_quality.clamp(0.0, 1.0);
        
        println!("     Data weight: {:.3}, Recog weight: {:.3} (conf={:.2}, vis_penalty={:.1})", 
            w_data, w_recog, face_conf, vis_penalty);
        
        let direction = if unknown_flag { 1.0 } else { -1.0 };
        let llr = (w_data * w_recog * llr_emp * direction).clamp(-self.llr_cap_neg, self.llr_cap_pos);
        
        println!("     Final LLR: {:.4} (unknown={}, direction={})", llr, unknown_flag, direction);
        
        llr
    }
    
    fn broken_double_count_example(&self, is_known_face: bool, _face_conf: f64) -> (f64, f64, f64) {
        let base_prior = 0.3;
        let identity_penalty = if !is_known_face { 0.2 } else { 0.0 };
        let total_broken = base_prior + identity_penalty;
        (base_prior, identity_penalty, total_broken)
    }
}

fn simulate_identity_patterns(system: &mut IdentitySystem) {
    println!("🗂️  SIMULATING IDENTITY PATTERNS");
    println!("=================================");
    
    let key_home = IdentityKey {
        zone_id: "front_entrance".to_string(),
        day_of_week: 2,
        time_slot: 20,
        presence_bucket: 0,
    };
    
    let key_away = IdentityKey {
        zone_id: "front_entrance".to_string(),
        day_of_week: 2,
        time_slot: 20,
        presence_bucket: 1,
    };
    
    println!("   Pattern 1: When user is home");
    for _ in 0..15 {
        system.update_counts(key_home.clone(), 20, true, false);
    }
    for _ in 0..3 {
        system.update_counts(key_home.clone(), 20, false, false);
    }
    system.update_counts(key_home.clone(), 20, false, true);
    
    println!("   Pattern 2: When user is away");
    for _ in 0..8 {
        system.update_counts(key_away.clone(), 20, false, false);
    }
    for _ in 0..2 {
        system.update_counts(key_away.clone(), 20, true, false);
    }
    for _ in 0..2 {
        system.update_counts(key_away.clone(), 20, false, true);
    }
    system.update_counts(key_away.clone(), 20, true, true);
    
    println!("   Simulated realistic identity patterns by presence state\n");
}

fn test_identity_scenarios(system: &mut IdentitySystem) {
    println!("🧪 TESTING IDENTITY SCENARIOS");
    println!("==============================");
    
    println!("Scenario 1: Unknown person, user home, high face confidence");
    let mut ledger1 = EvidenceLedger::new(-2.3);
    let adj1 = system.identity_llr_adjustment(
        &mut ledger1, false, 0.9, false, "front_entrance", 2, 20 * 4, 0.2);
    println!("     Identity LLR: {:.4}\n", adj1);
    
    println!("Scenario 2: Unknown person, user away, high face confidence");
    let mut ledger2 = EvidenceLedger::new(-2.3);
    let adj2 = system.identity_llr_adjustment(
        &mut ledger2, false, 0.9, false, "front_entrance", 2, 20 * 4, 0.85);
    println!("     Identity LLR: {:.4}\n", adj2);
    
    println!("Scenario 3: Known person, user away, high face confidence");
    let mut ledger3 = EvidenceLedger::new(-2.3);
    let adj3 = system.identity_llr_adjustment(
        &mut ledger3, true, 0.95, false, "front_entrance", 2, 20 * 4, 0.85);
    println!("     Identity LLR: {:.4}\n", adj3);
    
    println!("Scenario 4: Unknown person, occluded face, low confidence");
    let mut ledger4 = EvidenceLedger::new(-2.3);
    let adj4 = system.identity_llr_adjustment(
        &mut ledger4, false, 0.4, true, "front_entrance", 2, 20 * 4, 0.85);
    println!("     Identity LLR: {:.4}\n", adj4);
    
    println!("📊 COMPARISON WITH BROKEN DOUBLE-COUNTING:");
    let (base, penalty, total) = system.broken_double_count_example(false, 0.9);
    println!("     Broken system: Base={:.3} + Identity={:.3} = Total={:.3}", base, penalty, total);
    println!("     Fixed system scenarios:");
    println!("       Unknown + home: {:.3} LLR", adj1);
    println!("       Unknown + away: {:.3} LLR", adj2);
    println!("       Known + away:   {:.3} LLR", adj3);
    println!("       Unknown + poor visibility: {:.3} LLR", adj4);
    println!();
}

fn demonstrate_evidence_protection(system: &mut IdentitySystem) {
    println!("🛡️  EVIDENCE LEDGER PROTECTION");
    println!("===============================");
    
    let mut ledger = EvidenceLedger::new(-2.0);
    
    println!("Initial ledger state: claimed={:?}", ledger.used);
    
    let adj1 = system.identity_llr_adjustment(
        &mut ledger, false, 0.8, false, "front_entrance", 2, 20 * 4, 0.8);
    println!("First identity call: {:.4} LLR", adj1);
    println!("Ledger after first call: claimed={:?}", ledger.used);
    
    let adj2 = system.identity_llr_adjustment(
        &mut ledger, false, 0.8, false, "front_entrance", 2, 20 * 4, 0.8);
    println!("Second identity call: {:.4} LLR", adj2);
    println!("Ledger after second call: claimed={:?}", ledger.used);
    
    println!("\nSimulating other evidence modules:");
    
    if ledger.claim("time") {
        println!("Time module: Successfully claimed 'time' evidence");
        ledger.log_odds += -0.15;
    }
    
    if ledger.claim("location") {
        println!("Location module: Successfully claimed 'location' evidence");
        ledger.log_odds += 0.0;
    }
    
    if ledger.claim("identity") {
        println!("Broken module: Successfully claimed 'identity' - THIS SHOULD NOT HAPPEN");
        ledger.log_odds += 0.5;
    } else {
        println!("Broken module: Failed to claim 'identity' - CORRECTLY BLOCKED");
    }
    
    println!("\nFinal ledger state: claimed={:?}", ledger.used);
    println!("Final log odds: {:.4}", ledger.log_odds);
    
    let final_prob = 1.0 / (1.0 + (-ledger.log_odds).exp());
    println!("Final probability: {:.4} ({:.1}%)", final_prob, final_prob * 100.0);
    
    println!("\n🎯 KEY INSIGHTS:");
    println!("• Identity evidence is processed exactly once");
    println!("• Evidence Ledger prevents double-counting");
    println!("• Multiple modules can claim different evidence types");
    println!("• Attempts to re-claim used evidence are blocked");
    println!("• System is protected from downstream identity penalties");
}

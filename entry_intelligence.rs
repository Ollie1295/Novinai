//! Entry Point Intelligence - Behavior-Based Entry Assessment

use std::collections::HashMap;

fn main() {
    println!("🚪 ENTRY POINT INTELLIGENCE");
    println!("============================");
    println!("Behavior-based entry assessment\n");

    let mut entry_system = EntrySystem::new();
    simulate_entry_patterns(&mut entry_system);
    test_entry_scenarios(&mut entry_system);
    demonstrate_entry_behavior(&mut entry_system);
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct EntryKey {
    day_of_week: u8,
    is_away_bucket: u8,
    expected_flag: u8,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum EntryPoint {
    FrontDoor,
    BackDoor,
    Window,
    Garage,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AuthTokenType {
    None,
    DeliveryCode,
    GuestPass,
    ServiceCode,
}

struct EntrySystem {
    benign: HashMap<EntryKey, HashMap<EntryPoint, Vec<f64>>>,
    threat: HashMap<EntryKey, HashMap<EntryPoint, Vec<f64>>>,
    alpha: f64,
    kappa: f64,
    kernel: Vec<f64>,
    token_bonus: HashMap<AuthTokenType, f64>,
    ring_knock_llr: f64,
    public_path_llr: f64,
    long_lurk_llr: f64,
    lurk_threshold_sec: f64,
    pos_cap: f64,
    neg_cap: f64,
}

impl EntrySystem {
    fn new() -> Self {
        let mut token_bonus = HashMap::new();
        token_bonus.insert(AuthTokenType::DeliveryCode, -2.2);
        token_bonus.insert(AuthTokenType::GuestPass, -1.6);
        token_bonus.insert(AuthTokenType::ServiceCode, -2.8);
        token_bonus.insert(AuthTokenType::None, 0.0);

        Self {
            benign: HashMap::new(),
            threat: HashMap::new(),
            alpha: 1.0,
            kappa: 200.0,
            kernel: vec![0.25, 0.5, 0.25],
            token_bonus,
            ring_knock_llr: -1.2,
            public_path_llr: -0.6,
            long_lurk_llr: 0.9,
            lurk_threshold_sec: 25.0,
            pos_cap: 1.6,
            neg_cap: 3.0,
        }
    }
    
    fn update_entry_data(&mut self, key: EntryKey, entry_point: EntryPoint, time_slot: usize, is_threat: bool) {
        self.benign.entry(key.clone()).or_insert_with(HashMap::new)
            .entry(entry_point.clone()).or_insert_with(|| vec![0.0; 96]);
        self.threat.entry(key.clone()).or_insert_with(HashMap::new)
            .entry(entry_point.clone()).or_insert_with(|| vec![0.0; 96]);
        
        if is_threat {
            self.threat.get_mut(&key).unwrap()
                .get_mut(&entry_point).unwrap()[time_slot] += 1.0;
        } else {
            self.benign.get_mut(&key).unwrap()
                .get_mut(&entry_point).unwrap()[time_slot] += 1.0;
        }
    }
    
    fn smoothed_bin(&self, hist: &[f64], slot: usize) -> f64 {
        let prev = (slot + 95) % 96;
        let next = (slot + 1) % 96;
        self.kernel[0] * hist[prev] + self.kernel[1] * hist[slot] + self.kernel[2] * hist[next]
    }
    
    fn entry_llr_adjustment(&self,
                            entry_point: EntryPoint,
                            pressed_doorbell: bool,
                            knocked: bool,
                            auth_token_type: AuthTokenType,
                            dwell_sec: f64,
                            approach_from_public_path: bool,
                            time_slot: usize,
                            day_of_week: u8,
                            expected_window_active: bool,
                            away_prob: f64) -> f64 {
        
        let expected_flag = if expected_window_active { 1 } else { 0 };
        let is_away_bucket = if away_prob >= 0.7 { 1 } else { 0 };
        let key = EntryKey { day_of_week, is_away_bucket, expected_flag };
        
        println!("   Entry Analysis for {:?}:", entry_point);
        println!("     Context: day={}, away={}, expected={}", day_of_week, is_away_bucket, expected_flag);
        println!("     Behavior: doorbell={}, knock={}, dwell={:.1}s, public_path={}", 
            pressed_doorbell, knocked, dwell_sec, approach_from_public_path);
        println!("     Auth: {:?}", auth_token_type);

        let empty_map = HashMap::new();
        let empty_hist = vec![0.0; 96];
        
        let b_hist = self.benign.get(&key).unwrap_or(&empty_map).get(&entry_point).unwrap_or(&empty_hist);
        let t_hist = self.threat.get(&key).unwrap_or(&empty_map).get(&entry_point).unwrap_or(&empty_hist);

        let b_s = self.smoothed_bin(b_hist, time_slot);
        let t_s = self.smoothed_bin(t_hist, time_slot);

        let b_tot: f64 = b_hist.iter().sum::<f64>() + self.alpha * 96.0;
        let t_tot: f64 = t_hist.iter().sum::<f64>() + self.alpha * 96.0;

        let p_b = (b_s + self.alpha) / b_tot;
        let p_t = (t_s + self.alpha) / t_tot;

        let llr_emp = (p_t / p_b).ln();
        let n_eff = b_s + t_s;
        let w_data = n_eff / (n_eff + self.kappa);

        println!("     Historical: B={:.2}, T={:.2}, P(B)={:.4}, P(T)={:.4}", b_s, t_s, p_b, p_t);
        println!("     Empirical LLR: {:.4}, Data weight: {:.3}", llr_emp, w_data);

        let mut llr_behavior = 0.0;
        
        if pressed_doorbell || knocked {
            llr_behavior += self.ring_knock_llr;
            println!("     Ring/knock detected: {:.3} LLR", self.ring_knock_llr);
        }
        
        if approach_from_public_path {
            llr_behavior += self.public_path_llr;
            println!("     Public path approach: {:.3} LLR", self.public_path_llr);
        }
        
        if !pressed_doorbell && !knocked && dwell_sec >= self.lurk_threshold_sec {
            llr_behavior += self.long_lurk_llr;
            println!("     Long lurk without ring/knock: {:.3} LLR", self.long_lurk_llr);
        }

        let llr_token = *self.token_bonus.get(&auth_token_type).unwrap_or(&0.0);
        if llr_token != 0.0 {
            println!("     Auth token bonus: {:.3} LLR", llr_token);
        }

        let llr_expected = if expected_window_active { -2.0 } else { 0.0 };
        if llr_expected != 0.0 {
            println!("     Expected window: {:.3} LLR", llr_expected);
        }

        let llr_context = w_data * llr_emp;
        let llr_total = (llr_context + llr_behavior + llr_token + llr_expected).clamp(-self.neg_cap, self.pos_cap);

        println!("     Components: Context={:.4}, Behavior={:.4}, Token={:.4}, Expected={:.4}", 
            llr_context, llr_behavior, llr_token, llr_expected);
        println!("     Final Entry LLR: {:.4}", llr_total);

        llr_total
    }
    
    fn broken_front_door_bias(&self) -> f64 {
        0.15
    }
}

fn simulate_entry_patterns(system: &mut EntrySystem) {
    println!("🗂️  SIMULATING ENTRY PATTERNS");
    println!("==============================");

    let key_normal = EntryKey { day_of_week: 2, is_away_bucket: 0, expected_flag: 0 };
    let key_away = EntryKey { day_of_week: 2, is_away_bucket: 1, expected_flag: 0 };
    let key_expected = EntryKey { day_of_week: 2, is_away_bucket: 1, expected_flag: 1 };

    println!("   Pattern 1: Front door when user is home");
    for hour in [10, 14, 18, 19, 20] {
        for _ in 0..8 {
            let slot = hour * 4;
            system.update_entry_data(key_normal.clone(), EntryPoint::FrontDoor, slot, false);
        }
        if hour == 20 {
            system.update_entry_data(key_normal.clone(), EntryPoint::FrontDoor, 20 * 4, true);
        }
    }

    println!("   Pattern 2: Front door when user is away (unexpected)");
    for hour in [12, 16, 19] {
        for _ in 0..3 {
            let slot = hour * 4;
            system.update_entry_data(key_away.clone(), EntryPoint::FrontDoor, slot, false);
        }
        if hour == 19 {
            system.update_entry_data(key_away.clone(), EntryPoint::FrontDoor, 19 * 4, true);
        }
    }

    println!("   Pattern 3: Front door expected deliveries/services");
    for hour in [10, 11, 13, 14, 15] {
        for _ in 0..6 {
            let slot = hour * 4;
            system.update_entry_data(key_expected.clone(), EntryPoint::FrontDoor, slot, false);
        }
        if hour == 11 {
            system.update_entry_data(key_expected.clone(), EntryPoint::FrontDoor, 11 * 4, true);
        }
    }

    println!("   Pattern 4: Back door and window patterns");
    for hour in [2, 3, 22, 23] {
        for _ in 0..2 {
            system.update_entry_data(key_away.clone(), EntryPoint::BackDoor, hour * 4, false);
        }
        system.update_entry_data(key_away.clone(), EntryPoint::BackDoor, hour * 4, true);
        
        system.update_entry_data(key_away.clone(), EntryPoint::Window, hour * 4, false);
        for _ in 0..3 {
            system.update_entry_data(key_away.clone(), EntryPoint::Window, hour * 4, true);
        }
    }

    println!("   Simulated realistic entry patterns by location and context\n");
}

fn test_entry_scenarios(system: &mut EntrySystem) {
    println!("🧪 TESTING ENTRY SCENARIOS");
    println!("===========================");

    println!("Scenario 1: Normal front door visitor - rings doorbell, 8PM, user home");
    let adj1 = system.entry_llr_adjustment(
        EntryPoint::FrontDoor, true, false, AuthTokenType::None, 5.0, true, 20 * 4, 2, false, 0.2);
    println!("     Result: {:.4} LLR\n", adj1);

    println!("Scenario 2: Front door visitor - no ring, long lurk, 8PM, user away");
    let adj2 = system.entry_llr_adjustment(
        EntryPoint::FrontDoor, false, false, AuthTokenType::None, 30.0, true, 20 * 4, 2, false, 0.85);
    println!("     Result: {:.4} LLR\n", adj2);

    println!("Scenario 3: Expected delivery with code - front door, user away");
    let adj3 = system.entry_llr_adjustment(
        EntryPoint::FrontDoor, true, false, AuthTokenType::DeliveryCode, 8.0, true, 14 * 4, 2, true, 0.85);
    println!("     Result: {:.4} LLR\n", adj3);

    println!("Scenario 4: Back door entry - no ring, user away, night");
    let adj4 = system.entry_llr_adjustment(
        EntryPoint::BackDoor, false, false, AuthTokenType::None, 15.0, false, 23 * 4, 2, false, 0.85);
    println!("     Result: {:.4} LLR\n", adj4);

    println!("Scenario 5: Window approach - very suspicious");
    let adj5 = system.entry_llr_adjustment(
        EntryPoint::Window, false, false, AuthTokenType::None, 20.0, false, 2 * 4, 2, false, 0.85);
    println!("     Result: {:.4} LLR\n", adj5);

    let broken_penalty = system.broken_front_door_bias();
    println!("📊 COMPARISON WITH BROKEN FRONT DOOR BIAS:");
    println!("     Broken system: +{:.3} penalty for ANY front door use", broken_penalty);
    println!("     Fixed system scenarios:");
    println!("       Normal front door visit:    {:.3} LLR", adj1);
    println!("       Lurking at front door:      {:.3} LLR", adj2);
    println!("       Expected delivery + code:   {:.3} LLR", adj3);
    println!("       Back door at night:         {:.3} LLR", adj4);
    println!("       Window approach:            {:.3} LLR", adj5);
    println!();
}

fn demonstrate_entry_behavior(system: &mut EntrySystem) {
    println!("🎭 ENTRY BEHAVIOR INTELLIGENCE");
    println!("===============================");

    let scenarios = vec![
        ("Doorbell + delivery code", true, false, AuthTokenType::DeliveryCode, 5.0, true),
        ("Knock + public approach", false, true, AuthTokenType::None, 8.0, true),
        ("Guest pass + doorbell", true, false, AuthTokenType::GuestPass, 3.0, true),
        ("Service code + knock", false, true, AuthTokenType::ServiceCode, 6.0, true),
        ("No ring, quick visit", false, false, AuthTokenType::None, 12.0, true),
        ("No ring, long lurk", false, false, AuthTokenType::None, 35.0, true),
        ("Side approach, no ring", false, false, AuthTokenType::None, 18.0, false),
    ];

    println!("All scenarios: Front door, 2PM, user away, no expected window\n");

    for (desc, doorbell, knock, auth, dwell, public_path) in scenarios {
        let adjustment = system.entry_llr_adjustment(
            EntryPoint::FrontDoor, doorbell, knock, auth, dwell, public_path, 14 * 4, 2, false, 0.8);

        let interpretation = if adjustment < -1.5 {
            "STRONGLY reduces threat (authorized/normal behavior)"
        } else if adjustment < -0.5 {
            "Reduces threat (good entry behavior)"
        } else if adjustment < 0.1 {
            "Neutral (standard entry behavior)"
        } else if adjustment < 0.8 {
            "Slight increase (mildly suspicious behavior)"
        } else {
            "Significant increase (suspicious entry pattern)"
        };

        println!("{}: {:.3} LLR - {}", desc, adjustment, interpretation);
    }

    println!("\n🎯 KEY INSIGHTS:");
    println!("• Front doors are GOOD - they're where legitimate visitors go");
    println!("• Ring/knock behavior strongly indicates benign intent");
    println!("• Authentication codes provide powerful benign evidence");
    println!("• Long lurking without ring/knock is the actual suspicious behavior");
    println!("• Entry point + behavior context determines threat, not location alone");
    println!("• Back doors and windows have naturally different threat profiles");
}

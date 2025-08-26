//! Debug Evidence Capping

use insane_ai_security::thinking::{ThinkingAIConfig, Evidence};

fn main() {
    println!("🔍 DEBUG EVIDENCE CAPPING");
    println!("========================");
    
    let config = ThinkingAIConfig::default();
    println!("pos_cap: {}, neg_cap: {}", config.pos_cap, config.neg_cap);
    
    // Intruder evidence before capping
    let intruder_raw = Evidence {
        llr_time: 3.2,      
        llr_entry: -3.5,    
        llr_behavior: 3.8,  
        llr_identity: -2.8, 
        llr_presence: 2.5,  
        llr_token: -3.0,    
    };
    
    println!("\n🔥 INTRUDER EVIDENCE:");
    println!("Before capping: {:.2}", intruder_raw.sum());
    
    // Manually apply capping like fused_evidence does
    let intruder_capped = Evidence {
        llr_time: intruder_raw.llr_time.clamp(-config.neg_cap, config.pos_cap),
        llr_entry: intruder_raw.llr_entry.clamp(-config.neg_cap, config.pos_cap),
        llr_behavior: intruder_raw.llr_behavior.clamp(-config.neg_cap, config.pos_cap),
        llr_identity: intruder_raw.llr_identity.clamp(-config.neg_cap, config.pos_cap),
        llr_presence: intruder_raw.llr_presence.clamp(-config.neg_cap, config.pos_cap),
        llr_token: intruder_raw.llr_token.clamp(-config.neg_cap, config.pos_cap),
    };
    
    println!("After capping: {:.2}", intruder_capped.sum());
    println!("Components after capping:");
    println!("  time: {:.2} (was {:.2})", intruder_capped.llr_time, intruder_raw.llr_time);
    println!("  entry: {:.2} (was {:.2})", intruder_capped.llr_entry, intruder_raw.llr_entry);
    println!("  behavior: {:.2} (was {:.2})", intruder_capped.llr_behavior, intruder_raw.llr_behavior);
    println!("  identity: {:.2} (was {:.2})", intruder_capped.llr_identity, intruder_raw.llr_identity);
    println!("  presence: {:.2} (was {:.2})", intruder_capped.llr_presence, intruder_raw.llr_presence);
    println!("  token: {:.2} (was {:.2})", intruder_capped.llr_token, intruder_raw.llr_token);
    
    // Family evidence
    let family_raw = Evidence {
        llr_time: -0.5,     
        llr_entry: 1.2,     
        llr_behavior: -1.8, 
        llr_identity: 2.1,  
        llr_presence: 0.8,  
        llr_token: 1.5,     
    };
    
    println!("\n✅ FAMILY EVIDENCE:");
    println!("Before capping: {:.2}", family_raw.sum());
    
    let family_capped = Evidence {
        llr_time: family_raw.llr_time.clamp(-config.neg_cap, config.pos_cap),
        llr_entry: family_raw.llr_entry.clamp(-config.neg_cap, config.pos_cap),
        llr_behavior: family_raw.llr_behavior.clamp(-config.neg_cap, config.pos_cap),
        llr_identity: family_raw.llr_identity.clamp(-config.neg_cap, config.pos_cap),
        llr_presence: family_raw.llr_presence.clamp(-config.neg_cap, config.pos_cap),
        llr_token: family_raw.llr_token.clamp(-config.neg_cap, config.pos_cap),
    };
    
    println!("After capping: {:.2}", family_capped.sum());
    
    println!("\n💡 ANALYSIS:");
    println!("The intruder evidence is being severely capped!");
    println!("  Raw sum: {:.2} → Capped sum: {:.2}", intruder_raw.sum(), intruder_capped.sum());
    println!("  Loss: {:.2} LLR units", intruder_raw.sum() - intruder_capped.sum());
}

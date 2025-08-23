fn main() {
    println!("🌙 2 AM BACK GARDEN - LEGITIMATE HIGH THREAT");
    println!("=============================================");
    
    let contextual_prior: f64 = 0.40; // Very high for 2 AM back garden
    let personal_prior: f64 = 0.05;   // Unknown person base
    let blended_prior: f64 = 0.7 * contextual_prior + 0.3 * personal_prior;
    
    println!("Priors:");
    println!("  2 AM back garden context: {:.1}% (very suspicious)", contextual_prior * 100.0);
    println!("  Unknown person: {:.1}%", personal_prior * 100.0);  
    println!("  Blended: {:.1}%", blended_prior * 100.0);
    
    let evidence_boost: f64 = 2.1; // Strong evidence for this scenario
    let final_prob: f64 = sigmoid(logit(blended_prior) + evidence_boost);
    
    println!("\nEvidence Analysis:");
    println!("  Evidence boost: {:.1} LLR (motion + approach + face)", evidence_boost);
    println!("  Final probability: {:.1}%", final_prob * 100.0);
    
    let decision = if final_prob > 0.85 {
        "🚨 CRITICAL - Call Police Immediately"
    } else if final_prob > 0.60 {
        "⚠️  HIGH ALERT - Take Immediate Action"  
    } else {
        "📋 Standard Alert"
    };
    
    println!("  Decision: {}", decision);
    
    println!("\n🎯 KEY INSIGHT:");
    println!("This scenario SHOULD trigger critical alerts!");
    println!("Unlike 8PM front door visitor, this is genuinely threatening:");
    println!("  • 2 AM = suspicious time for any activity");
    println!("  • Back garden = avoiding normal entry points"); 
    println!("  • User home = not expecting anyone");
    println!("  • Unknown person = no legitimate reason to be there");
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

fn logit(p: f64) -> f64 {
    (p / (1.0 - p)).ln()
}

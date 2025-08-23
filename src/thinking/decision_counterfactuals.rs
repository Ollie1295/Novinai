use super::incident_engine::Evidence;

#[derive(Clone, Debug)]
pub struct CounterfactualSuggestion { pub description: String, pub delta_llr: f64 }

pub fn minimal_changes_to_threshold(fused: &Evidence, prior_logit: f64, threshold_logit: f64) -> Vec<CounterfactualSuggestion> {
    let mut candidates = vec![
        CounterfactualSuggestion{ description:"Ring/knock (visitor protocol)".to_string(), delta_llr:-1.2 },
        CounterfactualSuggestion{ description:"Valid delivery/service token".to_string(),    delta_llr:-2.2 },
        CounterfactualSuggestion{ description:"Reduce dwell time below 20s".to_string(),     delta_llr:-0.3 },
        CounterfactualSuggestion{ description:"Approach via public path".to_string(),        delta_llr:-0.6 },
        CounterfactualSuggestion{ description:"Recognized family/guest".to_string(),         delta_llr:-1.8 },
    ];
    let mut logit = prior_logit + fused.sum();
    let mut chosen = Vec::new();
    candidates.sort_by(|a,b| a.delta_llr.partial_cmp(&b.delta_llr).unwrap());
    for c in candidates { if logit <= threshold_logit { break; } logit += c.delta_llr; chosen.push(c); }
    chosen
}

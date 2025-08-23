use super::incident_engine::{Evidence, Incident, sigmoid};

#[derive(Clone, Debug)]
pub enum Question { RequestSecondAngle { cam: String }, AwaitDoorbell, ImproveFaceCapture, CheckDeliveryToken }
#[derive(Clone, Debug)]
pub struct QuestionProposal { pub q: Question, pub expected_entropy_reduction: f64 }

fn entropy(p: f64) -> f64 { if p <= 0.0 || p >= 1.0 { 0.0 } else { -p * p.ln() - (1.0 - p)*(1.0 - p).ln() } }

#[derive(Debug, Clone)]
pub struct ReasonerConfig {
    pub ring_llr: f64, pub token_llr: f64, pub face_gain_llr: f64,
    pub p_ring_given_context: f64, pub p_token_available: f64,
    pub p_second_angle_available: f64, pub p_face_improvable: f64,
}
impl Default for ReasonerConfig {
    fn default() -> Self { Self{ ring_llr:-1.2, token_llr:-2.2, face_gain_llr:-0.6, p_ring_given_context:0.25, p_token_available:0.2, p_second_angle_available:0.6, p_face_improvable:0.5 } }
}

pub fn generate_questions(_inc: &Incident, fused: &Evidence, prior_logit: f64, cfg: &ReasonerConfig) -> Vec<QuestionProposal> {
    let p0 = sigmoid(prior_logit + fused.sum()); let h0 = entropy(p0);
    let mut props = Vec::new();
    // AwaitDoorbell
    { let p_yes = cfg.p_ring_given_context; let p_no = 1.0 - p_yes;
      let p_yes_post = sigmoid(prior_logit + (fused.sum() + cfg.ring_llr));
      let p_no_post = sigmoid(prior_logit + fused.sum());
      let e_h = p_yes*entropy(p_yes_post) + p_no*entropy(p_no_post);
      props.push(QuestionProposal{ q: Question::AwaitDoorbell, expected_entropy_reduction: (h0 - e_h).max(0.0)}); }
    // CheckDeliveryToken
    { let p_yes = cfg.p_token_available; let p_no = 1.0 - p_yes;
      let p_yes_post = sigmoid(prior_logit + (fused.sum() + cfg.token_llr));
      let p_no_post = sigmoid(prior_logit + fused.sum());
      let e_h = p_yes*entropy(p_yes_post) + p_no*entropy(p_no_post);
      props.push(QuestionProposal{ q: Question::CheckDeliveryToken, expected_entropy_reduction: (h0 - e_h).max(0.0)}); }
    // RequestSecondAngle
    { let p_avail = cfg.p_second_angle_available; let p_not = 1.0 - p_avail;
      let p_post_avail = sigmoid(prior_logit + (fused.sum() + cfg.face_gain_llr));
      let p_post_not = sigmoid(prior_logit + fused.sum());
      let e_h = p_avail*entropy(p_post_avail) + p_not*entropy(p_post_not);
      props.push(QuestionProposal{ q: Question::RequestSecondAngle{ cam: "Cam-2".to_string() }, expected_entropy_reduction: (h0 - e_h).max(0.0)}); }
    // ImproveFaceCapture
    { let p_imp = cfg.p_face_improvable; let p_no = 1.0 - p_imp;
      let p_post_imp = sigmoid(prior_logit + (fused.sum() + cfg.face_gain_llr));
      let p_post_no = sigmoid(prior_logit + fused.sum());
      let e_h = p_imp*entropy(p_post_imp) + p_no*entropy(p_post_no);
      props.push(QuestionProposal{ q: Question::ImproveFaceCapture, expected_entropy_reduction: (h0 - e_h).max(0.0)}); }
    props.sort_by(|a,b| b.expected_entropy_reduction.partial_cmp(&a.expected_entropy_reduction).unwrap());
    props
}

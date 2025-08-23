use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct Evidence {
    pub llr_time: f64,
    pub llr_entry: f64,
    pub llr_behavior: f64,
    pub llr_identity: f64,
    pub llr_presence: f64,
    pub llr_token: f64,
}
impl Evidence {
    pub fn sum(&self) -> f64 {
        self.llr_time + self.llr_entry + self.llr_behavior + self.llr_identity + self.llr_presence + self.llr_token
    }
    pub fn capped_sum(&self, pos_cap: f64, neg_cap: f64) -> f64 {
        self.sum().clamp(-neg_cap, pos_cap)
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    pub ts: f64,
    pub cam: String,
    pub person_track: String,
    pub rang_doorbell: bool,
    pub knocked: bool,
    pub dwell_s: f64,
    pub away_prob: f64,
    pub expected_window: bool,
    pub token: Option<String>,
    pub evidence: Evidence,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IncidentStatus { Open, Closed }

#[derive(Clone, Debug)]
pub struct Incident {
    pub id: u64,
    pub started_at: f64,
    pub last_updated: f64,
    pub person_session_id: String,
    pub events: Vec<Event>,
    pub cameras: HashSet<String>,
    pub suppressed_count: u32,
    pub status: IncidentStatus,
}
impl Incident {
    pub fn new(id: u64, start_ts: f64, person_session_id: String) -> Self {
        Self { id, started_at: start_ts, last_updated: start_ts, person_session_id, events: Vec::new(), cameras: HashSet::new(), suppressed_count: 0, status: IncidentStatus::Open }
    }
    pub fn add_event(&mut self, ev: Event) { self.last_updated = ev.ts.max(self.last_updated); self.cameras.insert(ev.cam.clone()); self.events.push(ev); }
    pub fn total_dwell(&self) -> f64 { self.events.iter().map(|e| e.dwell_s).sum() }
    pub fn latest(&self) -> Option<&Event> { self.events.last() }
    pub fn fused_evidence(&self, pos_cap: f64, neg_cap: f64) -> Evidence {
        let mut llr_time: f64 = 0.0; let mut llr_entry: f64 = 0.0; let mut llr_behavior: f64 = 0.0;
        let mut llr_identity: f64 = 0.0; let mut llr_presence: f64 = 0.0; let mut llr_token: f64 = 0.0;
        let n = self.events.len().max(1) as f64;
        for e in &self.events {
            llr_time += e.evidence.llr_time; llr_entry += e.evidence.llr_entry; llr_behavior += e.evidence.llr_behavior;
            if e.evidence.llr_identity.abs() > llr_identity.abs() { llr_identity = e.evidence.llr_identity; }
            if e.evidence.llr_presence.abs() > llr_presence.abs() { llr_presence = e.evidence.llr_presence; }
            if e.evidence.llr_token.abs() > llr_token.abs() { llr_token = e.evidence.llr_token; }
        }
        Evidence {
            llr_time: (llr_time/n).clamp(-neg_cap,pos_cap),
            llr_entry:(llr_entry/n).clamp(-neg_cap,pos_cap),
            llr_behavior:(llr_behavior/n).clamp(-neg_cap,pos_cap),
            llr_identity: llr_identity.clamp(-neg_cap,pos_cap),
            llr_presence: llr_presence.clamp(-neg_cap,pos_cap),
            llr_token: llr_token.clamp(-neg_cap,pos_cap),
        }
    }
}

#[derive(Clone, Debug)]
pub struct IncidentStore { pub incidents: HashMap<(String,String), Incident>, pub ttl_secs: f64, pub id_counter: u64 }
impl IncidentStore {
    pub fn new(ttl_secs: f64) -> Self { Self { incidents: HashMap::new(), ttl_secs, id_counter: 1 } }
    pub fn upsert_event(&mut self, home: &str, ev: Event) -> u64 {
        let key = (home.to_string(), ev.person_track.clone());
        let now = ev.ts;
        self.incidents.retain(|_, inc| now - inc.last_updated <= self.ttl_secs && inc.status==IncidentStatus::Open);
        if let Some(inc) = self.incidents.get_mut(&key) { inc.add_event(ev); inc.id }
        else { let id=self.id_counter; self.id_counter+=1; let mut inc=Incident::new(id, now, key.1.clone()); inc.add_event(ev); self.incidents.insert(key, inc); id }
    }
    pub fn get_incident(&self, home: &str, person_session: &str) -> Option<&Incident> { self.incidents.get(&(home.to_string(), person_session.to_string())) }
    pub fn get_incident_mut(&mut self, home: &str, person_session: &str) -> Option<&mut Incident> { self.incidents.get_mut(&(home.to_string(), person_session.to_string())) }
}

pub fn sigmoid(x: f64) -> f64 { 1.0/(1.0+(-x).exp()) }
pub fn calibrate_logit(raw_logit: f64, mean: f64, temperature: f64, odds_cap: f64) -> f64 {
    let z = (raw_logit - mean) / temperature.max(1.0); sigmoid(z.clamp(-odds_cap, odds_cap))
}

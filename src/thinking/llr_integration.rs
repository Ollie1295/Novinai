//! LLR Evidence Integration Interface

use super::Evidence;
use crate::pipeline::RawEvent;

pub trait LLRExtractor {
    fn extract_evidence(&self, event: &RawEvent) -> Evidence;
    fn extract_time_llr(&self, event: &RawEvent) -> f64;
    fn extract_entry_llr(&self, event: &RawEvent) -> f64;
    fn extract_behavior_llr(&self, event: &RawEvent) -> f64;
    fn extract_identity_llr(&self, event: &RawEvent) -> f64;
    fn extract_presence_llr(&self, event: &RawEvent) -> f64;
    fn extract_token_llr(&self, event: &RawEvent) -> f64;
}

pub struct DemoLLRExtractor {}

impl Default for DemoLLRExtractor {
    fn default() -> Self {
        Self {}
    }
}

impl LLRExtractor for DemoLLRExtractor {
    fn extract_evidence(&self, event: &RawEvent) -> Evidence {
        Evidence {
            llr_time: self.extract_time_llr(event),
            llr_entry: self.extract_entry_llr(event),
            llr_behavior: self.extract_behavior_llr(event),
            llr_identity: self.extract_identity_llr(event),
            llr_presence: self.extract_presence_llr(event),
            llr_token: self.extract_token_llr(event),
        }
    }
    
    fn extract_time_llr(&self, _event: &RawEvent) -> f64 { 0.0 }
    fn extract_entry_llr(&self, _event: &RawEvent) -> f64 { -0.1 }
    fn extract_behavior_llr(&self, _event: &RawEvent) -> f64 { 0.3 }
    fn extract_identity_llr(&self, _event: &RawEvent) -> f64 { 0.2 }
    fn extract_presence_llr(&self, _event: &RawEvent) -> f64 { 0.2 }
    fn extract_token_llr(&self, _event: &RawEvent) -> f64 { 0.0 }
}

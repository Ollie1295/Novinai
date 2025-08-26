use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryTone {
    pub level: String,
}

pub struct OvernightSummaryGenerator;
pub struct MorningSummaryWithDelivery;

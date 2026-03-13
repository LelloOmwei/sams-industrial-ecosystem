// Common trait for logic controllers
pub trait LogicController {
    fn process_atom(&self, atom: SemanticAtom) -> Option<ProcessedSemanticAtom>;
}

// Common types that both implementations will use
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAtom {
    pub id: String,
    pub timestamp: u64,
    pub energy_cost: f64,  // in microjoules (μJ)
    pub trust_pqc: bool,
    pub data: HashMap<String, serde_json::Value>,
    pub tags: Vec<String>,
    pub payload: Option<Vec<u8>>,  // Raw payload for intervention
}

#[derive(Debug, Clone)]
pub struct ProcessedSemanticAtom {
    pub original: SemanticAtom,
    pub processing_time: Duration,
    pub tags_added: Vec<String>,
    pub security_alert: Option<String>,
    pub intervention_applied: bool,
    pub system_health: SystemHealth,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SystemHealth {
    Optimal,
    Warning,
    Critical,
    Intervention,
}

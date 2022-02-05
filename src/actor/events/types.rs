use crate::actor::events::utils::enum_str;
use serde::{Deserialize, Serialize};

// Event types
#[derive(Serialize)]
pub struct NewsResponse {
    pub img: String,
    pub heading: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct SimulatorResponse {
    pub payload: String,
    pub ideal_reproduction_number: f64,
    pub compliance_factor: f64,
    pub recovery_rate: f64,
    pub infection_rate: f64,
}

#[derive(Serialize, Deserialize)]
pub struct WSPayload {
    event_type: String,
    payload: String,
}

enum_str!(
    enum WSResponse {
        // News(NewsEvent),
        Start(SimulatorResponse),
        Control(SimulatorResponse)
        Error(String),
    }
);

#[derive(Deserialize, Clone)]
pub struct CurParams {
    pub susceptible: f64,
    pub exposed: f64,
    pub infectious: f64,
    pub removed: f64,
    pub current_reproduction_number: f64,
    pub ideal_reproduction_number: f64,
    pub compliance_factor: f64,
    pub recovery_rate: f64,
    pub infection_rate: f64,
}

#[derive(Deserialize)]
pub struct ControlMeasure {
    pub level: i32,
    pub cur_date: f64,
    pub name: String,
    pub params: CurParams,
}

#[derive(Deserialize)]
pub struct Event {
    pub level: i32,
    pub cur_date: f64,
    pub name: String,
    pub params: CurParams,
}

#[derive(Deserialize)]
pub struct WSRequest {
    pub kind: String,
    pub payload: String,
}

use crate::actor::events::utils::enum_str;
use diesel::pg::types::sql_types::Jsonb;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
pub struct NewsResponse {
    pub img: String,
    pub heading: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventNews {
    pub announcement: String,
    pub accept: String,
    pub reject: String,
    pub postpone: String,
}

#[derive(Serialize)]
pub struct NewsRequest {
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Read {
    ControlNews(String),
    EventNews(EventNews),
}

#[derive(Serialize)]
pub struct SimulatorResponse {
    pub date: i32,
    pub region: i32,
    pub payload: String,
    pub ideal_reproduction_number: f64,
    pub compliance_factor: f64,
    pub recovery_rate: f64,
    pub infection_rate: f64,
}

#[derive(Serialize)]
pub struct ActionResponse {
    pub simulation_data: SimulatorResponse,
    pub description: String,
    pub is_success: bool,
}

#[derive(Serialize, Deserialize)]
pub struct WSPayload {
    event_type: String,
    payload: String,
}

enum_str!(
    enum WSResponse {
        // News(NewsEvent),
        Seed(String),
        Start(SimulatorResponse),
        Control(ActionResponse),
        Event(ActionResponse),
        EventParams(EventParams),
        Error(String),
        Ok(String),
    }
);

#[derive(AsExpression, FromSqlRow, Serialize, Deserialize, Clone, Debug)]
#[sql_type = "Jsonb"]
pub struct SimulatorParams {
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
pub struct Start {
    pub region: i32,
}

#[derive(Deserialize)]
pub struct Level {
    pub id: i32,
    pub region: i32,
}

#[derive(Deserialize)]
pub struct StartParams {
    pub params: HashMap<String, SimulatorParams>,
}

#[derive(Deserialize, PartialEq)]
#[serde(tag = "action")]
pub enum ControlMeasureAction {
    Apply,
    Remove,
}

#[derive(Deserialize)]
pub struct Save {
    pub cur_date: i32,
    pub region: u32,
    pub params: SimulatorParams,
}

#[derive(Deserialize)]
pub struct ControlMeasure {
    pub level: i32,
    pub cur_date: i32,
    pub name: String,
    pub params: SimulatorParams,
    pub region: u32,
    pub action: ControlMeasureAction,
}

#[derive(Deserialize, PartialEq)]
#[serde(tag = "action")]
pub enum EventAction {
    Request,
    Accept,
    Decline,
    Postpone,
}

#[derive(Deserialize)]
pub struct Event {
    pub cur_date: i32,
    pub id: i32,
    pub params: SimulatorParams,
    pub action: EventAction,
}

#[derive(Deserialize)]
pub struct WSRequest {
    pub kind: String,
    pub region: u32,
    pub payload: String,
}

pub struct Seed {}

#[derive(Serialize, Deserialize)]
pub struct SectionData {
    pub population: f64,
    pub init_params: SimulatorParams,
}

#[derive(Deserialize)]
pub struct ControlMeasureLevel {
    pub params_delta: Vec<f64>,
    pub cost: u32,
}

#[derive(Deserialize)]
pub struct ControlMeasureParams {
    pub description: String,
    pub levels: HashMap<i32, ControlMeasureLevel>,
    pub mess_up_chance: f32,
}

#[derive(Serialize, Deserialize)]
pub struct EventParams {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub params_delta: Vec<f64>,
    pub region: i32,
    pub reward: i32,
}

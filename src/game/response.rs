use crate::db::models::status::ActiveControlMeasures;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct StartLevelRequest {
    pub level: i32,
}

#[derive(Serialize)]
pub struct StartLevelError {
    pub message: String,
}

#[derive(Deserialize)]
pub struct EndLevelRequest {
    pub infected: f64,
    pub removed: f64,
    pub money_spent: i32,
}

#[derive(Serialize)]
pub struct EndLevelResponse {
    pub score: f64,
}

#[derive(Serialize)]
pub struct ActiveControlMeasuresResponse {
    pub num_control_measures: usize,
    pub active_control_measures: HashMap<String, Vec<ActiveControlMeasures>>,
}

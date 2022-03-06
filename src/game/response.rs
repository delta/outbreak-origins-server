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
    pub money_left: f64,
}

#[derive(Serialize)]
pub struct EndLevelResponse {
    pub score: f64,
}

#[derive(Serialize)]
pub struct ActiveControlMeasuresResponse {
    pub active_control_measures: HashMap<String, ActiveControlMeasures>,
}

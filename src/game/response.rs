use crate::db::models::status::ActiveControlMeasures;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct StartLevelRequest {
    pub level: i32,
}

#[derive(Serialize)]
pub struct StartLevelError {
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct EndLevelRequest {
    pub infected: f64,
    pub removed: f64,
    pub money_left: f64,
}

#[derive(Serialize)]
pub struct EndLevelResponse {
    pub message: String,
    pub score: f64,
}

#[derive(Serialize)]
pub struct ActiveControlMeasuresResponse {
    pub active_control_measures: HashMap<String, ActiveControlMeasures>,
}

#[derive(Serialize, Deserialize)]
pub struct EndLevelData {
    pub start_money: f64,
    pub mortality: f64,
}

use crate::db::models::status::ActiveControlMeasures;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct LevelRequest {
    pub level: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct LevelError {
    message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LevelResponse {
    pub cur_level: i32,
    pub retries_left: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DbResponse {
    pub message: String,
}

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

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreResponse {
    pub status: String,
    pub data: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoneyResponse {
    pub status: String,
    pub data: i32,
}
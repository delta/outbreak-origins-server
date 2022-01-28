use crate::db::models;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub status: String,
    pub data: Vec<models::LeaderboardEntry>,
}

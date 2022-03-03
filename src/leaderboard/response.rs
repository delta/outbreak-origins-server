use crate::db::models;
use crate::leaderboard::response::models::LeaderboardEntry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub status: String,
    pub data: Vec<models::LeaderboardEntry>,
    pub user_rank: LeaderboardEntry,
}

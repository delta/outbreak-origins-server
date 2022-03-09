use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct LevelError {
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LevelResponse {
    pub cur_level: i32,
    pub is_randomized: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChangeLevelResponse {
    pub status: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DbResponse {
    pub message: String,
}

use serde::{Deserialize, Serialize};

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
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DbResponse {
    pub message: String,
}

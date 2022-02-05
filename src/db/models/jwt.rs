use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
    pub id: i32,
    pub email: String,
    pub level: i32,
    pub created_at: usize,
    pub exp: usize,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
    pub types: String,
    pub email: String,
    pub created_at: usize,
    pub exp: usize,
}

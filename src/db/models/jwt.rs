use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
    pub kind: String,
    pub email: String,
    pub name: String,
    pub created_at: usize,
    pub exp: usize,
}

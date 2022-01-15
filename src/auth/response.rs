use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AuthResult {
    pub is_verified: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogoutResult {
    pub status: bool,
}

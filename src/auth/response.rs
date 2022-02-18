use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AuthResult {
    pub is_verified: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogoutResult {
    pub is_logged_out: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckAuthResult {
    pub status: bool,
    pub email: Option<String>,
}

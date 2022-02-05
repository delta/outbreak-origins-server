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

#[derive(Debug, Clone, Serialize)]
pub struct CheckAuthResult {
    pub status: bool,
    pub level: Option<i32>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResetPasswordResult {
    pub status: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenValidateResult {
    pub status: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChangePasswordResult {
    pub message: String,
    pub status: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerifyUserResult {
    pub status: bool,
    pub message: String,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub name: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetToken {
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChangePassword {
    pub jwt: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResetPasswordEmail {
    pub email: String,
}

#[derive(Deserialize)]
pub struct UserVerify {
    pub jwt: String,
}

#[derive(Deserialize)]
pub struct ResendVerification {
    pub email: String,
}

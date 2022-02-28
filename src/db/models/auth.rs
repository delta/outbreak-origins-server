use serde::Deserialize;

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

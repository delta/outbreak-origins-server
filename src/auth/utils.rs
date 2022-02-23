use crate::db::models::Claims;
use chrono::{Duration, Utc};
use dotenv::dotenv;
use jsonwebtoken::{
    decode, encode, errors::Result, Algorithm, DecodingKey, EncodingKey, Header, TokenData,
    Validation,
};
use sendgrid::{Destination, Mail, SGClient};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize)]
pub struct UserVerify {
    pub email: String,
    pub token: String,
}

// Result is from jsonwebtoken error
pub fn create_jwt(types: String, email: String, created_at: Option<usize>) -> Result<String> {
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET");
    let expiry = std::env::var("EXPIRY")
        .expect("EXPIRY")
        .parse::<i64>()
        .expect("Needed a number");
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(expiry))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        types,
        email,
        created_at: if let Some(c) = created_at {
            c
        } else {
            Utc::now().timestamp() as usize
        },
        exp: expiration as usize,
    };
    let header = Header::new(Algorithm::HS512);
    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
}

pub fn get_info_token(token: String) -> Result<TokenData<Claims>> {
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET");
    let token_message = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::new(Algorithm::HS512),
    );
    token_message
}

pub fn verify_user(token: String, email: &str, name: &str) {
    dotenv().expect("Can't load environment variables");

    let api_key = env::var("SENDGRID_API_KEY").expect("SENDGRID_API_KEY must be set");
    let from_mail = env::var("SENDGRID_VERIFIED_MAIL").expect("SENDGRID_VERIFIED_MAIL must be set");

    let link = format!(
        "http://{}/auth/user/verify?token={}&email={}",
        env::var("APP_URL").expect("APP_URL must be set"),
        token,
        email
    );

    let msg = format!("<h1>Click this link</h1>\n<p>Greetings from outbreak origins, use this <a href {}>link</a> to verify your email and start playing</p>", link);

    let mail: Mail = Mail::new()
        .add_to(Destination {
            address: email,
            name: name,
        })
        .add_from(from_mail.as_str())
        .add_subject("Verify your account")
        .add_html(msg.as_str());

    let sgc = SGClient::new(api_key);

    SGClient::send(&sgc, mail).expect("Failed to send email");
}

pub fn gen_token() -> String {
    use rand::Rng;

    let mut str = String::new();

    for (_i, _) in (0..32).enumerate() {
        str.push(rand::thread_rng().gen_range(33_u8..126_u8) as char);
    }

    str
}

#[allow(dead_code)]
pub fn reset_password_mail(email: &str, name: &str) {
    dotenv().expect("Can't load environment variables");

    let api_key = env::var("SENDGRID_API_KEY").expect("SENDGRID_API_KEY must be set");
    let from_mail = env::var("SENDGRID_VERIFIED_MAIL").expect("SENDGRID_VERIFIED_MAIL must be set");

    let reset_jwt = create_jwt("reset".to_string(), email.to_string(), None);

    let link = format!(
        "http://{}/auth/user/reset_password?email={}&jwt={}",
        env::var("APP_URL").expect("APP_URL must be set"),
        email,
        reset_jwt.unwrap()
    );

    let msg = format!("<h1>Click this link</h1>\n<p>Greetings from outbreak origins, use this <a href {}>link</a> to reset your password</p>", link);

    let mail: Mail = Mail::new()
        .add_to(Destination {
            address: email,
            name,
        })
        .add_from(from_mail.as_str())
        .add_subject("Reset your password")
        .add_html(msg.as_str());

    let sgc = SGClient::new(api_key);

    SGClient::send(&sgc, mail).expect("Failed to send email");
}

use crate::db::models::Claims;
use chrono::{Duration, Utc};
use dotenv::dotenv;
use jsonwebtoken::{
    decode, encode, errors::Result, Algorithm, DecodingKey, EncodingKey, Header, TokenData,
    Validation,
};
use sendgrid::{Destination, Mail, SGClient};
use std::env;

// Result is from jsonwebtoken error
pub fn create_jwt(id: i32, email: String, created_at: Option<usize>) -> Result<String> {
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
        id,
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

pub fn send_mail() {
    dotenv().expect("Can't load environment variables");

    let api_key = env::var("SENDGRID_API_KEY").expect("SENDGRID_API_KEY must be set");

    let mail: Mail = Mail::new()
        .add_to(Destination {
            address: "mukundh.srivathsan.nitt@gmail.com",
            name: "Mukundh",
        })
        .add_from("mukundhsrivathsan@gmail.com")
        .add_subject("Hello World!")
        .add_html("<h1>Hello World!</h1>");

    let sgc = SGClient::new(api_key);

    SGClient::send(&sgc, mail).expect("Failed to send email");
}

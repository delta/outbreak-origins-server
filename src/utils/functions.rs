use crate::models::Claims;
use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, errors::Result, Algorithm, DecodingKey, EncodingKey, Header, TokenData,
    Validation,
};

// Result is from jsonwebtoken error
pub fn create_jwt(id: i32, username: String) -> Result<String> {
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
        username,
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

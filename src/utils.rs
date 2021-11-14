use crate::models::{Claims, Error, Result};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

pub fn create_jwt(id: i32, username: String) -> Result<String> {
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET");
    // let expiration = Utc::now()
    //     .checked_add_signed(chrono::Duration::seconds(expiry_date))
    //     .expect("valid timestamp")
    //     .timestamp();

    let claims = Claims {
        id,
        username,
        // exp: expiration as usize,
    };
    let header = Header::new(Algorithm::HS512);
    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(|_| Error::JWTTokenCreationError)
}

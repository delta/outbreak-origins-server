use crate::db::models::Claims;
use chrono::{Duration, Utc};
use dotenv::dotenv;
use jsonwebtoken::{
    decode, encode, errors, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use sendgrid::{Destination, Mail, SGClient};
use std::env;

// Result is from jsonwebtoken error
pub fn create_jwt(
    kind: String,
    email: String,
    name: String,
    created_at: Option<usize>,
    time: i64,
) -> errors::Result<String> {
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET");
    let expiration = Utc::now()
        .checked_add_signed(Duration::minutes(time))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        kind,
        email,
        name,
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

pub fn get_info_token(token: &str) -> errors::Result<TokenData<Claims>> {
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET");
    let token_message = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::new(Algorithm::HS512),
    );
    token_message
}

pub fn send_verify_email(email: &str, name: &str) -> Result<String, String> {
    dotenv().expect("Can't load environment variables");

    let api_key = env::var("SENDGRID_API_KEY").expect("SENDGRID_API_KEY must be set");
    let from_mail =
        env::var("SENDGRID_VERIFIED_EMAIL").expect("SENDGRID_VERIFIED_EMAIL must be set");

    let email_expiry = env::var("EXPIRY_EMAIL")
        .expect("EXPIRY_EMAIL must be set")
        .parse::<i64>()
        .expect("Should be a number");

    let verify_jwt = create_jwt(
        "Verify".to_string(),
        email.to_string(),
        name.to_string(),
        None,
        email_expiry,
    )
    .unwrap();

    let link = format!(
        "{}/verifyemail/{}",
        env::var("FRONTEND_APP_URL").expect("FRONTEND_APP_URL must be set"),
        verify_jwt
    );

    let msg = format!("<h1>Click this link</h1>\n<p>Greetings from outbreak origins, use this <a href={}>link</a> to verify your email and start playing</p>", link);

    let mail: Mail = Mail::new()
        .add_to(Destination {
            address: email,
            name,
        })
        .add_from(from_mail.as_str())
        .add_subject("Verify your account")
        .add_html(msg.as_str());

    let sgc = SGClient::new(api_key);

    match SGClient::send(&sgc, mail) {
        Ok(_) => Ok(String::from("Email sent successfully")),
        Err(x) => {
            println!("{}", x);
            Err(String::from("Couldn't send email"))
        }
    }
}

pub fn send_reset_password_mail(name: &str, email: &str) -> Result<String, String> {
    dotenv().expect("Can't load environment variables");

    let api_key = env::var("SENDGRID_API_KEY").expect("SENDGRID_API_KEY must be set");
    let from_mail =
        env::var("SENDGRID_VERIFIED_EMAIL").expect("SENDGRID_VERIFIED_EMAIL must be set");
    let email_expiry = env::var("EXPIRY_EMAIL")
        .expect("EXPIRY_EMAIL must be set")
        .parse::<i64>()
        .expect("Should be a number");

    let reset_jwt = create_jwt(
        "Reset".to_string(),
        email.to_string(),
        name.to_string(),
        None,
        email_expiry,
    );

    let link = format!(
        "{}/resetpassword/{}",
        env::var("FRONTEND_APP_URL").expect("FRONTEND_APP_URL must be set"),
        reset_jwt.unwrap()
    );
    let msg = format!("<h1>Reset Password</h1>\n<p>Greetings from outbreak origins, use this <a href={} > link</a> to reset your password</p>", link);

    let mail: Mail = Mail::new()
        .add_to(Destination {
            address: email,
            name,
        })
        .add_from(from_mail.as_str())
        .add_subject("Reset your password")
        .add_html(msg.as_str());

    let sgc = SGClient::new(api_key);

    match SGClient::send(&sgc, mail) {
        Ok(_) => Ok(String::from("Email sent successfully")),
        Err(x) => {
            println!("{}", x);
            Err(String::from("Couldn't send email"))
        }
    }
}

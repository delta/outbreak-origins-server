use diesel::prelude::*;

use crate::models;
use crate::utils::create_jwt;
use bcrypt::{hash, verify, DEFAULT_COST};
// use std::{fs::File, io::Read};
use serde::Deserialize;
use std::env;
use std::error::Error;
// use oauth2::reqwest::http_client;
// use oauth2::{
//     AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
//     RevocationUrl, Scope, TokenUrl,
// };

type DbError = Box<dyn std::error::Error + Send + Sync>;

// #[derive(Deserialize, Debug)]
// struct GoogleCreds {
//     client_id: String,
//     client_secret: String,
// }

pub fn find_user_by_uid(conn: &PgConnection) -> Result<Option<models::User>, DbError> {
    use crate::schema::users::dsl::*;

    let user_res = users.first::<models::User>(conn).optional()?;

    Ok(user_res)
}

pub fn insert_new_user(
    fusername: &str,
    fpassword: &str,
    conn: &PgConnection,
) -> Result<(), DbError> {
    use crate::schema::users::dsl::*;
    println!("In here");
    let new_user = models::NewUser {
        username: fusername.to_owned(),
        password: hash(fpassword.to_owned(), DEFAULT_COST)?,
    };

    diesel::insert_into(users).values(&new_user).execute(conn)?;
    Ok(())
}

pub fn verify_user_by_username(
    fusername: &str,
    fpassword: &str,
    conn: &PgConnection,
) -> Result<(bool, String, String), DbError> {
    use crate::schema::users::dsl::*;
    let user = users
        .filter(username.eq(fusername))
        .first::<models::User>(conn)
        .optional()?;
    let is_verified = match user {
        Some(u) => match u.password {
            Some(p) => {
                if verify(fpassword.to_owned(), &p)? {
                    (
                        true,
                        create_jwt(u.id, u.username)?,
                        String::from("Successfully authenticated"),
                    )
                } else {
                    (false, String::new(), String::from("Wrong Password"))
                }
            }
            None => (false, String::new(), String::from("Password doesn't exist")),
        },
        None => (false, String::new(), String::from("User doesn't exist")),
    };
    Ok(is_verified)
}

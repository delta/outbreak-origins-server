use diesel::prelude::*;

use crate::auth::utils::{create_jwt, gen_token, verify_user};
use crate::db::models;
use crate::db::types::DbError;
use bcrypt::{hash, verify, DEFAULT_COST};

pub fn insert_new_user(
    ffirstname: &str,
    flastname: &str,
    fpassword: &str,
    femail: &str,
    conn: &PgConnection,
) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;
    let t = gen_token();
    let new_user = models::NewUser {
        firstname: ffirstname.to_owned(),
        lastname: flastname.to_owned(),
        password: hash(fpassword.to_owned(), DEFAULT_COST)?,
        email: femail.to_owned(),
        token: t.to_owned(),
    };
    diesel::insert_into(users).values(&new_user).execute(conn)?;
    verify_user(t, femail, ffirstname);
    Ok(())
}

pub fn verify_user_by_email(
    femail: &str,
    fpassword: &str,
    conn: &PgConnection,
) -> Result<(bool, String, String), DbError> {
    use crate::db::schema::users::dsl::*;
    let user = users
        .filter(email.eq(femail))
        .first::<models::User>(conn)
        .optional()?;
    let is_verified = match user {
        Some(u) => match u.password {
            Some(p) => {
                if verify(fpassword.to_owned(), &p)? {
                    (
                        true,
                        create_jwt(u.id.to_string(), u.email, None)?,
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

pub fn verify_user_by_token(
    femail: &str,
    ftoken: String,
    conn: &PgConnection,
) -> Result<(bool, String), DbError> {
    use crate::db::schema::users::dsl::*;
    let user = users
        .filter(email.eq(femail))
        .first::<models::User>(conn)
        .optional()?;
    let is_verified = match user {
        Some(u) => {
            if u.token == ftoken {
                let verified = diesel::update(users.filter(email.eq(femail)))
                    .set(is_email_verified.eq(true))
                    .execute(conn)?;
                println!("{}", verified);
                (
                    true,
                    String::from("Successfully authenticated"),
                )
            } else {
                (false, String::from("Wrong Token"))
            }
        }
        None => (false, String::from("User doesn't exist")),
    };
    Ok(is_verified)
}

pub fn reset_password(
    femail: &str,
    ftoken: String,
    conn: &PgConnection,
) -> Result<(bool, String), DbError> {
    use crate::db::schema::users::dsl::*;
    let user = users
        .filter(email.eq(femail))
        .first::<models::User>(conn)
        .optional()?;
    let is_reset = match user {
        Some(u) => {
            if u.token == ftoken {
                (
                    true,
                    String::from("valid JWT"),
                )
            } else {
                (false, String::from("Wrong JWT"))
            }
        }
        None => (false, String::from("User doesn't exist")),
    };
    Ok(is_reset)
}
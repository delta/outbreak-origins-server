use diesel::prelude::*;

use crate::auth::utils::{create_jwt, send_verify_email};
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
    let new_user = models::NewUser {
        firstname: ffirstname.to_owned(),
        lastname: flastname.to_owned(),
        password: hash(fpassword.to_owned(), DEFAULT_COST)?,
        email: femail.to_owned(),
    };
    diesel::insert_into(users).values(&new_user).execute(conn)?;
    send_verify_email(femail, ffirstname).unwrap();
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
        Some(u) => {
            if !u.is_email_verified {
                return Ok((false, String::new(), String::from("User not verified")));
            }
            match u.password {
                Some(p) => {
                    if verify(fpassword.to_owned(), &p)? {
                        let expiry = std::env::var("EXPIRY")
                            .expect("EXPIRY")
                            .parse::<i64>()
                            .expect("Needed a number");
                        (
                            true,
                            create_jwt(String::from("Login"), u.email, u.firstname, None, expiry)?,
                            String::from("Successfully authenticated"),
                        )
                    } else {
                        (false, String::new(), String::from("Wrong Password"))
                    }
                }
                None => (false, String::new(), String::from("Password doesn't exist")),
            }
        }
        None => (false, String::new(), String::from("User doesn't exist")),
    };
    Ok(is_verified)
}

pub fn verify_user_by_token(femail: &str, conn: &PgConnection) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;
    diesel::update(users.filter(email.eq(femail)))
        .set(is_email_verified.eq(true))
        .execute(conn)?;
    Ok(())
}

pub fn reset_password(femail: &str, fpassword: &str, conn: &PgConnection) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;
    diesel::update(users.filter(email.eq(femail)))
        .set(password.eq(hash(fpassword.to_owned(), DEFAULT_COST).unwrap()))
        .execute(conn)?;
    Ok(())
}

pub fn get_user_email(femail: &str, conn: &PgConnection) -> Result<Option<String>, DbError> {
    use crate::db::schema::users::dsl::*;
    let user = users
        .filter(email.eq(femail))
        .first::<models::User>(conn)
        .optional()?;
    let name = match user {
        Some(u) => Some(u.firstname),
        None => None,
    };
    Ok(name)
}

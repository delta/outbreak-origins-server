use diesel::prelude::*;

use crate::auth::utils::create_jwt;
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
        score: 0,
        money: 0,
    };

    diesel::insert_into(users).values(&new_user).execute(conn)?;
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
                        create_jwt(u.id, u.email, u.curlevel, None)?,
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

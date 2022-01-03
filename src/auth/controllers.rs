use diesel::prelude::*;

use crate::models;
use crate::utils::{functions::create_jwt, types::DbError};
use bcrypt::{hash, verify, DEFAULT_COST};

pub fn insert_new_user(
    fusername: &str,
    fpassword: &str,
    femail: &str,
    conn: &PgConnection,
) -> Result<(), DbError> {
    use crate::schema::users::dsl::*;
    let new_user = models::NewUser {
        username: fusername.to_owned(),
        password: hash(fpassword.to_owned(), DEFAULT_COST)?,
        email: femail.to_owned(),
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

use crate::auth::extractors::Authenticated;
use crate::db::{models, types::DbError};
use diesel::prelude::*;

pub fn get_current_level(
    conn: &PgConnection,
    user: Authenticated,
) -> Result<(i32, bool, bool), DbError> {
    use crate::db::schema::users::dsl::*;
    let user_email = user.0.as_ref().map(|y| y.email.clone());

    let user_result = &users
        .filter(email.eq(user_email.unwrap()))
        .load::<models::User>(conn)?[0];

    Ok((
        user_result.curlevel,
        user_result.is_randomized,
        user_result.is_active,
    ))
}

pub fn change_level_type(
    conn: &PgConnection,
    user: Authenticated,
    level_type: bool,
) -> Result<bool, DbError> {
    use crate::db::schema::users;
    let user_email = user.0.unwrap().email;
    let user_result = &users::table
        .filter(users::email.eq(user_email.clone()))
        .load::<models::User>(conn)?[0];
    let money = if level_type { 600 } else { 500 };
    if user_result.is_level_active {
        Ok(false)
    } else {
        diesel::update(users::table.filter(users::email.eq(user_email)))
            .set((
                users::is_level_active.eq(true),
                users::is_randomized.eq(level_type),
                users::money.eq(money),
            ))
            .execute(conn)?;
        Ok(true)
    }
}

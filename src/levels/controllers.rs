use crate::auth::extractors::Authenticated;
use crate::db::models;
use crate::levels::response;
use diesel::prelude::*;
use crate::db::types::DbError;

pub fn update_user_at_level_end(
    conn: &PgConnection,
    user: Authenticated,
    user_score: i32,
) -> Result<response::DbResponse, DbError> {
    use crate::db::schema::users::dsl::*;
    let user_email = user.0.as_ref().map(|y| y.email.clone());
    match diesel::update(users.filter(email.eq(user_email.unwrap())))
        .set((
            money.eq(1000),
            score.eq(user_score),
            retryattemptsleft.eq(retryattemptsleft - 1),
        ))
        .execute(conn)
    {
        Ok(_) => Ok(response::DbResponse {
            message: "Updated user".to_string(),
        }),
        Err(e) => Err(DbError::from(e)),
    }
}

pub fn get_current_level(conn: &PgConnection, user: &Authenticated) -> i32 {
    use crate::db::schema::users::dsl::*;
    let user_email = (*user).0.as_ref().map(|y| y.email.clone());

    let user_result: i32 = users
        .filter(email.eq(user_email.unwrap()))
        .load::<models::User>(conn)
        .unwrap()[0]
        .curlevel;

    user_result
}

pub fn get_retry_attempts(conn: &PgConnection, user: &Authenticated) -> i32 {
    use crate::db::schema::users::dsl::*;
    let user_email = (*user).0.as_ref().map(|y| y.email.clone());

    let user_result: i32 = users
        .filter(email.eq(user_email.unwrap()))
        .load::<models::User>(conn)
        .unwrap()[0]
        .retryattemptsleft;

    user_result
}

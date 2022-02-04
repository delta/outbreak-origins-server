use crate::auth::extractors::Authenticated;
use crate::db::models;
use crate::db::types::DbError;
use crate::levels::response;
use diesel::prelude::*;

pub fn update_current_level(
    conn: &PgConnection,
    level: i32,
    user: Authenticated,
) -> Result<response::DbResponse, DbError> {
    use crate::db::schema::users::dsl::*;
    let user_email = user.0.as_ref().map(|y| y.email.clone());

    let user_result: QueryResult<models::User> =
        diesel::update(users.filter(email.eq(user_email.unwrap())))
            .set(curlevel.eq(level))
            .get_result(conn);
    match user_result {
        Ok(user) => Ok(response::DbResponse {
            message: format!("User {} updated to level {}", user.email, user.curlevel),
        }),
        Err(e) => Err(DbError::from(e)),
    }
}

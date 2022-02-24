use crate::auth::extractors::Authenticated;
use crate::db::models;
use crate::db::types::DbError;
use crate::levels::response;
use diesel::prelude::*;

#[allow(dead_code)]
pub fn update_current_level(
    conn: &PgConnection,
    user: Authenticated,
) -> Result<response::DbResponse, DbError> {
    use crate::db::schema::users::dsl::*;
    let user_email = user.0.as_ref().map(|y| y.email.clone());

    let user_result: QueryResult<models::User> =
        diesel::update(users.filter(email.eq(user_email.unwrap())))
            .set(curlevel.eq(curlevel + 1))
            .get_result(conn);
    match user_result {
        Ok(user) => Ok(response::DbResponse {
            message: format!("User {} updated to level {}", user.email, user.curlevel),
        }),
        Err(e) => Err(DbError::from(e)),
    }
}

pub fn get_current_level(conn: &PgConnection, user: Authenticated) -> i32 {
    use crate::db::schema::users::dsl::*;
    let user_email = user.0.as_ref().map(|y| y.email.clone());
    println!("{:?}", user_email);

    let user_result: i32 = users
        .filter(email.eq(user_email.unwrap()))
        .load::<models::User>(conn)
        .unwrap()[0]
        .curlevel;

    user_result
}

use crate::auth::extractors::Authenticated;
use crate::db::models;
use diesel::prelude::*;

pub fn get_current_level(conn: &PgConnection, user: Authenticated) -> (i32, bool) {
    use crate::db::schema::users::dsl::*;
    let user_email = user.0.as_ref().map(|y| y.email.clone());

    let user_result = &users
        .filter(email.eq(user_email.unwrap()))
        .load::<models::User>(conn)
        .unwrap()[0];

    (user_result.curlevel, user_result.is_randomized)
}

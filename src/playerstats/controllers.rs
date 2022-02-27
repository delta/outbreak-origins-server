use crate::db::models;
use crate::db::types::DbError;
use diesel::prelude::*;

pub fn get_score(conn: &PgConnection, useremail: String) -> Result<i32, DbError> {
    use crate::db::schema::users::dsl::*;

    let score_data = users
        .filter(email.eq(useremail))
        .load::<models::User>(conn)
        .unwrap()[0]
        .score;

    Ok(score_data)
}

pub fn get_money(conn: &PgConnection, useremail: String) -> Result<i32, DbError> {
    use crate::db::schema::users::dsl::*;

    let money_data = users
        .filter(email.eq(useremail))
        .load::<models::User>(conn)
        .unwrap()[0]
        .money;

    Ok(money_data)
}
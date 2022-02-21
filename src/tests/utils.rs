use crate::db::models;
use crate::db::types::DbError;

use diesel::pg::PgConnection;
use diesel::prelude::*;

#[allow(dead_code)]
pub fn insert_test_user(usr: &models::TestUser, conn: &PgConnection) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;
    diesel::insert_into(users).values(usr).execute(conn)?;
    Ok(())
}

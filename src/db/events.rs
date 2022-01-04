use crate::db::models;

use diesel::prelude::*;

type DbError = Box<dyn std::error::Error + Send + Sync>;

pub fn find_event_by_id(conn: &PgConnection, id: i32) -> Result<Option<models::Event>, DbError> {
    use crate::db::schema::events::dsl::*;

    let event_res = events.find(id).first(conn).optional()?;
    Ok(event_res)
}
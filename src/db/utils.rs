use crate::db::models;
use crate::db::types::DbError;
use crate::db::types::PgPool;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use r2d2::Pool;
pub use r2d2_diesel::ConnectionManager;
use std::env;

pub fn create_db_pool() -> PgPool {
    dotenv().expect("Can't load environment variables");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool")
}

pub fn find_event_by_id(
    event_id: i32,
    conn: &PgConnection,
) -> Result<Option<models::Event>, DbError> {
    use crate::db::schema::events::dsl::*;

    let event_res = events.find(event_id).first(conn).optional()?;
    Ok(event_res)
}

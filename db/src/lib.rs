#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod models;
pub mod schema;

use diesel::prelude::*;
use std::error::Error;

use diesel::pg::PgConnection;
use dotenv::dotenv;
use r2d2::{Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;
use std::env;

type DbError = Box<dyn std::error::Error + Send + Sync>;
pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub fn create_db_pool() -> PgPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool")
}

pub fn find_event_by_id(conn: &PgConnection, id: i32) -> Result<Option<models::Event>, DbError> {
    use crate::schema::events::dsl::*;

    let event_res = events.find(id).first(conn).optional()?;
    Ok(event_res)
}

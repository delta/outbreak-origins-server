pub mod events;
pub mod models;
pub mod schema;

// use std::error::Error;

use diesel::pg::PgConnection;
use dotenv::dotenv;
use r2d2::{Pool, PooledConnection};
pub use r2d2_diesel::ConnectionManager;
use std::env;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub fn create_db_pool() -> PgPool {
    dotenv().expect("Can't load environment variables");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool")
}

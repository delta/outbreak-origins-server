use crate::db::models;
use crate::db::types::DbError;
use crate::db::types::PgPool;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use r2d2::Pool;
pub use r2d2_diesel::ConnectionManager;
use std::env;

pub fn create_db_pool() -> PgPool {
    dotenv().expect("Can't load environment variables");
    let db_base_url = env::var("DB_BASE_URL").expect("DB_BASE_URL must be set");
    let db_name = env::var("DB_NAME").expect("DB_NAME must be set");
    let manager = ConnectionManager::<PgConnection>::new(&format!("{}/{}", db_base_url, db_name));
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool")
}

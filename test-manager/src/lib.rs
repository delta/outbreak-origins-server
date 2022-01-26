#![crate_name = "test_manager"]
#[macro_use]
extern crate diesel_migrations;
extern crate dotenv;

use diesel::prelude::*;
use diesel::PgConnection;
use diesel_migrations::embed_migrations;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

embed_migrations!();

pub struct TestDbManager {
    pub conn_pool: Pool<ConnectionManager<PgConnection>>,
}

impl TestDbManager {
    pub fn new() -> Self {
        let db_url = dotenv::var("TEST_DB_URL").expect("TEST_DB_URL must be set");
        let conn_manager = ConnectionManager::<PgConnection>::new(db_url);
        let conn_pool = Pool::builder()
            .build(conn_manager)
            .expect("Failed to create test pool");

        let conn = conn_pool.get().unwrap();

        embedded_migrations::run(&*conn).expect("Couldn't run migrations on Test DB");

        Self { conn_pool }
    }
}

impl Drop for TestDbManager {
    fn drop(&mut self) {
        let conn = self.conn_pool.get().unwrap();

        let table_list = dotenv::var("TABLES").expect("TABLES must be set");

        let query = diesel::sql_query(format!("DROP TABLE {} CASCADE", table_list).as_str());
        query.execute(&*conn).expect("Couldn't delete tables");
    }
}

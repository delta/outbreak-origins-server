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
        let db_url = dotenv::var("DB_BASE_URL").expect("DB_BASE_URL must be set");
        let conn = PgConnection::establish(&format!("{}/postgres", db_url))
            .expect("Could not connect to postgres database");
        let test_db_name = dotenv::var("TEST_DB_NAME").expect("TEST_DB_NAME must be set");
        let query = diesel::sql_query(format!("CREATE DATABASE {}", test_db_name));
        query
            .execute(&conn)
            .expect(&format!("Could not create database {}", test_db_name));

        let conn_manager =
            ConnectionManager::<PgConnection>::new(format!("{}/{}", db_url, test_db_name));
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
        let db_url = dotenv::var("DB_BASE_URL").expect("DB_BASE_URL must be set");
        let conn = PgConnection::establish(&format!("{}/postgres", db_url))
            .expect("Could not connect to postgres database");
        let test_db_name = dotenv::var("TEST_DB_NAME").expect("TEST_DB_NAME must be set");
        let query = diesel::sql_query(format!("DROP DATABASE {} WITH (FORCE)", test_db_name));
        query
            .execute(&conn)
            .expect(&format!("Could drop create database {}", test_db_name));
    }
}

#[macro_use]
extern crate diesel;
extern crate dotenv;

use crate::db::types::PgPool;
use crate::db::utils::create_db_pool;
use actix_files as fs;
use actix_identity::IdentityService;
use actix_web::middleware as mw;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use diesel::PgConnection;

use dotenv::dotenv;

mod actor;
mod auth;
mod db;
mod leaderboard;
mod middleware;
mod tests;
use crate::middleware as common_middleware;

#[macro_use]
extern crate diesel_migrations;
use diesel_migrations::embed_migrations;

embed_migrations!("migrations/");
use diesel::prelude::*;

use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

pub struct TestDbManager {
    conn_pool: Pool<ConnectionManager<PgConnection>>,
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

        Self {
            conn_pool: conn_pool,
        }
    }
}

impl Drop for TestDbManager {
    fn drop(&mut self) {
        let conn = self.conn_pool.get().unwrap();

        let table_list = vec![
            "events",
            "regions",
            "status",
            "users",
            "__diesel_schema_migrations",
        ];
        let all_tables = table_list.join(", ");

        let query = diesel::sql_query(format!("DROP TABLE {} CASCADE", all_tables).as_str());
        query
            .execute(&*conn)
            .expect("Couldn't delete tables");
    }
}

pub async fn ws_index(
    r: HttpRequest,
    stream: web::Payload,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    println! {"{:?}",r};
    let res = actor::ws::start(actor::Game::new(pool), &r, stream);
    println!("{:?}", res);
    res
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let res = dotenv();
    if res.is_err() {
        std::process::exit(1)
    }

    let pool = create_db_pool();
    let app_url = dotenv::var("APP_URL").unwrap();

    HttpServer::new(move || {
        App::new()
            // set up DB pool to be used with web::Data<Pool> extractor
            .data(pool.clone())
            .wrap(auth::middleware::CheckAuth {})
            .wrap(IdentityService::new(auth::middleware::cookie_policy()))
            .wrap(common_middleware::cors_config())
            .wrap(mw::Logger::default())
            .service(web::resource("/ws/").route(web::get().to(ws_index)))
            .service(fs::Files::new("/events", "static/").index_file("index.html"))
            .configure(auth::routes::auth_routes)
    })
    .bind(&app_url)?
    .run()
    .await
}

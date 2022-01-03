#[macro_use]
extern crate diesel;

use actix_web::{App, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;

mod auth;
mod models;
mod schema;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let res = dotenv();
    if res.is_err() {
        std::process::exit(1)
    }

    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<PgConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");
    let bind = "127.0.0.1:8081";
    HttpServer::new(move || {
        App::new()
            // set up DB pool to be used with web::Data<Pool> extractor
            .data(pool.clone())
            .configure(auth::routes::auth_routes)
    })
    .bind(&bind)?
    .run()
    .await
}

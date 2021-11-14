#[macro_use]
extern crate diesel;

use actix_cors::Cors;
use actix_web::{http, App, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use serde_json;

mod actions;
mod models;
mod routes;
mod schema;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<PgConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");
    let bind = "127.0.0.1:8081";
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8080")
            .allowed_methods(vec!["POST"])
            .allowed_headers(vec![
                http::header::CONTENT_TYPE,
                http::header::ACCESS_CONTROL_ALLOW_HEADERS,
                http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
                http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            ])
            // .expose_headers(vec![
            //     http::header::SET_COOKIE,
            //     http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            // ])
            .supports_credentials();
        // let cors = Cors::permissive();
        App::new()
            // set up DB pool to be used with web::Data<Pool> extractor
            .data(pool.clone())
            // .wrap(middleware::Logger::default())
            .service(routes::get_user)
            .wrap(cors)
            .service(routes::register_user)
            .service(routes::login_user)
        // .service(routes::google_auth)
    })
    .bind(&bind)?
    .run()
    .await
}

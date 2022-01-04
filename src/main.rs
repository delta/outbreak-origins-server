#[macro_use]
extern crate diesel;
extern crate dotenv;

use actix_identity::IdentityService;
use actix_web::{App, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;

mod auth;
mod middleware;
mod models;
mod schema;
mod utils;
use crate::middleware as common_middleware;

pub async fn ws_index(
    r: HttpRequest,
    stream: web::Payload,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    println! {"{:?}",r};
    let res = ws::start(Game::new(pool), &r, stream);
    println!("{:?}", res);
    res
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let res = dotenv();
    if res.is_err() {
        std::process::exit(1)
    }

    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<PgConnection>::new(connspec);
    let pool = create_db_pool();
    let bind = "127.0.0.1:8081";
    HttpServer::new(move || {
        App::new()
            // set up DB pool to be used with web::Data<Pool> extractor
            .data(pool.clone())
            .wrap(auth::middleware::CheckAuth {})
            .wrap(IdentityService::new(auth::middleware::cookie_policy()))
            .wrap(common_middleware::cors_config())
            .wrap(middleware::Logger::default())
            .service(web::resource("/ws/").route(web::get().to(ws_index)))
            .service(fs::Files::new("/", "static/").index_file("index.html"))
            .configure(auth::routes::auth_routes)
    })
    .bind(&bind)?
    .run()
    .await


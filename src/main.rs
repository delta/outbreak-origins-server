#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod actor;
pub mod db;
use crate::actor::*;
use crate::db::*;
use actix_files as fs;

use actix_web::{web,Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use actix_web::{middleware,App, HttpServer};

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
    let pool = create_db_pool();

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource("/ws/").route(web::get().to(ws_index)))
            .service(fs::Files::new("/", "static/").index_file("index.html"))
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}

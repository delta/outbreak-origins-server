#[macro_use]
extern crate diesel;
extern crate dotenv;

use crate::db::types::PgPool;
use crate::db::utils::create_db_pool;
use actix_files as fs;
use actix_identity::IdentityService;
use actix_web::middleware as mw;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};

use dotenv::dotenv;

mod actor;
mod auth;
mod db;
mod game;
mod leaderboard;
mod levels;
mod middleware;
<<<<<<< HEAD
mod tests;

=======
mod routes;
>>>>>>> 4b6173f (feat: Adds Levels Route)
use crate::middleware as common_middleware;

pub async fn ws_index(
    r: HttpRequest,
    stream: web::Payload,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    println! {"{:?}",r};
    let res = actor::implementation::ws::start(actor::implementation::Game::new(pool), &r, stream);
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
            .configure(leaderboard::routes::leaderboard_routes)
            .wrap(auth::middleware::CheckAuth {})
            .wrap(IdentityService::new(auth::middleware::cookie_policy()))
            .wrap(common_middleware::cors_config())
            .wrap(mw::Logger::default())
            .service(web::resource("/ws/").route(web::get().to(ws_index)))
            .service(fs::Files::new("/events", "static/").index_file("index.html"))
            .configure(auth::routes::auth_routes)
            .configure(levels::routes::level_select_routes)
            .configure(game::routes::game_routes)
    })
    .bind(&app_url)?
    .run()
    .await
}

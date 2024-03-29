#[macro_use]
extern crate diesel;
extern crate dotenv;

use crate::db::utils::create_db_pool;
use actix_files as fs;
use actix_identity::IdentityService;
use actix_web::{web, App, HttpServer};

use tracing_actix_web::TracingLogger;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

use dotenv::dotenv;

mod actor;
mod auth;
mod db;
mod game;
mod leaderboard;
mod middleware;
mod playerstats;
mod utils;

use crate::middleware as common_middleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let res = dotenv();
    if res.is_err() {
        println!("Env error");
        println!("{:?}", res);
        std::process::exit(1)
    }

    LogTracer::init().expect("Unable to setup log tracer!");

    let app_name = "outbreak_server".to_string();
    let log_file = tracing_appender::rolling::hourly("logs", "server.log");
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(log_file);
    let bunyan_formatting_layer = BunyanFormattingLayer::new(app_name, non_blocking_writer);
    let subscriber = Registry::default()
        .with(EnvFilter::new("INFO"))
        .with(JsonStorageLayer)
        .with(bunyan_formatting_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let pool = create_db_pool();
    let app_url = dotenv::var("APP_URL").unwrap();

    HttpServer::new(move || {
        App::new()
            // set up DB pool to be used with web::Data<Pool> extractor
            .data(pool.clone())
            .wrap(auth::middleware::CheckAuth {})
            .wrap(IdentityService::new(auth::middleware::cookie_policy()))
            .wrap(common_middleware::cors_config())
            .wrap(TracingLogger)
            .service(web::resource("/ws/").route(web::get().to(actor::routes::ws_index)))
            .service(fs::Files::new("/events", "static/").index_file("index.html"))
            .configure(auth::routes::auth_routes)
            .configure(playerstats::routes::stats_routes)
            .configure(game::routes::game_routes)
            .configure(leaderboard::routes::leaderboard_routes)
    })
    .bind(&app_url)?
    .run()
    .await
}

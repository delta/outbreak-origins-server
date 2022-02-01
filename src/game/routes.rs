<<<<<<< HEAD
use crate::auth::extractors::Authenticated;
use crate::db::types::PgPool;
use crate::game::response;
use crate::levels::controllers::get_current_level;
use actix_web::{get, web, Error, HttpResponse};
use std::fs::File;
=======
use actix_web::{post, web, Error, HttpResponse};
use std::fs::File;

>>>>>>> a13592d (feat: Adds start-level route)
use std::io::Read;
// #[get("/money")]
// async fn get_money() -> Result<HttpResponse, Error> {

// }

// #[get("/level-score")]
// async fn get_level_score() -> Result<HttpResponse, Error> {

// }

// #[get("/active-control-measures/")]
// async fn get_active_control_measures() -> Result<HttpResponse, Error> {

// }

// #[get("/active-events/")]
// async fn get_active_events() -> Result<HttpResponse, Error> {

// }

// #[get("/event")]
// async fn get_event() -> Result<HttpResponse, Error> {

// }

// #[post("/apply")]
// async fn apply_control_measures() -> Result<HttpResponse, Error> {

// }

// #[post("/handle-event/")]
// async fn handle_event() -> Result<HttpResponse, Error> {

// }

// #[post("/end-level")]
// async fn end_level() -> Result<HttpResponse, Error> {

// }

#[get("/start-level")]
async fn start_level(
    user: Authenticated,
    pool: web::Data<PgPool>,
    level: web::Query<response::StartLevelRequest>,
) -> Result<HttpResponse, Error> {
    let cur_level = get_current_level(&pool.get().unwrap(), user);
    if cur_level < level.level {
        Ok(HttpResponse::Ok().json(response::StartLevelError {
            message: format!("Level {} is not yet unlocked", level.level),
        }))
    } else {
        let mut file =
            File::open(format!("src/game/levels/{}/level_start.json", level.level)).unwrap();
        let mut json_string = String::new();
        file.read_to_string(&mut json_string).unwrap();
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(json_string))
    }
}

pub fn game_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/user/api/").service(start_level));
}

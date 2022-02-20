use crate::auth::extractors::Authenticated;
use crate::db::types::PgPool;
use crate::game::controllers::get_active_control_measures;
use crate::game::response;
use crate::levels::controllers::get_current_level;
use actix_web::{get, web, Error, HttpResponse};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

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

#[get("/active-control-measures")]
async fn active_control_measures(
    user: Authenticated,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let acm_res = get_active_control_measures(&pool.get().unwrap(), user).map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().json(response::ActiveControlMeasuresResponse {
            num_control_measures: 0,
            active_control_measures: HashMap::new(),
        })
    })?;
    Ok(HttpResponse::Ok().json(acm_res))
}

pub fn game_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user/api/")
            .service(start_level)
            .service(active_control_measures),
    );
}

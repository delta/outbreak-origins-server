use crate::auth::extractors::Authenticated;
use crate::db::types::PgPool;
use crate::game::controllers::get_active_control_measures;
use crate::game::response;
use crate::levels::controllers::{get_current_level, update_user_at_level_end};
use actix_web::{get, post, web, Error, HttpResponse};
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
            active_control_measures: HashMap::new(),
        })
    })?;
    Ok(HttpResponse::Ok().json(acm_res))
}

#[post("/end-level")]
async fn end_level(
    user: Authenticated,
    pool: web::Data<PgPool>,
    data: web::Json<response::EndLevelRequest>,
) -> Result<HttpResponse, Error> {
    let mortality = 0.8;
    let population = 15000.0;
    let start_money = 1000.0;

    let deaths = (data.removed * mortality) / population;
    let caseload = (data.infected + data.removed) / (2.0 * population);
    let money_left = data.money_left / start_money;

    let deaths_weight = -4.0; // negative cuz more deaths means less score
    let caseload_weight = -1.5; // same with caseload
    let money_weight = 2.0; // positive cuz more money remaining means better score
    let score_scale = 1000.0;

    let performance_factor =
        deaths * deaths_weight + caseload * caseload_weight + money_left * money_weight; // will be between [0 and sum_of_weights]

    let score = score_scale * (10.0 + performance_factor);

    match update_user_at_level_end(&pool.get().unwrap(), user) {
        Ok(_) => Ok(HttpResponse::Ok().json(response::EndLevelResponse {
            message: "Success".to_string(),
            score,
        })),
        Err(_) => Ok(
            HttpResponse::InternalServerError().json(response::EndLevelResponse {
                message: "Failed".to_string(),
                score: 0.0,
            }),
        ),
    }
}

pub fn game_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user/api/")
            .service(start_level)
            .service(active_control_measures)
            .service(end_level),
    );
}

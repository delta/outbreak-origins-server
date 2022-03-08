use crate::auth::extractors::Authenticated;
use crate::db::types::PgPool;
use crate::game::controllers::get_active_control_measures;
use crate::game::controllers::{update_user_at_level_end};
use crate::levels::controllers::{get_current_level};
use crate::game::response;
use actix_web::{get, post, web, Error, HttpResponse};
use std::collections::HashMap;
use tracing::{error, info, instrument};
use std::fs::File;
use std::io::Read;


#[get("/active-control-measures")]
#[instrument(skip(pool))]
async fn active_control_measures(
    user: Authenticated,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let acm_res = get_active_control_measures(&pool.get().unwrap(), user).map_err(|e| {
        error!("Couldn't get active control measures: {}", e);
        HttpResponse::InternalServerError().json(response::ActiveControlMeasuresResponse {
            active_control_measures: HashMap::new(),
        })
    })?;
    Ok(HttpResponse::Ok().json(acm_res))
}

#[post("/end-level")]
#[instrument(skip(pool))]
async fn end_level(
    user: Authenticated,
    pool: web::Data<PgPool>,
    data: web::Json<response::EndLevelRequest>,
) -> Result<HttpResponse, Error> {
    let cur_level = get_current_level(&pool.get().unwrap(), &user);
    let file = File::open(format!("src/game/levels/{}/endLevel.json", cur_level)).unwrap();
    let end_level_data: response::EndLevelData = serde_json::from_reader(file).unwrap();
    let mortality = end_level_data.mortality;
    let population = 15000.0;
    let start_money = end_level_data.start_money;
    let deaths = (data.removed * mortality) / population;
    let caseload = (data.infected + data.removed) / (2.0 * population);
    let money_left = data.money_left / start_money;

    let deaths_weight = -20.0; // negative cuz more deaths means less score
    let caseload_weight = -5.0; // same with caseload
    let money_weight = 0.25; // positive cuz more money remaining means better score
    let score_scale = 1000.0;

    let performance_factor =
        deaths * deaths_weight + caseload * caseload_weight + money_left * money_weight; // will be between [0 and sum_of_weights]

    let score = score_scale * (20.0 + performance_factor);

    match update_user_at_level_end(&pool.get().unwrap(), user ,score as i32, start_money) {
        Ok(_) => {
            info!("User ended level successfully");
            Ok(HttpResponse::Ok().json(response::EndLevelResponse {
                message: "Success".to_string(),
                score,
            }))
        }
        Err(e) => {
            error!("Couldn't update user: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(response::EndLevelResponse {
                    message: "Failed".to_string(),
                    score: 0.0,
                }),
            )
        }
    }
}

pub fn game_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user/api/")
            .service(active_control_measures)
            .service(end_level),
    );
}

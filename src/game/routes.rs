use crate::auth::extractors::Authenticated;
use crate::db::types::PgPool;
use crate::game::controllers::get_active_control_measures;
use crate::game::response;
use crate::levels::controllers::update_user_at_level_end;
use actix_web::{get, post, web, Error, HttpResponse};
use std::collections::HashMap;
use tracing::{instrument, error, info};

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
        Ok(_) => {
            info!("User ended level successfully");
            Ok(HttpResponse::Ok().json(response::EndLevelResponse {
                message: "Success".to_string(),
                score,
            }))
        },
        Err(e) => {
            error!("Couldn't update user: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(response::EndLevelResponse {
                    message: "Failed".to_string(),
                    score: 0.0,
                }),
            )
        },
    }
}

pub fn game_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user/api/")
            .service(active_control_measures)
            .service(end_level),
    );
}

use crate::auth::extractors::Authenticated;
use crate::db::types::PgPool;
use crate::game::controllers::{
    change_level_type, get_active_control_measures, get_current_level, update_user_at_level_end,
};
use crate::game::{requests, response};
use actix_web::{get, http::StatusCode, post, web, Error, HttpResponse};
use std::collections::HashMap;
use std::fs::File;
use tracing::{error, info, instrument};

#[get("/dashboard")]
async fn level_details(
    user: Authenticated,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let email = user.0.unwrap().email;
    let (curr_level, is_randomized, is_active, retries_left) =
        web::block(move || get_current_level(&pool.get().unwrap(), email))
            .await
            .map_err(|e| {
                error!("Couldn't get current level: {}", e);
                HttpResponse::InternalServerError().json(response::LevelError {
                    message: "Couldn't get current level".to_string(),
                })
            })?;
    Ok(HttpResponse::Ok().json(response::LevelResponse {
        cur_level: if retries_left < 1 { -1 } else { curr_level },
        is_active,
        is_randomized,
        retries_left,
    }))
}

#[post("/change-level")]
async fn change_level(
    user: Authenticated,
    pool: web::Data<PgPool>,
    level_type: web::Json<requests::ChangeLevelRequest>,
) -> Result<HttpResponse, Error> {
    let status = web::block(move || {
        let conn = pool.get().unwrap();
        change_level_type(&conn, user, level_type.is_randomized)
    })
    .await
    .map_err(|e| {
        error!("Couldn't change level: {}", e);
        HttpResponse::InternalServerError().json(response::LevelError {
            message: "Couldn't change current level".to_string(),
        })
    })?;
    Ok(HttpResponse::Ok()
        .status(if status {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        })
        .json(response::ChangeLevelResponse { status }))
}

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
    data: web::Json<requests::EndLevelRequest>,
) -> Result<HttpResponse, Error> {
    let email = user.0.as_ref().unwrap().email.clone();
    let conn1 = pool.get().unwrap();
    let (cur_level, _, _, _) = web::block(move || get_current_level(&conn1, email.clone()))
        .await
        .map_err(|e| {
            error!("Couldn't get level: {}", e);
            HttpResponse::InternalServerError().json(response::EndLevelResponse {
                message: "Failed".to_string(),
                score: 0.0,
            })
        })?;
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

    match update_user_at_level_end(&pool.get().unwrap(), user, score as i32, start_money) {
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
            .service(end_level)
            .service(level_details)
            .service(change_level),
    );
}

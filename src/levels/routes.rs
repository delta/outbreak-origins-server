use crate::auth::extractors::Authenticated;
use crate::db::types::PgPool;
use crate::levels::requests;
use crate::levels::{controllers, response};
use actix_web::{get, http::StatusCode, post, web, Error, HttpResponse};
use tracing::{error, instrument};

#[get("")]
#[instrument(skip(pool))]
async fn level_details(
    user: Authenticated,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let (curr_level, is_randomized, is_active) =
        web::block(move || controllers::get_current_level(&pool.get().unwrap(), user))
            .await
            .map_err(|e| {
                error!("Couldn't get current level: {}", e);
                HttpResponse::InternalServerError().json(response::LevelError {
                    message: "Couldn't get current level".to_string(),
                })
            })?;
    Ok(HttpResponse::Ok().json(response::LevelResponse {
        cur_level: curr_level,
        is_active,
        is_randomized,
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
        controllers::change_level_type(&conn, user, level_type.is_randomized)
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

pub fn level_select_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user/api/dashboard")
            .service(level_details)
            .service(change_level),
    );
}

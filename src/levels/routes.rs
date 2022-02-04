use crate::auth::extractors::Authenticated;
use crate::db::types::PgPool;
use crate::levels::{controllers, response};
use actix_web::{get, web, Error, HttpResponse};

#[get("")]
async fn level_details(
    web::Query(level): web::Query<response::LevelRequest>,
    user: Authenticated,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let level = level.level;
    web::block(move || {
        let conn = pool.get()?;
        controllers::update_current_level(&conn, level, user)
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().json(response::LevelResponse {
            message: String::from("Failed"),
        })
    })?;
    Ok(HttpResponse::Ok().json(response::LevelResponse {
        message: String::from("Success"),
    }))
}

pub fn level_select_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/user/api/dashboard").service(level_details));
}

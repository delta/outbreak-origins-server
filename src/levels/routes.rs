use crate::auth::extractors::Authenticated;
use crate::db::types::PgPool;
use crate::levels::{controllers, response};
use actix_web::{get, web, Error, HttpResponse};

#[get("")]
async fn level_details(
    user: Authenticated,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let mut curr_level = controllers::get_current_level(&pool.get().unwrap(), &user);
    let retryattemptsleft = controllers::get_retry_attempts(&pool.get().unwrap(), &user);
    if retryattemptsleft < 1 {
        curr_level = -1;
    }
    Ok(HttpResponse::Ok().json(response::LevelResponse {
        cur_level: curr_level,
    }))
}

pub fn level_select_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/user/api/dashboard").service(level_details));
}

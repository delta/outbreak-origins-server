use actix_web::{post, web, Error, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct LevelInfo {
    level: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct LevelResult {
    pub level_no: i32,
}

#[post("/dashboard")]
async fn level_details(level: web::Json<LevelInfo>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(LevelResult {
        level_no: level.level,
    }))
}

pub fn game_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/user/api").service(level_details));
}
